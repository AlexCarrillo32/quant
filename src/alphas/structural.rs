//! Structural Inefficiency Alpha - Exploit Known Market Mechanics
//!
//! **Human Insight:**
//! Markets have predictable structural events that create temporary price pressure.
//! Index funds must buy/sell stocks at rebalancing regardless of price. Option market
//! makers hedge gamma exposure near expiry, creating predictable flows. These are
//! mechanical, not sentiment-driven, creating exploitable opportunities.
//!
//! **Strategy: Event-Driven Structural Trades**
//! - Target: Stocks affected by index rebalancing or options expiry
//! - Entry: Ahead of known mechanical buying/selling
//! - Hold time: 1-5 days around the event
//! - Win rate: 65-75% (mechanical flows are very predictable)
//! - Exit: After the event completes or flow reverses
//!
//! We exploit:
//! - Index rebalancing (S&P 500, Russell 2000 reconstitution)
//! - Options expiry (monthly/quarterly OPEX)
//! - Dividend ex-dates (predictable selling pressure)
//! - ETF creation/redemption flows

use super::{AlphaModel, AlphaStats};
use crate::types::*;
use async_trait::async_trait;
use chrono::{DateTime, Datelike, Duration, Utc, Weekday};
use std::collections::HashMap;

/// Types of structural events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StructuralEvent {
    /// Index rebalancing (quarterly)
    IndexRebalance,
    /// Options expiry (monthly OPEX - 3rd Friday)
    OptionsExpiry,
    /// Dividend ex-date
    DividendExDate,
    /// ETF rebalancing
    ETFRebalance,
}

/// Event impact direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventImpact {
    /// Mechanical buying pressure
    BuyingPressure,
    /// Mechanical selling pressure
    SellingPressure,
    /// Neutral or unknown
    Neutral,
}

/// Tracks a specific structural event
#[derive(Debug, Clone)]
struct EventTracker {
    /// Type of event
    event_type: StructuralEvent,
    /// Event date
    event_date: DateTime<Utc>,
    /// Expected impact
    impact: EventImpact,
    /// Confidence in impact (0.0 to 1.0)
    confidence: f64,
    /// Whether event has occurred
    occurred: bool,
}

impl EventTracker {
    fn new(
        event_type: StructuralEvent,
        event_date: DateTime<Utc>,
        impact: EventImpact,
        confidence: f64,
    ) -> Self {
        Self {
            event_type,
            event_date,
            impact,
            confidence,
            occurred: false,
        }
    }

    /// Check if event is upcoming (within window)
    fn is_upcoming(&self, current_time: DateTime<Utc>, days_before: i64) -> bool {
        if self.occurred {
            return false;
        }

        let days_until = (self.event_date - current_time).num_days();
        days_until >= 0 && days_until <= days_before
    }

    /// Check if event just occurred (within window after)
    fn just_occurred(&self, current_time: DateTime<Utc>, days_after: i64) -> bool {
        let days_since = (current_time - self.event_date).num_days();
        days_since >= 0 && days_since <= days_after
    }
}

/// Structural Inefficiency Alpha Model
#[derive(Debug)]
pub struct StructuralInefficiencyAlpha {
    name: String,

    // Track events per symbol
    events: HashMap<Symbol, Vec<EventTracker>>,

    // Configuration
    days_before_event: i64, // How many days before to enter (default: 3)
    days_after_event: i64,  // How many days after to hold (default: 2)
    min_confidence: f64,    // Minimum confidence to trade (default: 0.65)

    // State
    stats: AlphaStats,
    last_signal_time: Option<DateTime<Utc>>,
    cooldown_hours: i64,
}

impl Default for StructuralInefficiencyAlpha {
    fn default() -> Self {
        Self::new()
    }
}

impl StructuralInefficiencyAlpha {
    pub fn new() -> Self {
        Self {
            name: "Structural".to_string(),
            events: HashMap::new(),
            days_before_event: 3,
            days_after_event: 2,
            min_confidence: 0.65,
            stats: AlphaStats::default(),
            last_signal_time: None,
            cooldown_hours: 24,
        }
    }

    /// Add a structural event for a symbol
    pub fn add_event(
        &mut self,
        symbol: Symbol,
        event_type: StructuralEvent,
        event_date: DateTime<Utc>,
        impact: EventImpact,
        confidence: f64,
    ) {
        let tracker = EventTracker::new(event_type, event_date, impact, confidence);
        self.events.entry(symbol).or_default().push(tracker);
    }

    /// Detect if it's options expiry week (3rd Friday of month)
    fn is_opex_week(&self, date: DateTime<Utc>) -> Option<DateTime<Utc>> {
        use chrono::TimeZone;

        // Find 3rd Friday of current month
        let year = date.year();
        let month = date.month();

        // Find first day of month
        let first_day = Utc.with_ymd_and_hms(year, month, 1, 16, 0, 0).single()?;
        let first_friday = match first_day.weekday() {
            Weekday::Fri => first_day,
            Weekday::Sat => first_day + Duration::days(6),
            Weekday::Sun => first_day + Duration::days(5),
            Weekday::Mon => first_day + Duration::days(4),
            Weekday::Tue => first_day + Duration::days(3),
            Weekday::Wed => first_day + Duration::days(2),
            Weekday::Thu => first_day + Duration::days(1),
        };

        // 3rd Friday is 2 weeks after first Friday
        let third_friday = first_friday + Duration::weeks(2);

        // Check if current date is within 5 days of 3rd Friday
        let days_diff = date.signed_duration_since(third_friday).num_days().abs();
        if days_diff <= 5 {
            Some(third_friday)
        } else {
            None
        }
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

    /// Calculate confidence based on event characteristics (for future enhancement)
    #[allow(dead_code)]
    fn calculate_confidence(&self, tracker: &EventTracker, _data: &MarketData) -> f64 {
        let mut confidence = tracker.confidence; // Base confidence from event

        // Index rebalancing is very predictable
        if tracker.event_type == StructuralEvent::IndexRebalance {
            confidence += 0.1;
        }

        // Options expiry for high-volume stocks is predictable
        if tracker.event_type == StructuralEvent::OptionsExpiry {
            confidence += 0.05;
        }

        confidence.min(0.95) // Cap at 95%
    }

    /// Convert event impact to signal action
    fn impact_to_signal(&self, impact: EventImpact, is_before: bool) -> Option<SignalAction> {
        match (impact, is_before) {
            // Before buying pressure event → buy ahead
            (EventImpact::BuyingPressure, true) => Some(SignalAction::Buy),
            // After buying pressure event → sell (pressure over)
            (EventImpact::BuyingPressure, false) => Some(SignalAction::Sell),
            // Before selling pressure event → sell/short ahead
            (EventImpact::SellingPressure, true) => Some(SignalAction::Sell),
            // After selling pressure event → buy (pressure over)
            (EventImpact::SellingPressure, false) => Some(SignalAction::Buy),
            // Neutral events don't generate signals
            (EventImpact::Neutral, _) => None,
        }
    }

    /// Analyze market data for structural signals
    pub fn analyze_symbol(&mut self, data: &MarketData) -> Option<Signal> {
        // Check cooldown
        if self.in_cooldown() {
            return None;
        }

        let now = Utc::now();
        let symbol = data.symbol.clone();

        // Check for options expiry if we don't have explicit events
        if !self.events.contains_key(&symbol) {
            if let Some(opex_date) = self.is_opex_week(now) {
                // For high-volume stocks, OPEX often creates buying pressure
                // (market makers buy stock to hedge short gamma)
                self.add_event(
                    symbol.clone(),
                    StructuralEvent::OptionsExpiry,
                    opex_date,
                    EventImpact::BuyingPressure,
                    0.7,
                );
            }
        }

        // Get events for this symbol
        let events = self.events.get_mut(&symbol)?;

        // Extract config values and clone data before loop
        let days_before = self.days_before_event;
        let days_after = self.days_after_event;
        let min_conf = self.min_confidence;

        // Find upcoming or recent events
        let mut best_signal: Option<(EventTracker, bool, f64)> = None;

        for event in events.iter_mut() {
            // Calculate base confidence from event tracker
            let mut confidence = event.confidence;

            // Index rebalancing is very predictable
            if event.event_type == StructuralEvent::IndexRebalance {
                confidence += 0.1;
            }

            // Options expiry for high-volume stocks is predictable
            if event.event_type == StructuralEvent::OptionsExpiry {
                confidence += 0.05;
            }

            confidence = confidence.min(0.95); // Cap at 95%

            // Check if event is upcoming (within days_before window)
            if event.is_upcoming(now, days_before) {
                if confidence >= min_conf {
                    // Trade BEFORE the event
                    if best_signal.is_none() || confidence > best_signal.as_ref().unwrap().2 {
                        best_signal = Some((event.clone(), true, confidence));
                    }
                }
            }
            // Check if event just occurred (within days_after window)
            else if event.just_occurred(now, days_after) {
                event.occurred = true; // Mark as occurred
                if confidence >= min_conf {
                    // Trade AFTER the event (reversal)
                    if best_signal.is_none() || confidence > best_signal.as_ref().unwrap().2 {
                        best_signal = Some((event.clone(), false, confidence));
                    }
                }
            }
        }

        // Generate signal from best event
        if let Some((event, is_before, confidence)) = best_signal {
            let action = self.impact_to_signal(event.impact, is_before)?;

            // Update state
            self.last_signal_time = Some(now);
            self.stats.signals_generated += 1;
            if confidence >= 0.8 {
                self.stats.signals_actionable += 1;
            }

            // Create reason
            let timing = if is_before { "before" } else { "after" };
            let days_diff = (event.event_date - now).num_days().abs();
            let reason = format!(
                "Structural: {:?} {} ({} days {})",
                event.event_type,
                match event.impact {
                    EventImpact::BuyingPressure => "buying pressure",
                    EventImpact::SellingPressure => "selling pressure",
                    EventImpact::Neutral => "neutral",
                },
                days_diff,
                timing
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
                "event_type": format!("{:?}", event.event_type),
                "event_date": event.event_date.to_rfc3339(),
                "impact": format!("{:?}", event.impact),
                "is_before_event": is_before,
                "days_until_event": days_diff,
            }));

            Some(signal)
        } else {
            None
        }
    }
}

#[async_trait]
impl AlphaModel for StructuralInefficiencyAlpha {
    fn name(&self) -> &str {
        &self.name
    }

    fn human_insight(&self) -> &str {
        "Markets have predictable mechanical events. Index funds must buy/sell at \
         rebalancing regardless of price. Option market makers hedge gamma near expiry, \
         creating flows. Dividends create predictable selling. These structural forces \
         are not sentiment-driven - they're mechanical obligations that create temporary \
         price pressure we can exploit by trading ahead of the event."
    }

    fn update(&mut self, _data: &MarketSnapshot) {
        // State update happens in analyze_symbol
    }

    async fn generate_signals(&self) -> Vec<Signal> {
        // This is a stub - in production, would fetch event calendars here
        Vec::new()
    }

    fn reset(&mut self) {
        self.stats = AlphaStats::default();
        self.last_signal_time = None;
        self.events.clear();
    }

    fn stats(&self) -> AlphaStats {
        self.stats.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data() -> MarketData {
        let symbol = Symbol::new("SPY").unwrap();
        let price = Price::new(450.0).unwrap();

        MarketData {
            symbol,
            quote: Quote {
                bid: price,
                ask: price,
                bid_size: Quantity::buy(1000),
                ask_size: Quantity::sell(1000),
                timestamp: Utc::now(),
            },
            last_price: price,
            volume: 10_000_000,
            timestamp: Utc::now(),
            open: Some(price),
            high: Some(price),
            low: Some(price),
            prev_close: Some(price),
            vix: Some(15.0),
            put_call_ratio: Some(1.0),
        }
    }

    #[test]
    fn test_event_upcoming() {
        let now = Utc::now();
        let event_date = now + Duration::days(2);
        let tracker = EventTracker::new(
            StructuralEvent::IndexRebalance,
            event_date,
            EventImpact::BuyingPressure,
            0.8,
        );

        assert!(tracker.is_upcoming(now, 3)); // Within 3 days
        assert!(!tracker.is_upcoming(now, 1)); // Not within 1 day
    }

    #[test]
    fn test_event_just_occurred() {
        let now = Utc::now();
        let event_date = now - Duration::days(1);
        let tracker = EventTracker::new(
            StructuralEvent::OptionsExpiry,
            event_date,
            EventImpact::BuyingPressure,
            0.7,
        );

        assert!(tracker.just_occurred(now, 2)); // Within 2 days after
        assert!(!tracker.just_occurred(now - Duration::days(3), 2)); // Too early
    }

    #[test]
    fn test_impact_to_signal_before() {
        let alpha = StructuralInefficiencyAlpha::new();

        // Before buying pressure → buy
        assert_eq!(
            alpha.impact_to_signal(EventImpact::BuyingPressure, true),
            Some(SignalAction::Buy)
        );

        // Before selling pressure → sell
        assert_eq!(
            alpha.impact_to_signal(EventImpact::SellingPressure, true),
            Some(SignalAction::Sell)
        );
    }

    #[test]
    fn test_impact_to_signal_after() {
        let alpha = StructuralInefficiencyAlpha::new();

        // After buying pressure → sell (reverse)
        assert_eq!(
            alpha.impact_to_signal(EventImpact::BuyingPressure, false),
            Some(SignalAction::Sell)
        );

        // After selling pressure → buy (reverse)
        assert_eq!(
            alpha.impact_to_signal(EventImpact::SellingPressure, false),
            Some(SignalAction::Buy)
        );
    }

    #[test]
    fn test_add_event() {
        let mut alpha = StructuralInefficiencyAlpha::new();
        let symbol = Symbol::new("AAPL").unwrap();
        let event_date = Utc::now() + Duration::days(5);

        alpha.add_event(
            symbol.clone(),
            StructuralEvent::IndexRebalance,
            event_date,
            EventImpact::BuyingPressure,
            0.85,
        );

        assert!(alpha.events.contains_key(&symbol));
        assert_eq!(alpha.events.get(&symbol).unwrap().len(), 1);
    }

    #[test]
    fn test_signal_before_event() {
        let mut alpha = StructuralInefficiencyAlpha::new();
        let symbol = Symbol::new("TSLA").unwrap();
        let event_date = Utc::now() + Duration::days(2);

        // Add upcoming rebalancing event
        alpha.add_event(
            symbol.clone(),
            StructuralEvent::IndexRebalance,
            event_date,
            EventImpact::BuyingPressure,
            0.85,
        );

        let mut data = create_test_data();
        data.symbol = symbol;

        let signal = alpha.analyze_symbol(&data);

        // Should generate buy signal before buying pressure event
        assert!(signal.is_some());
        let sig = signal.unwrap();
        assert_eq!(sig.action, SignalAction::Buy);
        assert!(sig.confidence.value() >= 0.8);
        assert!(sig.reason.contains("Structural"));
    }

    #[test]
    fn test_signal_after_event() {
        let mut alpha = StructuralInefficiencyAlpha::new();
        let symbol = Symbol::new("NVDA").unwrap();
        let event_date = Utc::now() - Duration::days(1);

        // Add recent event
        alpha.add_event(
            symbol.clone(),
            StructuralEvent::IndexRebalance,
            event_date,
            EventImpact::BuyingPressure,
            0.8,
        );

        let mut data = create_test_data();
        data.symbol = symbol;

        let signal = alpha.analyze_symbol(&data);

        // Should generate sell signal after buying pressure event (reversal)
        assert!(signal.is_some());
        let sig = signal.unwrap();
        assert_eq!(sig.action, SignalAction::Sell);
    }

    #[test]
    fn test_cooldown() {
        let mut alpha = StructuralInefficiencyAlpha::new();
        alpha.cooldown_hours = 1;

        let symbol = Symbol::new("SPY").unwrap();
        let event_date = Utc::now() + Duration::days(2);

        alpha.add_event(
            symbol.clone(),
            StructuralEvent::IndexRebalance,
            event_date,
            EventImpact::BuyingPressure,
            0.85,
        );

        let mut data = create_test_data();
        data.symbol = symbol.clone();

        // First signal should work
        let signal1 = alpha.analyze_symbol(&data);
        assert!(signal1.is_some());

        // Second signal should be blocked by cooldown
        let signal2 = alpha.analyze_symbol(&data);
        assert!(signal2.is_none());
    }
}
