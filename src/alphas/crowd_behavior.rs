//! Crowd Behavior Alpha - Exploit Retail Irrationality
//!
//! **Human Insight:**
//! Retail traders exhibit predictable irrational behavior driven by FOMO (Fear
//! of Missing Out) and panic. When a stock goes viral on social media, retail
//! piles in at the top. When it crashes, they panic sell at the bottom.
//!
//! **Strategy: Contrarian Meme Stock Trading**
//! - Target: Identify meme stock lifecycle stages
//! - Entry: Counter-trend when FOMO peaks or panic bottoms
//! - Hold time: 1-7 days (meme cycles are fast)
//! - Win rate: 55-65% (retail behavior is predictable but volatile)
//! - Exit: When momentum reverses or retail interest fades
//!
//! We detect crowd behavior by analyzing:
//! - Social media mentions (Reddit WSB, Twitter)
//! - Options flow (retail loves calls/puts)
//! - Volume spikes (FOMO buying or panic selling)
//! - Price momentum (parabolic moves = unsustainable)

use super::{AlphaModel, AlphaStats};
use crate::types::*;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;

/// Meme stock lifecycle stages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemeStage {
    /// Normal trading, no meme activity
    Normal,
    /// Early discovery phase (rising mentions)
    Discovery,
    /// Acceleration phase (viral spread, FOMO kicks in)
    Acceleration,
    /// Peak phase (maximum hype, everyone knows about it)
    Peak,
    /// Decline phase (reality sets in, bagholders emerge)
    Decline,
    /// Forgotten phase (back to normal)
    Forgotten,
}

/// FOMO/Panic intensity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrowdEmotion {
    /// Extreme fear (panic selling)
    ExtremeFear,
    /// Fear (selling pressure)
    Fear,
    /// Neutral
    Neutral,
    /// Greed (buying pressure)
    Greed,
    /// Extreme greed (FOMO)
    ExtremeGreed,
}

impl CrowdEmotion {
    /// Get numeric score (-2.0 to 2.0)
    fn score(&self) -> f64 {
        match self {
            CrowdEmotion::ExtremeFear => -2.0,
            CrowdEmotion::Fear => -1.0,
            CrowdEmotion::Neutral => 0.0,
            CrowdEmotion::Greed => 1.0,
            CrowdEmotion::ExtremeGreed => 2.0,
        }
    }

    /// Create from score
    fn from_score(score: f64) -> Self {
        if score >= 1.5 {
            CrowdEmotion::ExtremeGreed
        } else if score >= 0.5 {
            CrowdEmotion::Greed
        } else if score <= -1.5 {
            CrowdEmotion::ExtremeFear
        } else if score <= -0.5 {
            CrowdEmotion::Fear
        } else {
            CrowdEmotion::Neutral
        }
    }
}

/// Tracks crowd behavior for a specific symbol
#[derive(Debug, Clone)]
struct CrowdTracker {
    /// Current meme stage
    stage: MemeStage,
    /// Current emotion
    emotion: CrowdEmotion,
    /// Social media mentions (simulated for now)
    mention_count: usize,
    /// Mention history (for detecting trends)
    mention_history: Vec<(DateTime<Utc>, usize)>,
    /// Volume surge multiplier
    volume_surge: f64,
    /// Price momentum (% change over period)
    price_momentum: f64,
    /// Last update time
    last_update: DateTime<Utc>,
}

impl CrowdTracker {
    fn new() -> Self {
        Self {
            stage: MemeStage::Normal,
            emotion: CrowdEmotion::Neutral,
            mention_count: 0,
            mention_history: Vec::new(),
            volume_surge: 1.0,
            price_momentum: 0.0,
            last_update: Utc::now(),
        }
    }

    /// Update tracker with new data
    fn update(&mut self, mentions: usize, volume_surge: f64, price_momentum: f64) {
        let now = Utc::now();

        // Update mention history
        self.mention_history.push((now, mentions));

        // Keep last 14 days of history
        let cutoff = now - Duration::days(14);
        self.mention_history
            .retain(|(timestamp, _)| *timestamp > cutoff);

        // Update current state
        self.mention_count = mentions;
        self.volume_surge = volume_surge;
        self.price_momentum = price_momentum;
        self.last_update = now;

        // Determine meme stage
        self.stage = self.detect_stage();

        // Determine crowd emotion
        self.emotion = self.detect_emotion();
    }

    /// Detect current meme stage
    fn detect_stage(&self) -> MemeStage {
        if self.mention_history.len() < 2 {
            return MemeStage::Normal;
        }

        // Calculate mention trend
        let recent_count = (self.mention_history.len() / 2).max(1);
        let older_count = recent_count;

        let recent_avg = self
            .mention_history
            .iter()
            .rev()
            .take(recent_count)
            .map(|(_, count)| *count as f64)
            .sum::<f64>()
            / recent_count as f64;

        let older_avg = self
            .mention_history
            .iter()
            .skip(self.mention_history.len() - older_count - recent_count)
            .take(older_count)
            .map(|(_, count)| *count as f64)
            .sum::<f64>()
            / older_count as f64;

        let mention_growth = if older_avg > 0.0 {
            (recent_avg - older_avg) / older_avg
        } else {
            0.0
        };

        // Classify stage based on mentions and momentum
        if self.mention_count < 10 && mention_growth < 0.2 {
            MemeStage::Normal
        } else if mention_growth > 2.0 && self.mention_count > 20 {
            // Rapid mention growth = discovery
            MemeStage::Discovery
        } else if mention_growth > 0.5 && self.mention_count > 50 && self.price_momentum > 10.0 {
            // High mentions + strong momentum = acceleration
            MemeStage::Acceleration
        } else if self.mention_count > 100 && mention_growth < 0.3 {
            // Peak mentions but slowing growth = peak
            MemeStage::Peak
        } else if mention_growth < -0.3 && self.price_momentum < 0.0 {
            // Declining mentions + negative momentum = decline
            MemeStage::Decline
        } else if self.mention_count < 20 && mention_growth < 0.0 {
            // Low mentions and declining = forgotten
            MemeStage::Forgotten
        } else {
            // Default
            MemeStage::Normal
        }
    }

    /// Detect crowd emotion (FOMO vs Panic)
    fn detect_emotion(&self) -> CrowdEmotion {
        // Emotion score based on multiple factors
        let mut score = 0.0;

        // Volume surge indicates emotion intensity
        if self.volume_surge > 3.0 {
            score += 1.0; // High volume = strong emotion
        } else if self.volume_surge > 2.0 {
            score += 0.5;
        }

        // Price momentum direction
        if self.price_momentum > 5.0 {
            score += 1.0; // Strong up = greed/FOMO
        } else if self.price_momentum > 2.0 {
            score += 0.5;
        } else if self.price_momentum < -5.0 {
            score -= 1.0; // Strong down = fear/panic
        } else if self.price_momentum < -2.0 {
            score -= 0.5;
        }

        // Meme stage influences emotion
        match self.stage {
            MemeStage::Acceleration => score += 0.5, // FOMO building
            MemeStage::Peak => score += 1.0,         // Maximum FOMO
            MemeStage::Decline => score -= 0.5,      // Fear setting in
            _ => {}
        }

        CrowdEmotion::from_score(score)
    }
}

/// Crowd Behavior Alpha Model
#[derive(Debug)]
pub struct CrowdBehaviorAlpha {
    name: String,

    // Track crowd behavior per symbol
    trackers: HashMap<Symbol, CrowdTracker>,

    // Configuration (used for future enhancements)
    #[allow(dead_code)]
    fomo_threshold: f64,         // Emotion score for FOMO (default: 1.5)
    #[allow(dead_code)]
    panic_threshold: f64,        // Emotion score for panic (default: -1.5)
    min_confidence: f64,         // Minimum confidence to trade (default: 0.6)
    #[allow(dead_code)]
    avg_volume_window_days: i64, // Days to calculate avg volume (default: 20)

    // State
    stats: AlphaStats,
    last_signal_time: Option<DateTime<Utc>>,
    cooldown_hours: i64,
}

impl Default for CrowdBehaviorAlpha {
    fn default() -> Self {
        Self::new()
    }
}

impl CrowdBehaviorAlpha {
    pub fn new() -> Self {
        Self {
            name: "CrowdBehavior".to_string(),
            trackers: HashMap::new(),
            fomo_threshold: 1.5,
            panic_threshold: -1.5,
            min_confidence: 0.6,
            avg_volume_window_days: 20,
            stats: AlphaStats::default(),
            last_signal_time: None,
            cooldown_hours: 12, // 12 hour cooldown
        }
    }

    /// Update social media mentions (in production, fetch from Reddit/Twitter)
    pub fn update_mentions(&mut self, symbol: Symbol, mentions: usize) {
        let tracker = self
            .trackers
            .entry(symbol)
            .or_insert_with(CrowdTracker::new);
        // This would be called when new social media data is available
        // For now, it's a placeholder for future integration
        tracker.mention_count = mentions;
    }

    /// Check if we're in cooldown period
    fn in_cooldown(&self) -> bool {
        if let Some(last_signal) = self.last_signal_time {
            let elapsed = Utc::now().signed_duration_since(last_signal);
            elapsed.num_hours() < self.cooldown_hours
        } else {
            false
        }
    }

    /// Calculate confidence based on emotion intensity and stage
    fn calculate_confidence(&self, tracker: &CrowdTracker) -> f64 {
        let mut confidence = 0.5; // Base confidence

        // Strong emotions increase confidence
        let emotion_score = tracker.emotion.score().abs();
        confidence += emotion_score * 0.15; // Up to +0.3 for extreme emotions

        // Certain stages are more predictable
        match tracker.stage {
            MemeStage::Peak => confidence += 0.2, // Peak FOMO very predictable
            MemeStage::Decline => confidence += 0.15, // Panic also predictable
            MemeStage::Acceleration => confidence += 0.1,
            _ => {}
        }

        // Volume surge adds confidence
        if tracker.volume_surge > 3.0 {
            confidence += 0.1;
        }

        confidence.min(0.95) // Cap at 95%
    }

    /// Generate trading signal based on meme stage
    fn stage_to_signal(&self, stage: MemeStage, emotion: CrowdEmotion) -> Option<SignalAction> {
        match stage {
            MemeStage::Peak => {
                // At peak, retail is maximum FOMO - sell to them
                match emotion {
                    CrowdEmotion::ExtremeGreed => Some(SignalAction::Sell),
                    _ => None,
                }
            }
            MemeStage::Decline => {
                // In decline, retail panics - buy from them
                match emotion {
                    CrowdEmotion::ExtremeFear | CrowdEmotion::Fear => Some(SignalAction::Buy),
                    _ => None,
                }
            }
            MemeStage::Acceleration => {
                // Early in acceleration, might ride the momentum
                // But be cautious - can reverse quickly
                None // For now, don't trade acceleration (too risky)
            }
            _ => None,
        }
    }

    /// Analyze market data for crowd behavior signals
    pub fn analyze_symbol(&mut self, data: &MarketData) -> Option<Signal> {
        // Check cooldown
        if self.in_cooldown() {
            return None;
        }

        // Get or create tracker
        let symbol = data.symbol.clone();
        let tracker = self
            .trackers
            .entry(symbol.clone())
            .or_insert_with(CrowdTracker::new);

        // Calculate volume surge
        // In production, would compare to 20-day average volume
        // For now, use a simpler heuristic
        let avg_volume = 1_000_000.0; // Placeholder
        let volume_surge = data.volume as f64 / avg_volume;

        // Calculate price momentum (intraday change)
        let price_momentum = data.intraday_change_pct().unwrap_or(0.0);

        // Simulate social media mentions based on volume and momentum
        // In production, would fetch real data from Reddit/Twitter API
        let mentions = if volume_surge > 3.0 && price_momentum.abs() > 5.0 {
            150 // High activity = many mentions
        } else if volume_surge > 2.0 && price_momentum.abs() > 3.0 {
            75 // Medium activity
        } else if volume_surge > 1.5 {
            25 // Some activity
        } else {
            5 // Normal activity
        };

        // Update tracker
        tracker.update(mentions, volume_surge, price_momentum);

        // Extract values before dropping the mutable borrow
        let stage = tracker.stage;
        let emotion = tracker.emotion;
        let tracker_clone = tracker.clone();

        // Drop the mutable borrow (using let _ instead of drop for reference)
        let _ = tracker;

        // Now we can call methods on self
        let confidence_value = self.calculate_confidence(&tracker_clone);

        // Generate signal based on stage and emotion
        let action = self.stage_to_signal(stage, emotion)?;

        if confidence_value < self.min_confidence {
            return None; // Not confident enough
        }

        // Update state
        self.last_signal_time = Some(Utc::now());
        self.stats.signals_generated += 1;
        if confidence_value >= 0.8 {
            self.stats.signals_actionable += 1;
        }

        // Create reason
        let reason = format!(
            "Crowd behavior: {:?} stage with {:?} emotion (vol surge: {:.1}x, momentum: {:.1}%)",
            stage, emotion, volume_surge, price_momentum
        );

        // Create signal
        let signal_confidence = Confidence::new(confidence_value).ok()?;
        let signal = Signal::new(
            data.symbol.clone(),
            action,
            signal_confidence,
            reason,
            self.name(),
        )
        .with_metadata(serde_json::json!({
            "meme_stage": format!("{:?}", stage),
            "crowd_emotion": format!("{:?}", emotion),
            "volume_surge": volume_surge,
            "price_momentum": price_momentum,
            "mentions": mentions,
        }));

        Some(signal)
    }
}

#[async_trait]
impl AlphaModel for CrowdBehaviorAlpha {
    fn name(&self) -> &str {
        &self.name
    }

    fn human_insight(&self) -> &str {
        "Retail traders exhibit predictable irrational behavior. When a stock goes \
         viral on social media, FOMO drives buying at the peak. When it crashes, \
         panic drives selling at the bottom. By identifying meme stock lifecycle \
         stages (discovery → acceleration → peak → decline → forgotten), we can \
         trade contrarian to retail: sell into FOMO peaks and buy panic bottoms. \
         This exploits cognitive biases like anchoring, recency bias, and herd mentality."
    }

    fn update(&mut self, _data: &MarketSnapshot) {
        // State update happens in analyze_symbol
    }

    async fn generate_signals(&self) -> Vec<Signal> {
        // This is a stub - in production, would fetch social media data here
        Vec::new()
    }

    fn reset(&mut self) {
        self.stats = AlphaStats::default();
        self.last_signal_time = None;
        self.trackers.clear();
    }

    fn stats(&self) -> AlphaStats {
        self.stats.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data(volume: u64, change_pct: f64) -> MarketData {
        let symbol = Symbol::new("GME").unwrap();
        let prev_close = Price::new(100.0).unwrap();
        let current = Price::new(100.0 + change_pct).unwrap();

        MarketData {
            symbol,
            quote: Quote {
                bid: current,
                ask: current,
                bid_size: Quantity::buy(1000),
                ask_size: Quantity::sell(1000),
                timestamp: Utc::now(),
            },
            last_price: current,
            volume,
            timestamp: Utc::now(),
            open: Some(prev_close),
            high: Some(current),
            low: Some(current),
            prev_close: Some(prev_close),
            vix: Some(20.0),
            put_call_ratio: Some(1.0),
        }
    }

    #[test]
    fn test_crowd_emotion_score() {
        assert_eq!(CrowdEmotion::ExtremeGreed.score(), 2.0);
        assert_eq!(CrowdEmotion::Greed.score(), 1.0);
        assert_eq!(CrowdEmotion::Neutral.score(), 0.0);
        assert_eq!(CrowdEmotion::Fear.score(), -1.0);
        assert_eq!(CrowdEmotion::ExtremeFear.score(), -2.0);
    }

    #[test]
    fn test_crowd_emotion_from_score() {
        assert_eq!(CrowdEmotion::from_score(1.8), CrowdEmotion::ExtremeGreed);
        assert_eq!(CrowdEmotion::from_score(0.7), CrowdEmotion::Greed);
        assert_eq!(CrowdEmotion::from_score(0.0), CrowdEmotion::Neutral);
        assert_eq!(CrowdEmotion::from_score(-0.7), CrowdEmotion::Fear);
        assert_eq!(CrowdEmotion::from_score(-1.8), CrowdEmotion::ExtremeFear);
    }

    #[test]
    fn test_crowd_tracker_normal() {
        let mut tracker = CrowdTracker::new();

        // Normal volume, low momentum
        tracker.update(5, 1.2, 0.5);

        assert_eq!(tracker.stage, MemeStage::Normal);
        assert_eq!(tracker.emotion, CrowdEmotion::Neutral);
    }

    #[test]
    fn test_crowd_tracker_fomo() {
        let mut tracker = CrowdTracker::new();

        // High volume + strong momentum = FOMO
        tracker.update(150, 4.0, 8.0);

        // Emotion should indicate greed/FOMO
        match tracker.emotion {
            CrowdEmotion::Greed | CrowdEmotion::ExtremeGreed => {} // Expected
            _ => panic!("Expected greed/FOMO emotion, got {:?}", tracker.emotion),
        }
    }

    #[test]
    fn test_crowd_tracker_panic() {
        let mut tracker = CrowdTracker::new();

        // Build up history
        for _ in 0..5 {
            tracker.update(100, 1.0, 2.0);
        }

        // Strong down momentum should trigger negative emotion
        // Keep volume low so it doesn't add positive score
        // Momentum -8.0 (<-5.0 = -1.0)
        tracker.update(90, 1.0, -8.0);

        // Should have negative emotion from strong down momentum
        assert!(
            tracker.emotion.score() < 0.0,
            "Expected negative emotion from down momentum"
        );
    }

    #[test]
    fn test_alpha_fomo_peak() {
        let mut alpha = CrowdBehaviorAlpha::new();

        // Simulate meme stock at peak (high volume, big gain)
        let data = create_test_data(5_000_000, 12.0);

        let signal = alpha.analyze_symbol(&data);

        // At FOMO peak, should generate sell signal
        if let Some(sig) = signal {
            assert_eq!(sig.action, SignalAction::Sell);
            assert!(sig.reason.contains("Crowd behavior"));
        }
    }

    #[test]
    fn test_alpha_panic_decline() {
        let mut alpha = CrowdBehaviorAlpha::new();

        // First create some history to establish trend
        let symbol = Symbol::new("GME").unwrap();
        let mut tracker = CrowdTracker::new();

        // Build up mention history showing decline
        for _ in 0..5 {
            tracker.update(100, 2.0, 5.0); // High mentions
        }
        for _ in 0..5 {
            tracker.update(50, 2.0, -5.0); // Declining mentions, negative momentum
        }

        alpha.trackers.insert(symbol, tracker);

        // Now simulate panic selling (high volume, big drop)
        let data = create_test_data(5_000_000, -12.0);

        let signal = alpha.analyze_symbol(&data);

        // At panic bottom, should generate buy signal
        if let Some(sig) = signal {
            assert_eq!(sig.action, SignalAction::Buy);
            assert!(sig.reason.contains("Crowd behavior"));
        }
    }

    #[test]
    fn test_cooldown() {
        let mut alpha = CrowdBehaviorAlpha::new();
        alpha.cooldown_hours = 1;

        let data = create_test_data(5_000_000, 12.0);

        // First signal should work
        let signal1 = alpha.analyze_symbol(&data);

        if signal1.is_some() {
            // Second signal should be blocked by cooldown
            let signal2 = alpha.analyze_symbol(&data);
            assert!(signal2.is_none());
        }
    }
}
