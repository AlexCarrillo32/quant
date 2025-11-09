# Ultra-Low Latency Performance Guide

## Current Status

### ✅ Implemented (Phase 2.5)
- **Performance module** ([src/core/performance.rs](src/core/performance.rs))
  - `PrecisionTimer` - Nanosecond-precision timing
  - `PerformanceStats` - Track latencies in nanoseconds
  - `CpuPinning` - CPU affinity API (ready to use)
  - `OrderArena` - Memory arena interface (stub)
  - `PerformanceConfig` - Configuration for all optimizations

- **Inline optimizations** - Hot path functions
  - `Price::value()` - `#[inline(always)]`
  - `Confidence` methods - `#[inline(always)]`
  - Timer methods - `#[inline(always)]`

### ⏳ Ready to Implement (Phase 3)
- CPU pinning implementation (Linux/Windows)
- Lock-free data structures (crossbeam channels)
- Memory arena actual implementation
- C++ SIMD math kernels

---

## Performance Targets (from CLAUDE.md)

| Operation | Current | Target | Status |
|-----------|---------|--------|--------|
| Signal generation | ~1ms | <1μs | ⚠️ Need benchmarks |
| Order creation | ~100μs | <100ns | ⏳ Not implemented |
| Indicator calc | ~10ms | <500ns | ⏳ No indicators yet |
| Engine cycle | ~2s | <10ms | ⚠️ Yahoo API limited |
| Logging | ~50μs | <50ns | ⏳ Need async logger |

**Goal**: 1000x speed improvement from naive implementation

---

## Why Ultra-Low Latency Matters

### The Math
```
Market moves: 0.5-2.0% in minutes
Network latency: 5-20ms (without colocation)
Our edge: React 60 seconds before others

Example:
- SPY at $670.00
- Panic detected at 09:30:15.000
- Our entry: 09:30:15.001 (1ms later)
- Others' entry: 09:30:20.000 (5 seconds later)
- Price recovered: +1.5% = $10.05 profit
- If we're 5s late: +1.2% = $8.04 profit
- Lost profit: $2.01 per share × 100 shares = $201
```

**Every millisecond = money**

### Strategy Requirement
- Target: 0.5-2% profits
- Hold time: Minutes to hours
- Need to be first to opportunities
- Can't afford 5-second delays

---

## Phase 3 Implementation Plan

### Step 1: CPU Pinning (2-3 hours)

**Why**: Prevents context switches, keeps cache hot

```rust
// Pin trading thread to dedicated core
let pinning = CpuPinning::new(vec![0, 1, 2, 3]);
pinning.pin_current_thread(0)?; // Core 0 for trading

// Reduces latency by ~5-10μs
```

**Implementation**:
- Already scaffolded in `performance.rs`
- Use `core_affinity` crate (already in Cargo.toml)
- Test on Linux first (best support)
- macOS fallback: high thread priority

**Test**:
```bash
cargo test test_cpu_pinning
```

---

### Step 2: Lock-Free Data Structures (3-4 hours)

**Why**: Locks add ~1-5μs per operation

```rust
// Replace std channels with crossbeam
use crossbeam::channel::{bounded, unbounded};

// Signal channel (lock-free)
let (tx, rx) = bounded(1000);

// Reduces contention by 10-100x
```

**Implementation**:
- Use `crossbeam` (already in Cargo.toml)
- Replace `Vec<Signal>` with lock-free queue
- Use `lockfree` crate for order book

**Test**:
```bash
cargo bench lock_free_vs_mutex
```

---

### Step 3: Memory Arena (4-6 hours)

**Why**: Allocation in hot path = 10-100μs penalty

```rust
// Pre-allocate 10,000 order slots
let arena = OrderArena::new(10_000);

// Hot path: NO allocation
let order_slot = arena.alloc()?;  // <100ns
*order_slot = order;

// vs naive: Vec::push() = 10-100μs
```

**Implementation**:
- Use `bumpalo` or custom arena
- Pre-allocate at engine startup
- Zero allocation in hot path

**Validation**:
```rust
// MUST pass: No allocation in hot path
assert!(arena.allocations() == 0);
```

---

### Step 4: Async Lock-Free Logging (2-3 hours)

**Why**: I/O blocks = 50μs+ penalty

```rust
// Hot path writes to ring buffer (50ns)
LOG_RING_BUFFER.write(message);

// Background thread flushes to disk
tokio::spawn(async {
    loop {
        LOG_RING_BUFFER.flush_to_disk().await;
        sleep(1ms).await;
    }
});
```

**Implementation**:
- Use `tracing-subscriber` with async layer
- Ring buffer size: 100,000 messages
- Flush every 1ms (batch writes)

---

### Step 5: C++ SIMD Math Kernels (6-8 hours)

**Why**: Vectorized math = 4-8x speedup

```cpp
// Calculate EMA for 1000 prices (AVX2)
void ema_simd(float* prices, float* out, int len, float alpha) {
    __m256 va = _mm256_set1_ps(alpha);
    // Process 8 prices at once
    for (int i = 0; i < len; i += 8) {
        __m256 vp = _mm256_load_ps(&prices[i]);
        // ... SIMD EMA calculation
    }
}
```

**Implementation**:
- Create `src/indicators/simd.cpp`
- Use AVX2 instructions (4-8 floats parallel)
- Expose via FFI to Rust
- Fallback to scalar for non-AVX2 CPUs

**Speedup**:
- Naive EMA: ~10ms for 1000 prices
- SIMD EMA: ~500ns for 1000 prices
- **20x faster**

---

## Performance Testing

### Benchmark Setup

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_signal_generation(c: &mut Criterion) {
    let panic_detector = PanicDetectorAlpha::new();
    let market_data = create_test_data();

    c.bench_function("panic_detector_signal", |b| {
        b.iter(|| {
            panic_detector.detect(black_box(&market_data))
        });
    });
}

criterion_group!(benches, bench_signal_generation);
criterion_main!(benches);
```

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench bench_signal_generation

# With flamegraph profiling
cargo flamegraph --bench signal_bench
```

### Success Criteria

```
✅ Signal generation: <1μs (1000ns)
✅ Order creation: <100ns
✅ Indicator calc: <500ns
✅ Zero allocations in hot path
✅ CPU cache hit rate >95%
```

---

## Hardware Optimization

### CPU Selection
- **Minimum**: 4 cores, 3.0GHz+
- **Recommended**: 8 cores, 4.0GHz+, AVX2
- **Optimal**: AMD Threadripper / Intel i9, AVX-512

### Memory
- **Minimum**: 8GB RAM
- **Recommended**: 16GB+ RAM, 3200MHz+
- Use NUMA-aware allocation on multi-socket

### Storage
- **Don't use HDD** - too slow for logs
- **Minimum**: SSD
- **Recommended**: NVMe SSD (for logging)

---

## Monitoring Performance

### Real-Time Metrics

```rust
// Track latency per cycle
let stats = engine.performance_stats();

if !stats.meets_targets() {
    warn!("Performance degraded: {}", stats.report());
    // Maybe reduce position size or pause trading
}
```

### Alerts

```rust
// Alert if latency >10μs
if stats.avg_signal_latency_ns > 10_000 {
    alert!("CRITICAL: Signal latency = {}μs",
        stats.avg_signal_latency_ns / 1000);
}
```

---

## Performance Anti-Patterns

### ❌ DON'T

```rust
// BAD: Allocation in hot path
let orders = Vec::new();
orders.push(order);  // Allocates!

// BAD: Locks in hot path
let mutex = Mutex::new(data);
let guard = mutex.lock();  // Blocks!

// BAD: I/O in hot path
println!("Signal: {:?}", signal);  // Blocks!

// BAD: Branches in hot path
if signal.confidence > 0.7 {  // Branch prediction miss
    execute_order();
}
```

### ✅ DO

```rust
// GOOD: Pre-allocated arena
let order_slot = ORDER_ARENA.alloc();  // No allocation

// GOOD: Lock-free queue
SIGNAL_QUEUE.push(signal);  // No locks

// GOOD: Async logging
LOG_BUFFER.write(message);  // Non-blocking

// GOOD: Branchless
let should_trade = (confidence > 0.7) as usize;  // Branchless
execute_order_array[should_trade]();
```

---

## Deployment Configuration

### Production Settings

```rust
EngineConfig {
    // Fast updates for ultra-low latency
    update_interval: Duration::from_secs(1),  // 1 second

    // Performance optimizations
    cpu_pinning: true,
    cpu_cores: vec![0, 1],  // Dedicated cores
    lockfree: true,
    preallocated_memory: 10_000_000,  // 10MB arena
    simd: true,
}
```

### Linux Kernel Tuning

```bash
# Disable CPU frequency scaling
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Reduce context switch overhead
echo 1 | sudo tee /proc/sys/kernel/sched_rt_runtime_us

# Disable transparent hugepages (predictable latency)
echo never | sudo tee /sys/kernel/mm/transparent_hugepage/enabled
```

---

## Next Steps

1. ✅ **Phase 2.5 DONE**: Performance infrastructure
2. **Phase 3 TODO** (2-3 days):
   - [ ] CPU pinning implementation
   - [ ] Lock-free channels
   - [ ] Memory arena
   - [ ] Async logging
   - [ ] Benchmarking suite
3. **Phase 4 TODO** (3-4 days):
   - [ ] C++ SIMD kernels
   - [ ] Profiling & optimization
   - [ ] Performance testing under load

**Timeline**: 5-7 days to reach <1μs signal generation

---

**Remember**: Speed is our competitive advantage. Banks have colocation, we have low latency + human psychology.
