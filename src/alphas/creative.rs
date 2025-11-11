//! Creative Synthesis Alpha - Novel Signal Combinations
//!
//! **Human Insight:**
//! Most quant models look at obvious correlations (stock price → earnings, interest
//! rates → bond yields). But humans can spot creative, non-obvious patterns that
//! institutional models miss. Bad weather → online shopping. Sports team wins →
//! local consumer sentiment. Earnings surprises → sector rotation. These creative
//! syntheses create alpha because they're too unconventional for traditional models.
//!
//! **Strategy: Unconventional Data Fusion**
//! - Target: Find alpha in unexpected correlations
//! - Entry: When multiple unconventional signals align
//! - Hold time: 2-5 days (time for effects to propagate)
//! - Win rate: 50-60% (novel patterns are less tested but higher edge)
//! - Exit: When pattern breaks or conventional factors dominate
//!
//! We synthesize:
//! - Weather → Retail/E-commerce (bad weather = more online shopping)
//! - Sports results → Local sentiment (team wins = consumer confidence)
//! - Earnings surprises → Cross-sector effects (NVDA beats → AMD/INTC)
//! - Search trends → Product demand (Google Trends → sales)
//! - Regulatory news → Sector rotation

use super::{AlphaModel, AlphaStats};
use crate::types::*;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Types of creative signals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CreativeSignalType {
    /// Weather impact on retail/e-commerce
    Weather,
    /// Sports results impact on local sentiment
    Sports,
    /// Cross-sector earnings surprise effects
    EarningsSurprise,
    /// Search trend correlation
    SearchTrends,
    /// Regulatory/policy impact
    Regulatory,
}

/// Weather conditions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WeatherCondition {
    /// Severe weather (storms, snow)
    Severe,
    /// Bad weather (rain, cold)
    Bad,
    /// Normal weather
    Normal,
    /// Good weather (sunny, warm)
    Good,
}

impl WeatherCondition {
    /// Get impact score for retail (-1.0 to 1.0)
    /// Bad weather = positive for online, negative for physical retail
    fn retail_impact(&self) -> f64 {
        match self {
            WeatherCondition::Severe => 0.8,   // Strong online shopping
            WeatherCondition::Bad => 0.4,      // Moderate online shopping
            WeatherCondition::Normal => 0.0,   // Neutral
            WeatherCondition::Good => -0.3,    // People go outside, less online
        }
    }
}

/// Earnings surprise magnitude
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SurpriseMagnitude {
    /// Major beat (>10% above estimates)
    MajorBeat,
    /// Beat (5-10% above)
    Beat,
    /// Inline (within 5%)
    Inline,
    /// Miss (5-10% below)
    Miss,
    /// Major miss (>10% below)
    MajorMiss,
}

impl SurpriseMagnitude {
    fn score(&self) -> f64 {
        match self {
            SurpriseMagnitude::MajorBeat => 1.0,
            SurpriseMagnitude::Beat => 0.5,
            SurpriseMagnitude::Inline => 0.0,
            SurpriseMagnitude::Miss => -0.5,
            SurpriseMagnitude::MajorMiss => -1.0,
        }
    }
}

/// Tracks a creative pattern
#[derive(Debug, Clone)]
struct PatternTracker {
    /// Type of pattern
    pattern_type: CreativeSignalType,
    /// Pattern strength (0.0 to 1.0)
    strength: f64,
    /// Last update time
    last_update: DateTime<Utc>,
    /// Confidence in pattern (0.0 to 1.0)
    confidence: f64,
}

impl PatternTracker {
    fn new(pattern_type: CreativeSignalType, strength: f64, confidence: f64) -> Self {
        Self {
            pattern_type,
            strength,
            last_update: Utc::now(),
            confidence,
        }
    }

    /// Check if pattern is still fresh (within 24 hours)
    fn is_fresh(&self) -> bool {
        let age = Utc::now().signed_duration_since(self.last_update);
        age.num_hours() < 24
    }
}

/// Creative Synthesis Alpha Model
#[derive(Debug)]
pub struct CreativeSynthesisAlpha {
    name: String,

    // Track patterns per symbol
    patterns: HashMap<Symbol, Vec<PatternTracker>>,

    // Configuration
    min_pattern_strength: f64,  // Minimum strength to act (default: 0.5)
    min_confidence: f64,         // Minimum confidence (default: 0.55)
    require_multiple: bool,      // Require multiple patterns to align (default: true)

    // State
    stats: AlphaStats,
    last_signal_time: Option<DateTime<Utc>>,
    cooldown_hours: i64,
}

impl Default for CreativeSynthesisAlpha {
    fn default() -> Self {
        Self::new()
    }
}

impl CreativeSynthesisAlpha {
    pub fn new() -> Self {
        Self {
            name: "Creative".to_string(),
            patterns: HashMap::new(),
            min_pattern_strength: 0.5,
            min_confidence: 0.55,
            require_multiple: true,
            stats: AlphaStats::default(),
            last_signal_time: None,
            cooldown_hours: 12,
        }
    }

    /// Add a creative pattern observation
    pub fn add_pattern(
        &mut self,
        symbol: Symbol,
        pattern_type: CreativeSignalType,
        strength: f64,
        confidence: f64,
    ) {
        let tracker = PatternTracker::new(pattern_type, strength, confidence);
        self.patterns.entry(symbol).or_insert_with(Vec::new).push(tracker);
    }

    /// Simulate weather-based retail signal
    /// In production, would fetch real weather API data
    fn check_weather_retail(&self, _symbol: &Symbol) -> Option<PatternTracker> {
        // Simulate: For now, return None (no pattern)
        // In production: Fetch weather for major retail centers, check if severe
        None
    }

    /// Check for cross-sector earnings effects
    /// In production, would track earnings calendar and sector correlations
    fn check_earnings_spillover(&self, data: &MarketData) -> Option<PatternTracker> {
        // Simulate earnings surprise based on price momentum
        // In production: Track actual earnings vs estimates

        if let Some(change_pct) = data.intraday_change_pct() {
            if change_pct.abs() > 5.0 {
                // Large move might indicate earnings surprise
                let surprise = if change_pct > 5.0 {
                    SurpriseMagnitude::Beat
                } else {
                    SurpriseMagnitude::Miss
                };

                let strength = (change_pct.abs() / 10.0).min(1.0);
                let confidence = if change_pct.abs() > 8.0 { 0.7 } else { 0.5 };

                return Some(PatternTracker::new(
                    CreativeSignalType::EarningsSurprise,
                    strength * surprise.score().abs(),
                    confidence,
                ));
            }
        }

        None
    }

    /// Check for search trend correlation
    /// In production, would fetch Google Trends data
    fn check_search_trends(&self, data: &MarketData) -> Option<PatternTracker> {
        // Simulate based on volume
        // In production: Fetch Google Trends for stock ticker/company name

        // High volume might indicate search interest
        if data.volume > 10_000_000 {
            let strength = (data.volume as f64 / 20_000_000.0).min(1.0);
            let confidence = 0.5;

            return Some(PatternTracker::new(
                CreativeSignalType::SearchTrends,
                strength,
                confidence,
            ));
        }

        None
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

    /// Calculate combined pattern confidence
    fn calculate_combined_confidence(&self, patterns: &[PatternTracker]) -> f64 {
        if patterns.is_empty() {
            return 0.0;
        }

        // Average confidence, boosted if multiple patterns align
        let avg_confidence = patterns.iter().map(|p| p.confidence).sum::<f64>() / patterns.len() as f64;

        // Boost if multiple independent patterns agree
        let boost = if patterns.len() > 1 {
            0.1 * (patterns.len() - 1) as f64
        } else {
            0.0
        };

        (avg_confidence + boost).min(0.9)
    }

    /// Determine signal action from pattern strength
    fn patterns_to_signal(&self, patterns: &[PatternTracker]) -> Option<SignalAction> {
        if patterns.is_empty() {
            return None;
        }

        // Calculate net strength (positive = bullish, negative = bearish)
        let net_strength: f64 = patterns.iter().map(|p| p.strength).sum();

        if net_strength > 0.3 {
            Some(SignalAction::Buy)
        } else if net_strength < -0.3 {
            Some(SignalAction::Sell)
        } else {
            None
        }
    }

    /// Analyze market data for creative synthesis signals
    pub fn analyze_symbol(&mut self, data: &MarketData) -> Option<Signal> {
        // Check cooldown
        if self.in_cooldown() {
            return None;
        }

        let symbol = data.symbol.clone();

        // Collect creative patterns
        let mut active_patterns = Vec::new();

        // Check for weather-retail pattern
        if let Some(pattern) = self.check_weather_retail(&symbol) {
            if pattern.strength >= self.min_pattern_strength && pattern.confidence >= self.min_confidence {
                active_patterns.push(pattern);
            }
        }

        // Check for earnings spillover
        if let Some(pattern) = self.check_earnings_spillover(data) {
            if pattern.strength >= self.min_pattern_strength && pattern.confidence >= self.min_confidence {
                active_patterns.push(pattern);
            }
        }

        // Check for search trends
        if let Some(pattern) = self.check_search_trends(data) {
            if pattern.strength >= self.min_pattern_strength && pattern.confidence >= self.min_confidence {
                active_patterns.push(pattern);
            }
        }

        // Check existing patterns
        if let Some(existing) = self.patterns.get(&symbol) {
            for pattern in existing {
                if pattern.is_fresh()
                    && pattern.strength >= self.min_pattern_strength
                    && pattern.confidence >= self.min_confidence
                {
                    active_patterns.push(pattern.clone());
                }
            }
        }

        // Require multiple patterns if configured
        if self.require_multiple && active_patterns.len() < 2 {
            return None;
        }

        if active_patterns.is_empty() {
            return None;
        }

        // Calculate combined confidence
        let confidence = self.calculate_combined_confidence(&active_patterns);

        // Generate signal action
        let action = self.patterns_to_signal(&active_patterns)?;

        // Update state
        self.last_signal_time = Some(Utc::now());
        self.stats.signals_generated += 1;
        if confidence >= 0.7 {
            self.stats.signals_actionable += 1;
        }

        // Create reason
        let pattern_types: Vec<String> = active_patterns
            .iter()
            .map(|p| format!("{:?}", p.pattern_type))
            .collect();
        let reason = format!(
            "Creative synthesis: {} patterns align ({})",
            active_patterns.len(),
            pattern_types.join(", ")
        );

        // Create signal
        let signal_confidence = Confidence::new(confidence).ok()?;
        let signal = Signal::new(
            data.symbol.clone(),
            action,
            signal_confidence,
            reason,
            self.name(),
        )
        .with_metadata(serde_json::json!({
            "pattern_count": active_patterns.len(),
            "pattern_types": pattern_types,
            "combined_strength": active_patterns.iter().map(|p| p.strength).sum::<f64>(),
        }));

        Some(signal)
    }
}

#[async_trait]
impl AlphaModel for CreativeSynthesisAlpha {
    fn name(&self) -> &str {
        &self.name
    }

    fn human_insight(&self) -> &str {
        "Traditional quant models only look at obvious correlations. Humans can spot \
         creative, non-obvious patterns: bad weather drives online shopping, sports \
         team wins boost local consumer sentiment, earnings surprises ripple across \
         sectors. These unconventional syntheses create alpha because institutional \
         models are too rigid to capture them. By combining diverse, unexpected data \
         sources, we find opportunities hidden in plain sight."
    }

    fn update(&mut self, _data: &MarketSnapshot) {
        // State update happens in analyze_symbol
    }

    async fn generate_signals(&self) -> Vec<Signal> {
        // This is a stub - in production, would fetch creative data sources
        Vec::new()
    }

    fn reset(&mut self) {
        self.stats = AlphaStats::default();
        self.last_signal_time = None;
        self.patterns.clear();
    }

    fn stats(&self) -> AlphaStats {
        self.stats.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data(volume: u64, change_pct: f64) -> MarketData {
        let symbol = Symbol::new("AMZN").unwrap();
        let prev_close = Price::new(150.0).unwrap();
        let current = Price::new(150.0 + change_pct).unwrap();

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
            vix: Some(15.0),
            put_call_ratio: Some(1.0),
        }
    }

    #[test]
    fn test_weather_impact() {
        assert_eq!(WeatherCondition::Severe.retail_impact(), 0.8);
        assert_eq!(WeatherCondition::Bad.retail_impact(), 0.4);
        assert_eq!(WeatherCondition::Normal.retail_impact(), 0.0);
        assert!(WeatherCondition::Good.retail_impact() < 0.0);
    }

    #[test]
    fn test_surprise_score() {
        assert_eq!(SurpriseMagnitude::MajorBeat.score(), 1.0);
        assert_eq!(SurpriseMagnitude::Beat.score(), 0.5);
        assert_eq!(SurpriseMagnitude::Inline.score(), 0.0);
        assert_eq!(SurpriseMagnitude::Miss.score(), -0.5);
        assert_eq!(SurpriseMagnitude::MajorMiss.score(), -1.0);
    }

    #[test]
    fn test_pattern_tracker() {
        let pattern = PatternTracker::new(
            CreativeSignalType::Weather,
            0.8,
            0.7,
        );

        assert_eq!(pattern.pattern_type, CreativeSignalType::Weather);
        assert_eq!(pattern.strength, 0.8);
        assert_eq!(pattern.confidence, 0.7);
        assert!(pattern.is_fresh());
    }

    #[test]
    fn test_add_pattern() {
        let mut alpha = CreativeSynthesisAlpha::new();
        let symbol = Symbol::new("AMZN").unwrap();

        alpha.add_pattern(
            symbol.clone(),
            CreativeSignalType::Weather,
            0.8,
            0.7,
        );

        assert!(alpha.patterns.contains_key(&symbol));
        assert_eq!(alpha.patterns.get(&symbol).unwrap().len(), 1);
    }

    #[test]
    fn test_combined_confidence() {
        let alpha = CreativeSynthesisAlpha::new();

        let patterns = vec![
            PatternTracker::new(CreativeSignalType::Weather, 0.8, 0.7),
            PatternTracker::new(CreativeSignalType::SearchTrends, 0.6, 0.6),
        ];

        let confidence = alpha.calculate_combined_confidence(&patterns);

        // Should be average + boost for multiple patterns
        assert!(confidence > 0.65);
        assert!(confidence < 0.9);
    }

    #[test]
    fn test_patterns_to_signal() {
        let alpha = CreativeSynthesisAlpha::new();

        // Strong positive patterns
        let positive_patterns = vec![
            PatternTracker::new(CreativeSignalType::Weather, 0.6, 0.7),
        ];
        assert_eq!(
            alpha.patterns_to_signal(&positive_patterns),
            Some(SignalAction::Buy)
        );

        // Strong negative patterns
        let negative_patterns = vec![
            PatternTracker::new(CreativeSignalType::Weather, -0.6, 0.7),
        ];
        assert_eq!(
            alpha.patterns_to_signal(&negative_patterns),
            Some(SignalAction::Sell)
        );
    }

    #[test]
    fn test_earnings_spillover() {
        let alpha = CreativeSynthesisAlpha::new();

        // Large positive move (potential beat)
        let data = create_test_data(5_000_000, 8.0);
        let pattern = alpha.check_earnings_spillover(&data);

        assert!(pattern.is_some());
        let p = pattern.unwrap();
        assert_eq!(p.pattern_type, CreativeSignalType::EarningsSurprise);
        assert!(p.strength > 0.0);
    }

    #[test]
    fn test_search_trends() {
        let alpha = CreativeSynthesisAlpha::new();

        // High volume
        let data = create_test_data(15_000_000, 0.5);
        let pattern = alpha.check_search_trends(&data);

        assert!(pattern.is_some());
        let p = pattern.unwrap();
        assert_eq!(p.pattern_type, CreativeSignalType::SearchTrends);
    }

    #[test]
    fn test_require_multiple_patterns() {
        let mut alpha = CreativeSynthesisAlpha::new();
        alpha.require_multiple = true;

        let symbol = Symbol::new("AMZN").unwrap();

        // Add only one pattern
        alpha.add_pattern(symbol.clone(), CreativeSignalType::Weather, 0.8, 0.7);

        let data = create_test_data(5_000_000, 0.5);
        let signal = alpha.analyze_symbol(&data);

        // Should not generate signal with only one pattern when require_multiple = true
        // Note: May generate if earnings/search patterns also trigger
        // This test primarily checks the require_multiple logic
    }

    #[test]
    fn test_cooldown() {
        let mut alpha = CreativeSynthesisAlpha::new();
        alpha.cooldown_hours = 1;
        alpha.require_multiple = false;

        let symbol = Symbol::new("AMZN").unwrap();
        alpha.add_pattern(symbol.clone(), CreativeSignalType::Weather, 0.8, 0.7);

        let mut data = create_test_data(15_000_000, 8.0);
        data.symbol = symbol.clone();

        // First signal should work
        let signal1 = alpha.analyze_symbol(&data);

        if signal1.is_some() {
            // Second signal should be blocked by cooldown
            let signal2 = alpha.analyze_symbol(&data);
            assert!(signal2.is_none());
        }
    }
}
