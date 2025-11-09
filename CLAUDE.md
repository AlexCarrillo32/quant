# The Human Edge Engine - Development Guidelines

## Purpose

These rules ensure we maintain **ultra-low latency performance** while building **human-intuition alpha models** that banks can't replicate.

---

## 0 — Core Philosophy

**The Human Edge** = Speed (Rust) + Intelligence (Human Psychology) + Creativity (Only Humans Can Build)

- **Speed**: Every microsecond counts when competing without colocation
- **Intelligence**: Model human behavior, not just math
- **Creativity**: Build strategies banks can't/won't build

---

## 1 — Performance Rules (MUST)

### Hot Path Performance

- **P-1 (MUST)** Hot path code must NEVER allocate memory
- **P-2 (MUST)** Use pre-allocated arenas for all trading data structures
- **P-3 (MUST)** Use lock-free data structures (crossbeam, lockfree crates)
- **P-4 (MUST)** Pin critical threads to dedicated CPU cores
- **P-5 (MUST)** Use `#[inline(always)]` for hot path functions
- **P-6 (MUST)** Avoid branches in hot path (use branchless techniques)
- **P-7 (MUST)** Never use `println!` in hot path (use async logger)

### Performance Testing

- **P-8 (MUST)** Benchmark all hot path functions with criterion
- **P-9 (MUST)** Profile with `perf` and flamegraph before optimizing
- **P-10 (MUST)** Measure latency in nanoseconds, not milliseconds
- **P-11 (MUST)** Test on target hardware (not just dev machine)

### Code Review Performance Checklist

Before merging hot path code, verify:
```rust
// ❌ BAD - Allocates on hot path
let orders = Vec::new();
orders.push(order);

// ✅ GOOD - Pre-allocated arena
let order_slot = ORDER_ARENA.alloc()?;
*order_slot = order;
```

---

## 2 — Rust Best Practices

### Memory Safety

- **R-1 (MUST)** Use `unsafe` ONLY when necessary and document why
- **R-2 (MUST)** All `unsafe` blocks must have safety comments
- **R-3 (SHOULD)** Use `#[repr(C)]` for structs passed to C++
- **R-4 (MUST)** Use `Pin` for self-referential structs

### Error Handling

- **R-5 (MUST)** Use `Result<T, E>` for fallible operations
- **R-6 (MUST)** Use `anyhow::Result` for application errors
- **R-7 (MUST)** Use `thiserror` for library errors
- **R-8 (MUST NOT)** Use `unwrap()` or `expect()` in hot path
- **R-9 (SHOULD)** Log errors before propagating

### Type Safety

- **R-10 (MUST)** Use newtype pattern for domain types
- **R-11 (SHOULD)** Use enums instead of booleans for clarity
- **R-12 (MUST)** Use `NonZeroU*` types when appropriate

Example:
```rust
// ❌ BAD
fn calculate_price(price: f64, quantity: u32) -> f64 { ... }

// ✅ GOOD
#[derive(Debug, Clone, Copy)]
struct Price(f64);

#[derive(Debug, Clone, Copy)]
struct Quantity(u32);

fn calculate_price(price: Price, quantity: Quantity) -> Price { ... }
```

---

## 3 — Alpha Model Development

### Human Psychology First

- **A-1 (MUST)** Every alpha model must have a **human insight** documented
- **A-2 (SHOULD)** Model WHY humans behave irrationally, not just THAT they do
- **A-3 (MUST)** Include confidence scores (0-1) for all signals
- **A-4 (SHOULD)** Explain signals in human terms, not just math

Example:
```rust
pub struct PanicDetectorAlpha {
    // ✅ GOOD - Documents the human insight
    /// Human Insight: When VIX spikes + Twitter sentiment is negative +
    /// Fed is hawkish, humans PANIC SELL irrationally.
    /// This creates buying opportunities.
    panic_threshold: f64,
}

impl PanicDetectorAlpha {
    pub fn detect(&self, market: &MarketData) -> Option<Signal> {
        // Document the reasoning
        if market.vix > 30.0 && market.twitter_fear > 0.7 {
            Some(Signal {
                action: Action::Buy,
                confidence: 0.85,
                reason: "Humans panicking - buy the dip", // ✅ Human explanation
            })
        } else {
            None
        }
    }
}
```

### Narrative-Driven Models

- **A-5 (MUST)** Track market narratives (Fed speak, earnings calls, news)
- **A-6 (SHOULD)** Detect narrative shifts, not just price movements
- **A-7 (SHOULD)** Use sentiment analysis on text data
- **A-8 (MUST)** Document which narratives drive which trades

### Behavioral Finance Integration

- **A-9 (SHOULD)** Reference behavioral finance research papers
- **A-10 (SHOULD)** Model cognitive biases: anchoring, recency, FOMO, panic
- **A-11 (MUST)** Test strategies against behavioral patterns, not just price

---

## 4 — Testing Strategy

### Unit Tests

- **T-1 (MUST)** Test all alpha models with realistic market scenarios
- **T-2 (MUST)** Test edge cases: panics, narratives shifts, meme stocks
- **T-3 (SHOULD)** Use property-based testing (proptest) for invariants

### Integration Tests

- **T-4 (MUST)** Test full signal generation pipeline
- **T-5 (MUST)** Test with historical market data
- **T-6 (SHOULD)** Simulate behavioral scenarios (e.g., "COVID crash")

### Performance Tests

- **T-7 (MUST)** Benchmark hot path functions with criterion
- **T-8 (MUST)** Test under load (1000+ symbols, 1M+ ticks/sec)
- **T-9 (SHOULD)** Profile memory usage and cache misses

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panic_detector_covid_crash() {
        // ✅ GOOD - Tests realistic behavioral scenario
        let market = MarketData {
            vix: 82.0,  // COVID crash levels
            twitter_sentiment: -0.9,  // Extreme fear
            put_call_ratio: 2.5,  // Everyone buying puts
        };

        let signal = panic_detector.detect(&market);
        assert!(signal.is_some());
        assert_eq!(signal.unwrap().action, Action::Buy);
    }
}
```

---

## 5 — Code Organization

### Module Structure

```
src/
├── main.rs              # Entry point
├── lib.rs               # Library exports
├── core/                # Trading engine core
│   ├── mod.rs
│   ├── engine.rs        # Main engine
│   ├── cpu_pinning.rs   # CPU affinity
│   └── memory.rs        # Pre-allocated arenas
├── network/             # Networking layer
│   ├── mod.rs
│   └── dpdk.rs          # DPDK bindings (future)
├── alphas/              # Alpha models
│   ├── mod.rs
│   ├── base.rs          # Base alpha trait
│   ├── panic_detector.rs
│   ├── narrative_shift.rs
│   ├── crowd_behavior.rs
│   ├── structural.rs
│   └── creative.rs
├── indicators/          # Technical indicators
│   ├── mod.rs
│   ├── ema.rs
│   ├── macd.rs
│   └── simd.cpp         # C++ SIMD kernels
├── human_layer/         # Python interface
│   ├── mod.rs
│   └── bindings.rs
└── types/               # Domain types
    ├── mod.rs
    ├── price.rs
    ├── signal.rs
    └── market_data.rs
```

### File Organization

- **O-1 (MUST)** One public struct/enum per file (exceptions for small helpers)
- **O-2 (MUST)** Group related functionality in modules
- **O-3 (SHOULD)** Keep files under 500 lines (refactor if larger)

---

## 6 — Git & CI/CD

### Commit Messages

- **G-1 (MUST)** Use Conventional Commits format
- **G-2 (SHOULD NOT)** Mention Claude or Anthropic
- **G-3 (SHOULD)** Explain WHY, not just WHAT

Example:
```
perf(core): reduce signal generation latency by 40%

Use pre-allocated arena instead of Vec for order storage.
Benchmarks show 40μs → 24μs improvement on 1000 orders.

Related: #123
```

### CI Pipeline

- **G-4 (MUST)** `cargo clippy` must pass with no warnings
- **G-5 (MUST)** `cargo test` must pass
- **G-6 (MUST)** `cargo bench` must not regress >10%
- **G-7 (SHOULD)** Run `cargo audit` for security

### Pre-commit Checklist

```bash
# Format
cargo fmt

# Lint
cargo clippy -- -D warnings

# Test
cargo test

# Benchmark (if hot path changed)
cargo bench

# Security audit
cargo audit

# Build release
cargo build --release
```

---

## 7 — Documentation

### Code Documentation

- **D-1 (MUST)** Document all public APIs with `///` doc comments
- **D-2 (MUST)** Include examples in doc comments
- **D-3 (SHOULD)** Document performance characteristics
- **D-4 (MUST)** Document human insights for alpha models

Example:
```rust
/// Detects panic selling opportunities using behavioral finance.
///
/// # Human Insight
///
/// When VIX spikes + social sentiment is negative + Fed is hawkish,
/// retail traders panic sell. This creates buying opportunities as
/// prices overshoot fundamentals.
///
/// # Performance
///
/// - Average latency: 120ns
/// - Memory: Zero allocation (uses pre-allocated buffers)
///
/// # Example
///
/// ```
/// let detector = PanicDetectorAlpha::new();
/// let signal = detector.detect(&market_data);
/// ```
pub struct PanicDetectorAlpha { ... }
```

### Project Documentation

- **D-5 (MUST)** Keep README.md up to date
- **D-6 (SHOULD)** Update architecture docs when making structural changes
- **D-7 (MUST)** Document performance benchmarks

---

## 8 — Performance Optimization Guidelines

### When to Optimize

- **O-1 (MUST)** Profile BEFORE optimizing (use `perf`, `flamegraph`)
- **O-2 (SHOULD)** Optimize hot path first (80/20 rule)
- **O-3 (MUST NOT)** Optimize prematurely

### Optimization Techniques

- **O-4 (SHOULD)** Use SIMD for vector operations (via C++ or Rust intrinsics)
- **O-5 (SHOULD)** Use lookup tables instead of calculations
- **O-6 (SHOULD)** Unroll loops in hot path
- **O-7 (SHOULD)** Use `likely`/`unlikely` hints for branches

Example:
```rust
// ❌ BAD - Slow
for i in 0..prices.len() {
    result[i] = calculate_expensive(prices[i]);
}

// ✅ GOOD - Fast (SIMD via C++)
unsafe {
    calculate_expensive_simd(
        prices.as_ptr(),
        result.as_mut_ptr(),
        prices.len(),
    );
}
```

---

## 9 — Security & Safety

### API Keys & Secrets

- **S-1 (MUST)** NEVER commit API keys or secrets
- **S-2 (MUST)** Use environment variables or config files
- **S-3 (MUST)** Add `.env` to `.gitignore`

### Data Validation

- **S-4 (MUST)** Validate all external data before using
- **S-5 (MUST)** Sanitize user input in Python interface
- **S-6 (SHOULD)** Use type system to prevent invalid states

---

## 10 — Python Interface Guidelines

### PyO3 Bindings

- **PY-1 (MUST)** Expose only high-level APIs to Python
- **PY-2 (MUST)** Keep hot path in Rust, not Python
- **PY-3 (SHOULD)** Provide Python type stubs (.pyi files)

### Strategy Development

- **PY-4 (SHOULD)** Allow humans to write strategies in Python
- **PY-5 (MUST)** Strategies call into Rust for execution
- **PY-6 (SHOULD)** Provide REPL/notebook interface for exploration

---

## Remember: Shortcuts

### QNEW - New Session
```
Understand all BEST PRACTICES in CLAUDE.md.
This is a HIGH-PERFORMANCE quant system.
Every microsecond counts.
```

### QPERF - Check Performance
```
Profile the hot path with perf/flamegraph.
Are we allocating? Are we branching? Are we cache-missing?
Report latency in nanoseconds.
```

### QCODE - Implement Feature
```
1. Write tests first (TDD)
2. Implement in Rust
3. Benchmark hot path
4. Document human insight (for alphas)
5. Run: cargo fmt && cargo clippy && cargo test && cargo bench
```

### QALPHA - Add New Alpha Model
```
1. Document the HUMAN INSIGHT
2. Explain WHY humans behave this way
3. Implement in Rust with confidence scores
4. Test with realistic scenarios
5. Benchmark latency
```

### QGIT - Commit Changes
```
cargo fmt && cargo clippy && cargo test
git add .
git commit -m "type(scope): description"
git push
```

---

## Project-Specific Notes

### The Five Human-Only Alphas

1. **Panic Detector**: Models fear/greed, not just volatility
2. **Narrative Shift**: Detects story changes ("inflation transitory" → "persistent")
3. **Crowd Behavior**: Exploits retail irrationality (WSB meme stocks)
4. **Structural Inefficiency**: Exploits human institutions (index rebalancing)
5. **Creative Synthesis**: Combines unrelated signals (weather → e-commerce)

### Performance Targets

| Operation | Target | Why |
|-----------|--------|-----|
| Signal generation | < 1μs | Compensate for network latency |
| Order creation | < 100ns | Hot path critical |
| Indicator calculation | < 500ns | SIMD-optimized |
| Logging | < 50ns | Async, lock-free |

### What Makes This Different

**Banks have**: Speed + Math
**We have**: Speed + Math + **Human Psychology** + **Narrative Analysis** + **Creativity**

The last three are our **unfair advantage**.

---

## Questions Before Implementation

Before implementing a feature, ask:

1. **Performance**: Does this need to be fast? (Hot path?)
2. **Human Insight**: What human behavior are we modeling?
3. **Testability**: How do we test this realistically?
4. **Creativity**: Is this something banks CAN'T/WON'T build?

---

**Remember**: We're not trying to be faster than banks (we can't, no colocation).
We're trying to be **smarter** (model humans) and **more creative** (build strategies they can't).

That's The Human Edge.
