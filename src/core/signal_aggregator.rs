//! Signal Aggregation
//!
//! Combines signals from multiple alpha models using various strategies

use crate::types::{Confidence, Signal, SignalAction, Symbol};
use std::collections::HashMap;

/// Strategy for combining signals from multiple alphas
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregationStrategy {
    /// Take the highest confidence signal
    HighestConfidence,

    /// Average confidence across all signals with same action
    WeightedAverage,

    /// Require unanimous agreement (all alphas agree)
    Unanimous,

    /// Majority vote (most alphas agree)
    MajorityVote,
}

/// Aggregates signals from multiple alpha models
#[derive(Debug)]
pub struct SignalAggregator {
    strategy: AggregationStrategy,
    min_confidence: f64,
}

impl SignalAggregator {
    /// Create new aggregator with strategy
    pub fn new(strategy: AggregationStrategy) -> Self {
        SignalAggregator {
            strategy,
            min_confidence: 0.5, // Default 50% minimum
        }
    }

    /// Set minimum confidence threshold
    pub fn with_min_confidence(mut self, min_confidence: f64) -> Self {
        self.min_confidence = min_confidence;
        self
    }

    /// Aggregate signals by symbol
    ///
    /// Takes a flat list of signals and groups them by symbol,
    /// then applies the aggregation strategy to each group.
    pub fn aggregate(&self, signals: Vec<Signal>) -> Vec<Signal> {
        // Group signals by symbol
        let mut by_symbol: HashMap<Symbol, Vec<Signal>> = HashMap::new();

        for signal in signals {
            by_symbol
                .entry(signal.symbol.clone())
                .or_insert_with(Vec::new)
                .push(signal);
        }

        // Aggregate each symbol's signals
        by_symbol
            .into_iter()
            .filter_map(|(_, symbol_signals)| self.aggregate_symbol_signals(symbol_signals))
            .collect()
    }

    /// Aggregate signals for a single symbol
    fn aggregate_symbol_signals(&self, signals: Vec<Signal>) -> Option<Signal> {
        if signals.is_empty() {
            return None;
        }

        match self.strategy {
            AggregationStrategy::HighestConfidence => self.highest_confidence(signals),
            AggregationStrategy::WeightedAverage => self.weighted_average(signals),
            AggregationStrategy::Unanimous => self.unanimous(signals),
            AggregationStrategy::MajorityVote => self.majority_vote(signals),
        }
    }

    /// Take signal with highest confidence
    fn highest_confidence(&self, mut signals: Vec<Signal>) -> Option<Signal> {
        signals.sort_by(|a, b| {
            b.confidence
                .value()
                .partial_cmp(&a.confidence.value())
                .unwrap()
        });

        signals
            .into_iter()
            .find(|s| s.confidence.value() >= self.min_confidence)
    }

    /// Average confidence across signals with same action
    fn weighted_average(&self, signals: Vec<Signal>) -> Option<Signal> {
        // Group by action
        let mut by_action: HashMap<SignalAction, Vec<Signal>> = HashMap::new();

        for signal in signals {
            by_action
                .entry(signal.action)
                .or_insert_with(Vec::new)
                .push(signal);
        }

        // Find action with highest average confidence
        let mut best: Option<(SignalAction, f64, Vec<Signal>)> = None;

        for (action, action_signals) in by_action {
            let avg_confidence = action_signals
                .iter()
                .map(|s| s.confidence.value())
                .sum::<f64>()
                / action_signals.len() as f64;

            if avg_confidence >= self.min_confidence {
                if best.is_none() || avg_confidence > best.as_ref().unwrap().1 {
                    best = Some((action, avg_confidence, action_signals));
                }
            }
        }

        // Create aggregated signal
        best.map(|(action, avg_confidence, action_signals)| {
            let mut base_signal = action_signals[0].clone();
            base_signal.confidence = Confidence::new(avg_confidence).unwrap();
            base_signal.reason = format!(
                "Aggregated from {} alphas (avg confidence: {:.1}%)",
                action_signals.len(),
                avg_confidence * 100.0
            );
            base_signal
        })
    }

    /// Require all alphas to agree on action
    fn unanimous(&self, signals: Vec<Signal>) -> Option<Signal> {
        if signals.is_empty() {
            return None;
        }

        // Check if all actions are the same
        let first_action = signals[0].action;
        let all_agree = signals.iter().all(|s| s.action == first_action);

        if !all_agree {
            return None;
        }

        // Average the confidence
        let avg_confidence =
            signals.iter().map(|s| s.confidence.value()).sum::<f64>() / signals.len() as f64;

        if avg_confidence < self.min_confidence {
            return None;
        }

        let mut base_signal = signals[0].clone();
        base_signal.confidence = Confidence::new(avg_confidence).unwrap();
        base_signal.reason = format!(
            "Unanimous agreement from {} alphas (confidence: {:.1}%)",
            signals.len(),
            avg_confidence * 100.0
        );

        Some(base_signal)
    }

    /// Take action with most votes
    fn majority_vote(&self, signals: Vec<Signal>) -> Option<Signal> {
        // Count votes by action
        let mut votes: HashMap<SignalAction, Vec<Signal>> = HashMap::new();

        for signal in signals {
            votes
                .entry(signal.action)
                .or_insert_with(Vec::new)
                .push(signal);
        }

        // Find action with most votes
        let majority = votes
            .into_iter()
            .max_by_key(|(_, action_signals)| action_signals.len());

        majority.and_then(|(action, action_signals)| {
            let total_signals = action_signals.len();
            let avg_confidence = action_signals
                .iter()
                .map(|s| s.confidence.value())
                .sum::<f64>()
                / total_signals as f64;

            if avg_confidence < self.min_confidence {
                return None;
            }

            let mut base_signal = action_signals[0].clone();
            base_signal.confidence = Confidence::new(avg_confidence).unwrap();
            base_signal.reason = format!(
                "Majority vote: {} of {} alphas agree (confidence: {:.1}%)",
                total_signals,
                action_signals.len(),
                avg_confidence * 100.0
            );

            Some(base_signal)
        })
    }
}

impl Default for SignalAggregator {
    fn default() -> Self {
        SignalAggregator::new(AggregationStrategy::HighestConfidence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Price, Symbol};

    fn create_signal(symbol: &str, action: SignalAction, confidence: f64) -> Signal {
        Signal::new(
            Symbol::new(symbol).unwrap(),
            action,
            Confidence::new(confidence).unwrap(),
            "Test signal",
            "TestAlpha",
        )
        .with_target_price(Price::new(100.0).unwrap())
    }

    #[test]
    fn test_highest_confidence() {
        let aggregator = SignalAggregator::new(AggregationStrategy::HighestConfidence);

        let signals = vec![
            create_signal("AAPL", SignalAction::Buy, 0.6),
            create_signal("AAPL", SignalAction::Buy, 0.9),
            create_signal("AAPL", SignalAction::Sell, 0.7),
        ];

        let result = aggregator.aggregate(signals);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].confidence.value(), 0.9);
        assert_eq!(result[0].action, SignalAction::Buy);
    }

    #[test]
    fn test_weighted_average() {
        let aggregator = SignalAggregator::new(AggregationStrategy::WeightedAverage);

        let signals = vec![
            create_signal("AAPL", SignalAction::Buy, 0.8),
            create_signal("AAPL", SignalAction::Buy, 0.6),
            create_signal("AAPL", SignalAction::Sell, 0.5),
        ];

        let result = aggregator.aggregate(signals);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].action, SignalAction::Buy);
        // Average: (0.8 + 0.6) / 2 = 0.7
        assert!((result[0].confidence.value() - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_unanimous_agreement() {
        let aggregator = SignalAggregator::new(AggregationStrategy::Unanimous);

        let signals = vec![
            create_signal("AAPL", SignalAction::Buy, 0.8),
            create_signal("AAPL", SignalAction::Buy, 0.7),
            create_signal("AAPL", SignalAction::Buy, 0.9),
        ];

        let result = aggregator.aggregate(signals);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].action, SignalAction::Buy);
    }

    #[test]
    fn test_unanimous_disagreement() {
        let aggregator = SignalAggregator::new(AggregationStrategy::Unanimous);

        let signals = vec![
            create_signal("AAPL", SignalAction::Buy, 0.8),
            create_signal("AAPL", SignalAction::Sell, 0.7),
        ];

        let result = aggregator.aggregate(signals);
        assert_eq!(result.len(), 0); // No unanimous agreement
    }

    #[test]
    fn test_majority_vote() {
        let aggregator = SignalAggregator::new(AggregationStrategy::MajorityVote);

        let signals = vec![
            create_signal("AAPL", SignalAction::Buy, 0.8),
            create_signal("AAPL", SignalAction::Buy, 0.7),
            create_signal("AAPL", SignalAction::Sell, 0.6),
        ];

        let result = aggregator.aggregate(signals);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].action, SignalAction::Buy); // 2 vs 1
    }

    #[test]
    fn test_min_confidence_filter() {
        let aggregator =
            SignalAggregator::new(AggregationStrategy::HighestConfidence).with_min_confidence(0.8);

        let signals = vec![
            create_signal("AAPL", SignalAction::Buy, 0.6),
            create_signal("AAPL", SignalAction::Sell, 0.7),
        ];

        let result = aggregator.aggregate(signals);
        assert_eq!(result.len(), 0); // All below threshold
    }

    #[test]
    fn test_multiple_symbols() {
        let aggregator = SignalAggregator::new(AggregationStrategy::HighestConfidence);

        let signals = vec![
            create_signal("AAPL", SignalAction::Buy, 0.9),
            create_signal("GOOGL", SignalAction::Sell, 0.8),
            create_signal("AAPL", SignalAction::Buy, 0.7),
        ];

        let result = aggregator.aggregate(signals);
        assert_eq!(result.len(), 2); // One for AAPL, one for GOOGL
    }
}
