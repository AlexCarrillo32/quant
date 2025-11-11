#!/usr/bin/env python3
"""
Behavioral Finance Strategy Example

Demonstrates the "Human Edge" - modeling human psychology and irrationality
to find trading opportunities that pure math models miss.

This strategy combines multiple behavioral signals:
1. Panic detection (fear spikes)
2. FOMO detection (greed peaks)
3. Narrative shifts (story changes)
"""

from typing import List, Optional, Dict, Tuple
from dataclasses import dataclass
from enum import Enum
import quant_engine as qe


class MarketEmotion(Enum):
    """Market emotional state"""

    EXTREME_FEAR = -2
    FEAR = -1
    NEUTRAL = 0
    GREED = 1
    EXTREME_GREED = 2


@dataclass
class MarketContext:
    """
    Market context data for behavioral analysis

    In production, this would come from:
    - VIX data (CBOE API)
    - Social media sentiment (Twitter/Reddit APIs)
    - News sentiment (NLP on financial news)
    - Put/call ratios (options data)
    """

    vix: float  # Volatility index
    twitter_sentiment: float  # -1.0 (fear) to +1.0 (greed)
    reddit_mentions: int  # Social media activity
    put_call_ratio: float  # Options positioning
    news_sentiment: float  # News sentiment score


class BehavioralStrategy:
    """
    The Human Edge Strategy

    This strategy exploits human psychological biases:
    - Humans panic sell at bottoms → BUY
    - Humans FOMO buy at tops → SELL
    - Humans are slow to react to narrative changes → EARLY POSITIONING
    """

    def __init__(
        self,
        panic_vix_threshold: float = 30.0,
        fomo_mention_threshold: int = 10000,
        high_confidence: float = 0.85,
        medium_confidence: float = 0.70,
    ):
        self.panic_vix_threshold = panic_vix_threshold
        self.fomo_mention_threshold = fomo_mention_threshold
        self.high_confidence = high_confidence
        self.medium_confidence = medium_confidence

    def detect_emotion(self, context: MarketContext) -> MarketEmotion:
        """
        Detect market emotional state from context

        Human Insight:
        When VIX spikes + sentiment is negative + put/call ratio is high,
        humans are PANICKING. This creates buying opportunities.
        """
        fear_score = 0
        greed_score = 0

        # VIX analysis
        if context.vix > self.panic_vix_threshold:
            fear_score += 2
        elif context.vix > 20:
            fear_score += 1

        if context.vix < 15:
            greed_score += 1

        # Sentiment analysis
        if context.twitter_sentiment < -0.5:
            fear_score += 2
        elif context.twitter_sentiment < -0.2:
            fear_score += 1
        elif context.twitter_sentiment > 0.5:
            greed_score += 2
        elif context.twitter_sentiment > 0.2:
            greed_score += 1

        # Put/call ratio (high = fear, low = greed)
        if context.put_call_ratio > 1.5:
            fear_score += 1
        elif context.put_call_ratio < 0.7:
            greed_score += 1

        # Social media activity (high mentions = FOMO)
        if context.reddit_mentions > self.fomo_mention_threshold:
            greed_score += 2

        # Classify emotion
        net_score = greed_score - fear_score

        if net_score >= 4:
            return MarketEmotion.EXTREME_GREED
        elif net_score >= 2:
            return MarketEmotion.GREED
        elif net_score <= -4:
            return MarketEmotion.EXTREME_FEAR
        elif net_score <= -2:
            return MarketEmotion.FEAR
        else:
            return MarketEmotion.NEUTRAL

    def analyze_panic(
        self, symbol: str, current_price: float, context: MarketContext
    ) -> Optional[qe.Signal]:
        """
        Detect panic selling opportunities

        Human Insight:
        "Be fearful when others are greedy, and greedy when others are fearful"
        - Warren Buffett

        When humans panic, they sell indiscriminately. This creates
        opportunities to buy quality assets at discounted prices.
        """
        emotion = self.detect_emotion(context)

        if emotion in [MarketEmotion.EXTREME_FEAR, MarketEmotion.FEAR]:
            confidence = (
                self.high_confidence
                if emotion == MarketEmotion.EXTREME_FEAR
                else self.medium_confidence
            )

            sym = qe.Symbol(symbol)
            action = qe.SignalAction.Buy

            signal = qe.Signal.new(
                sym,
                action,
                qe.Confidence(confidence),
                f"PANIC DETECTED: VIX={context.vix:.1f}, "
                f"Sentiment={context.twitter_sentiment:.2f}, "
                f"P/C={context.put_call_ratio:.2f} - Buy the fear",
                "BehavioralPanic",
            )

            # Conservative targets during panic
            signal.with_target_price(qe.Price(current_price * 1.05))  # +5% target
            signal.with_stop_loss(qe.Price(current_price * 0.97))  # -3% stop

            return signal

        return None

    def analyze_fomo(
        self, symbol: str, current_price: float, context: MarketContext
    ) -> Optional[qe.Signal]:
        """
        Detect FOMO (Fear Of Missing Out) peaks

        Human Insight:
        When everyone is talking about a stock on social media,
        when sentiment is extremely positive, when VIX is low,
        retail traders are FOMOing in at the TOP. This is a sell signal.

        Examples: GME peak, crypto tops, meme stocks
        """
        emotion = self.detect_emotion(context)

        if emotion in [MarketEmotion.EXTREME_GREED, MarketEmotion.GREED]:
            # Extra check: high social media mentions = FOMO
            if context.reddit_mentions > self.fomo_mention_threshold:
                confidence = (
                    self.high_confidence
                    if emotion == MarketEmotion.EXTREME_GREED
                    else self.medium_confidence
                )

                sym = qe.Symbol(symbol)
                action = qe.SignalAction.Sell

                signal = qe.Signal.new(
                    sym,
                    action,
                    qe.Confidence(confidence),
                    f"FOMO PEAK DETECTED: Mentions={context.reddit_mentions}, "
                    f"Sentiment={context.twitter_sentiment:.2f}, "
                    f"VIX={context.vix:.1f} - Sell into euphoria",
                    "BehavioralFOMO",
                )

                # Tight stops (FOMO can persist longer than expected)
                signal.with_target_price(qe.Price(current_price * 0.90))  # -10% target
                signal.with_stop_loss(qe.Price(current_price * 1.03))  # +3% stop

                return signal

        return None

    def analyze_narrative_shift(
        self,
        symbol: str,
        current_price: float,
        old_narrative: str,
        new_narrative: str,
        news_confidence: float,
    ) -> Optional[qe.Signal]:
        """
        Detect narrative shifts before consensus

        Human Insight:
        Markets move on stories, not just numbers. When the narrative
        changes (e.g., "inflation is transitory" → "inflation is persistent"),
        humans are slow to react. Early detection = edge.

        Examples:
        - Fed pivot signals
        - Industry disruption news
        - Regulatory changes
        """
        # In production, this would use NLP to detect narrative changes
        # For now, we simulate based on news_confidence

        if news_confidence > 0.7:  # Strong narrative shift
            # Determine direction based on narrative keywords
            # (In production, use sentiment analysis)
            bullish_keywords = ["beat", "strong", "growth", "positive"]
            bearish_keywords = ["miss", "weak", "decline", "negative"]

            is_bullish = any(kw in new_narrative.lower() for kw in bullish_keywords)
            is_bearish = any(kw in new_narrative.lower() for kw in bearish_keywords)

            if is_bullish or is_bearish:
                sym = qe.Symbol(symbol)
                action = qe.SignalAction.Buy if is_bullish else qe.SignalAction.Sell

                signal = qe.Signal.new(
                    sym,
                    action,
                    qe.Confidence(news_confidence),
                    f"NARRATIVE SHIFT: '{old_narrative}' → '{new_narrative}'",
                    "BehavioralNarrative",
                )

                # Wider targets for narrative trades (can run longer)
                if is_bullish:
                    signal.with_target_price(qe.Price(current_price * 1.10))
                    signal.with_stop_loss(qe.Price(current_price * 0.97))
                else:
                    signal.with_target_price(qe.Price(current_price * 0.90))
                    signal.with_stop_loss(qe.Price(current_price * 1.03))

                return signal

        return None


def main():
    """Example usage of behavioral strategy"""
    strategy = BehavioralStrategy()

    print("=" * 70)
    print("BEHAVIORAL FINANCE STRATEGY - THE HUMAN EDGE")
    print("=" * 70)
    print()

    # Example 1: Market panic (COVID-like crash)
    print("Example 1: MARKET PANIC (COVID-style crash)")
    panic_context = MarketContext(
        vix=65.0,  # Extreme fear
        twitter_sentiment=-0.85,  # Very negative
        reddit_mentions=5000,  # Low mentions (fear)
        put_call_ratio=2.3,  # Heavy put buying
        news_sentiment=-0.7,
    )

    signal = strategy.analyze_panic("SPY", 380.0, panic_context)
    if signal:
        print(f"  Signal: {signal}")
        print(f"  Emotion: {strategy.detect_emotion(panic_context).name}")
        print(f"  Trade: BUY THE PANIC - humans are irrationally fearful")
    print()

    # Example 2: FOMO peak (GME-style meme stock)
    print("Example 2: FOMO PEAK (Meme stock euphoria)")
    fomo_context = MarketContext(
        vix=12.0,  # Low fear
        twitter_sentiment=0.92,  # Extreme euphoria
        reddit_mentions=50000,  # Massive social media activity
        put_call_ratio=0.45,  # Everyone buying calls
        news_sentiment=0.8,
    )

    signal = strategy.analyze_fomo("GME", 350.0, fomo_context)
    if signal:
        print(f"  Signal: {signal}")
        print(f"  Emotion: {strategy.detect_emotion(fomo_context).name}")
        print(f"  Trade: SELL INTO FOMO - retail is buying the top")
    print()

    # Example 3: Narrative shift (Fed pivot)
    print("Example 3: NARRATIVE SHIFT (Fed policy change)")
    signal = strategy.analyze_narrative_shift(
        "TLT",
        95.0,
        "Fed will keep hiking rates aggressively",
        "Fed signals pause in rate hikes, growth concerns",
        news_confidence=0.85,
    )
    if signal:
        print(f"  Signal: {signal}")
        print(f"  Trade: EARLY POSITIONING on narrative change")
    print()

    # Example 4: Normal market (no signal)
    print("Example 4: NORMAL MARKET (No behavioral edge)")
    normal_context = MarketContext(
        vix=16.0,  # Normal
        twitter_sentiment=0.1,  # Neutral
        reddit_mentions=3000,  # Normal
        put_call_ratio=1.0,  # Balanced
        news_sentiment=0.0,
    )

    signal = strategy.analyze_panic("AAPL", 180.0, normal_context)
    if signal:
        print(f"  Signal: {signal}")
    else:
        print(f"  No signal - market is rational (no edge)")
        print(f"  Emotion: {strategy.detect_emotion(normal_context).name}")

    print()
    print("=" * 70)
    print("THE HUMAN EDGE: We win when markets are IRRATIONAL")
    print("=" * 70)


if __name__ == "__main__":
    main()
