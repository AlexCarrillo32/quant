//! PyO3 Python bindings for core types
#![cfg(feature = "python")]
//!
//! Exposes Rust types to Python with zero-copy semantics where possible.
//!
//! # Example (Python)
//!
//! ```python
//! from quant_engine import Price, Quantity, Symbol, Signal, SignalAction
//!
//! # Create types
//! symbol = Symbol("AAPL")
//! price = Price(150.25)
//! qty = Quantity.buy(100)
//!
//! # Work with signals
//! signal = Signal.new(
//!     symbol,
//!     SignalAction.Buy,
//!     confidence=0.85,
//!     reason="Strong momentum",
//!     source="MyAlpha"
//! )
//! ```

use crate::types::{
    Confidence as RustConfidence, Price as RustPrice, Quantity as RustQuantity,
    Signal as RustSignal, SignalAction as RustSignalAction, Symbol as RustSymbol,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

// ============================================================================
// Price Type
// ============================================================================

/// Price in dollars (always positive)
///
/// Examples:
///     >>> price = Price(150.25)
///     >>> price.value()
///     150.25
///     >>> price.percent_change(Price(165.0))
///     9.77
#[pyclass(name = "Price")]
#[derive(Clone)]
pub struct PyPrice {
    inner: RustPrice,
}

#[pymethods]
impl PyPrice {
    /// Create a new price
    ///
    /// Args:
    ///     value (float): Price value (must be positive)
    ///
    /// Raises:
    ///     ValueError: If price is negative, zero, NaN, or infinite
    #[new]
    fn new(value: f64) -> PyResult<Self> {
        let price = RustPrice::new(value).map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(PyPrice { inner: price })
    }

    /// Get the raw price value
    fn value(&self) -> f64 {
        self.inner.value()
    }

    /// Calculate percentage change to another price
    ///
    /// Args:
    ///     other (Price): Target price
    ///
    /// Returns:
    ///     float: Percentage change
    fn percent_change(&self, other: &PyPrice) -> f64 {
        self.inner.percent_change(other.inner)
    }

    /// String representation
    fn __str__(&self) -> String {
        format!("${:.2}", self.inner.value())
    }

    fn __repr__(&self) -> String {
        format!("Price({:.2})", self.inner.value())
    }

    /// Rich comparison operators
    fn __richcmp__(&self, other: &PyPrice, op: pyo3::basic::CompareOp) -> PyResult<bool> {
        use pyo3::basic::CompareOp;
        match op {
            CompareOp::Lt => Ok(self.inner < other.inner),
            CompareOp::Le => Ok(self.inner <= other.inner),
            CompareOp::Eq => Ok(self.inner == other.inner),
            CompareOp::Ne => Ok(self.inner != other.inner),
            CompareOp::Gt => Ok(self.inner > other.inner),
            CompareOp::Ge => Ok(self.inner >= other.inner),
        }
    }

    /// Addition
    fn __add__(&self, other: &PyPrice) -> PyPrice {
        PyPrice {
            inner: self.inner + other.inner,
        }
    }

    /// Subtraction
    fn __sub__(&self, other: &PyPrice) -> PyPrice {
        PyPrice {
            inner: self.inner - other.inner,
        }
    }

    /// Multiplication by scalar
    fn __mul__(&self, scalar: f64) -> PyPrice {
        PyPrice {
            inner: self.inner * scalar,
        }
    }

    /// Division by scalar
    fn __truediv__(&self, scalar: f64) -> PyPrice {
        PyPrice {
            inner: self.inner / scalar,
        }
    }
}

// Internal conversion helpers
impl PyPrice {
    pub fn to_rust(&self) -> RustPrice {
        self.inner
    }

    pub fn from_rust(price: RustPrice) -> Self {
        PyPrice { inner: price }
    }
}

// ============================================================================
// Quantity Type
// ============================================================================

/// Quantity (positive = buy, negative = sell)
///
/// Examples:
///     >>> buy = Quantity.buy(100)
///     >>> buy.value()
///     100
///     >>> buy.is_buy()
///     True
///     >>> sell = Quantity.sell(50)
///     >>> sell.value()
///     -50
#[pyclass(name = "Quantity")]
#[derive(Clone)]
pub struct PyQuantity {
    inner: RustQuantity,
}

#[pymethods]
impl PyQuantity {
    /// Create a quantity from a signed integer
    ///
    /// Args:
    ///     value (int): Quantity (positive = buy, negative = sell)
    ///
    /// Raises:
    ///     ValueError: If quantity is zero
    #[new]
    fn new(value: i64) -> PyResult<Self> {
        let qty = RustQuantity::new(value).map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(PyQuantity { inner: qty })
    }

    /// Create a buy quantity
    ///
    /// Args:
    ///     value (int): Number of shares to buy
    #[staticmethod]
    fn buy(value: u64) -> Self {
        PyQuantity {
            inner: RustQuantity::buy(value),
        }
    }

    /// Create a sell quantity
    ///
    /// Args:
    ///     value (int): Number of shares to sell
    #[staticmethod]
    fn sell(value: u64) -> Self {
        PyQuantity {
            inner: RustQuantity::sell(value),
        }
    }

    /// Get the signed value
    fn value(&self) -> i64 {
        self.inner.value()
    }

    /// Get the absolute value
    fn abs(&self) -> u64 {
        self.inner.abs()
    }

    /// Check if this is a buy order
    fn is_buy(&self) -> bool {
        self.inner.is_buy()
    }

    /// Check if this is a sell order
    fn is_sell(&self) -> bool {
        self.inner.is_sell()
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        format!("Quantity({})", self.inner.value())
    }
}

impl PyQuantity {
    pub fn to_rust(&self) -> RustQuantity {
        self.inner
    }

    pub fn from_rust(qty: RustQuantity) -> Self {
        PyQuantity { inner: qty }
    }
}

// ============================================================================
// Symbol Type
// ============================================================================

/// Stock symbol (e.g., "AAPL", "GOOGL")
///
/// Examples:
///     >>> symbol = Symbol("AAPL")
///     >>> symbol.as_str()
///     'AAPL'
///     >>> symbol2 = Symbol("aapl")  # Auto-uppercased
///     >>> symbol2.as_str()
///     'AAPL'
#[pyclass(name = "Symbol")]
#[derive(Clone)]
pub struct PySymbol {
    inner: RustSymbol,
}

#[pymethods]
impl PySymbol {
    /// Create a new symbol
    ///
    /// Args:
    ///     value (str): Symbol string (e.g., "AAPL")
    ///
    /// Raises:
    ///     ValueError: If symbol is invalid
    #[new]
    fn new(value: &str) -> PyResult<Self> {
        let symbol = RustSymbol::new(value).map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(PySymbol { inner: symbol })
    }

    /// Get the symbol as a string
    fn as_str(&self) -> &str {
        self.inner.as_str()
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        format!("Symbol('{}')", self.inner.as_str())
    }

    fn __hash__(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.inner.hash(&mut hasher);
        hasher.finish()
    }

    fn __eq__(&self, other: &PySymbol) -> bool {
        self.inner == other.inner
    }
}

impl PySymbol {
    pub fn to_rust(&self) -> RustSymbol {
        self.inner.clone()
    }

    pub fn from_rust(symbol: RustSymbol) -> Self {
        PySymbol { inner: symbol }
    }
}

// ============================================================================
// SignalAction Enum
// ============================================================================

/// Trading action
///
/// Values:
///     Buy: Buy signal
///     Sell: Sell signal
///     Close: Close position
///     Hold: Do nothing
#[pyclass(name = "SignalAction")]
#[derive(Clone, Copy)]
pub enum PySignalAction {
    Buy,
    Sell,
    Close,
    Hold,
}

#[pymethods]
impl PySignalAction {
    fn __str__(&self) -> &'static str {
        match self {
            PySignalAction::Buy => "BUY",
            PySignalAction::Sell => "SELL",
            PySignalAction::Close => "CLOSE",
            PySignalAction::Hold => "HOLD",
        }
    }

    fn __repr__(&self) -> String {
        format!("SignalAction.{}", self.__str__())
    }
}

impl PySignalAction {
    fn to_rust(&self) -> RustSignalAction {
        match self {
            PySignalAction::Buy => RustSignalAction::Buy,
            PySignalAction::Sell => RustSignalAction::Sell,
            PySignalAction::Close => RustSignalAction::Close,
            PySignalAction::Hold => RustSignalAction::Hold,
        }
    }

    fn from_rust(action: RustSignalAction) -> Self {
        match action {
            RustSignalAction::Buy => PySignalAction::Buy,
            RustSignalAction::Sell => PySignalAction::Sell,
            RustSignalAction::Close => PySignalAction::Close,
            RustSignalAction::Hold => PySignalAction::Hold,
        }
    }
}

// ============================================================================
// Confidence Type
// ============================================================================

/// Confidence score (0.0 to 1.0)
///
/// Examples:
///     >>> conf = Confidence(0.85)
///     >>> conf.value()
///     0.85
///     >>> conf.is_high()
///     True
///     >>> conf.as_percent()
///     85.0
#[pyclass(name = "Confidence")]
#[derive(Clone)]
pub struct PyConfidence {
    inner: RustConfidence,
}

#[pymethods]
impl PyConfidence {
    /// Create a new confidence score
    ///
    /// Args:
    ///     value (float): Confidence value between 0.0 and 1.0
    ///
    /// Raises:
    ///     ValueError: If value is not in [0.0, 1.0] range
    #[new]
    fn new(value: f64) -> PyResult<Self> {
        let conf =
            RustConfidence::new(value).map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(PyConfidence { inner: conf })
    }

    /// Get the raw confidence value
    fn value(&self) -> f64 {
        self.inner.value()
    }

    /// Get confidence as percentage (0-100)
    fn as_percent(&self) -> f64 {
        self.inner.as_percent()
    }

    /// Check if confidence is low (< 0.5)
    fn is_low(&self) -> bool {
        self.inner.is_low()
    }

    /// Check if confidence is medium (0.5 - 0.75)
    fn is_medium(&self) -> bool {
        self.inner.is_medium()
    }

    /// Check if confidence is high (> 0.75)
    fn is_high(&self) -> bool {
        self.inner.is_high()
    }

    fn __str__(&self) -> String {
        format!("{:.1}%", self.inner.as_percent())
    }

    fn __repr__(&self) -> String {
        format!("Confidence({:.2})", self.inner.value())
    }
}

impl PyConfidence {
    pub fn to_rust(&self) -> RustConfidence {
        self.inner
    }

    pub fn from_rust(conf: RustConfidence) -> Self {
        PyConfidence { inner: conf }
    }
}

// ============================================================================
// Signal Type
// ============================================================================

/// Trading signal with metadata
///
/// Examples:
///     >>> signal = Signal.new(
///     ...     Symbol("AAPL"),
///     ...     SignalAction.Buy,
///     ...     Confidence(0.85),
///     ...     "Strong momentum",
///     ...     "MyAlpha"
///     ... )
///     >>> signal.is_actionable()
///     True
#[pyclass(name = "Signal")]
#[derive(Clone)]
pub struct PySignal {
    inner: RustSignal,
}

#[pymethods]
impl PySignal {
    /// Create a new signal
    ///
    /// Args:
    ///     symbol (Symbol): Symbol to trade
    ///     action (SignalAction): Action to take
    ///     confidence (Confidence): Signal confidence
    ///     reason (str): Human-readable reason
    ///     source (str): Alpha model name
    ///
    /// Returns:
    ///     Signal: New signal instance
    #[staticmethod]
    fn new(
        symbol: &PySymbol,
        action: PySignalAction,
        confidence: &PyConfidence,
        reason: &str,
        source: &str,
    ) -> Self {
        let signal = RustSignal::new(
            symbol.to_rust(),
            action.to_rust(),
            confidence.to_rust(),
            reason,
            source,
        );
        PySignal { inner: signal }
    }

    /// Get the symbol
    #[getter]
    fn symbol(&self) -> PySymbol {
        PySymbol::from_rust(self.inner.symbol.clone())
    }

    /// Get the action
    #[getter]
    fn action(&self) -> PySignalAction {
        PySignalAction::from_rust(self.inner.action)
    }

    /// Get the confidence
    #[getter]
    fn confidence(&self) -> PyConfidence {
        PyConfidence::from_rust(self.inner.confidence)
    }

    /// Get the reason
    #[getter]
    fn reason(&self) -> &str {
        &self.inner.reason
    }

    /// Get the source
    #[getter]
    fn source(&self) -> &str {
        &self.inner.source
    }

    /// Get target price (if set)
    #[getter]
    fn target_price(&self) -> Option<PyPrice> {
        self.inner.target_price.map(PyPrice::from_rust)
    }

    /// Get stop loss (if set)
    #[getter]
    fn stop_loss(&self) -> Option<PyPrice> {
        self.inner.stop_loss.map(PyPrice::from_rust)
    }

    /// Get take profit (if set)
    #[getter]
    fn take_profit(&self) -> Option<PyPrice> {
        self.inner.take_profit.map(PyPrice::from_rust)
    }

    /// Get quantity (if set)
    #[getter]
    fn quantity(&self) -> Option<PyQuantity> {
        self.inner.quantity.map(PyQuantity::from_rust)
    }

    /// Set target price
    fn with_target_price(&mut self, price: &PyPrice) {
        self.inner.target_price = Some(price.to_rust());
    }

    /// Set stop loss
    fn with_stop_loss(&mut self, price: &PyPrice) {
        self.inner.stop_loss = Some(price.to_rust());
    }

    /// Set take profit
    fn with_take_profit(&mut self, price: &PyPrice) {
        self.inner.take_profit = Some(price.to_rust());
    }

    /// Set quantity
    fn with_quantity(&mut self, quantity: &PyQuantity) {
        self.inner.quantity = Some(quantity.to_rust());
    }

    /// Check if signal is actionable (high confidence, not hold)
    fn is_actionable(&self) -> bool {
        self.inner.is_actionable()
    }

    /// Calculate risk/reward ratio
    ///
    /// Args:
    ///     current_price (Price): Current market price
    ///
    /// Returns:
    ///     float or None: Risk/reward ratio if prices are set
    fn risk_reward_ratio(&self, current_price: &PyPrice) -> Option<f64> {
        self.inner.risk_reward_ratio(current_price.to_rust())
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        format!(
            "Signal(symbol='{}', action={:?}, confidence={:.2})",
            self.inner.symbol.as_str(),
            self.inner.action,
            self.inner.confidence.value()
        )
    }
}

impl PySignal {
    pub fn to_rust(&self) -> RustSignal {
        self.inner.clone()
    }

    pub fn from_rust(signal: RustSignal) -> Self {
        PySignal { inner: signal }
    }
}

// ============================================================================
// Backtester Types
// ============================================================================

use crate::backtest::{
    BacktestConfig as RustBacktestConfig, BacktestReport as RustBacktestReport,
    BacktestResult as RustBacktestResult, Backtester as RustBacktester,
    PerformanceMetrics as RustMetrics,
};
use chrono::{DateTime, NaiveDate, TimeZone, Utc};

/// Backtester configuration
///
/// Examples:
///     >>> config = BacktestConfig()
///     >>> config.initial_capital = 100000.0
///     >>> config.commission_per_trade = 1.0
#[pyclass(name = "BacktestConfig")]
#[derive(Clone)]
pub struct PyBacktestConfig {
    inner: RustBacktestConfig,
}

#[pymethods]
impl PyBacktestConfig {
    /// Create new backtest config with defaults
    #[new]
    fn new() -> Self {
        PyBacktestConfig {
            inner: RustBacktestConfig::default(),
        }
    }

    /// Get/set initial capital
    #[getter]
    fn initial_capital(&self) -> f64 {
        self.inner.initial_capital
    }

    #[setter]
    fn set_initial_capital(&mut self, value: f64) {
        self.inner.initial_capital = value;
    }

    /// Get/set commission per trade
    #[getter]
    fn commission_per_trade(&self) -> f64 {
        self.inner.commission_per_trade
    }

    #[setter]
    fn set_commission_per_trade(&mut self, value: f64) {
        self.inner.commission_per_trade = value;
    }

    /// Get/set slippage (%)
    #[getter]
    fn slippage_pct(&self) -> f64 {
        self.inner.slippage_pct
    }

    #[setter]
    fn set_slippage_pct(&mut self, value: f64) {
        self.inner.slippage_pct = value;
    }

    fn __repr__(&self) -> String {
        format!(
            "BacktestConfig(capital=${:.2}, commission=${:.2}, slippage={:.2}%)",
            self.inner.initial_capital, self.inner.commission_per_trade, self.inner.slippage_pct
        )
    }
}

/// Performance metrics from backtest
///
/// Examples:
///     >>> metrics = result.metrics()
///     >>> print(f"Sharpe: {metrics.sharpe_ratio()}")
#[pyclass(name = "PerformanceMetrics")]
#[derive(Clone)]
pub struct PyPerformanceMetrics {
    inner: RustMetrics,
}

#[pymethods]
impl PyPerformanceMetrics {
    /// Total return (%)
    fn total_return_pct(&self) -> f64 {
        self.inner.total_return_pct
    }

    /// Annual return (%)
    fn annual_return_pct(&self) -> f64 {
        self.inner.annual_return_pct
    }

    /// Sharpe ratio
    fn sharpe_ratio(&self) -> f64 {
        self.inner.sharpe_ratio
    }

    /// Sortino ratio
    fn sortino_ratio(&self) -> f64 {
        self.inner.sortino_ratio
    }

    /// Maximum drawdown (%)
    fn max_drawdown_pct(&self) -> f64 {
        self.inner.max_drawdown_pct
    }

    /// Win rate (0.0 to 1.0)
    fn win_rate(&self) -> f64 {
        self.inner.win_rate
    }

    /// Total number of trades
    fn total_trades(&self) -> usize {
        self.inner.total_trades
    }

    /// Number of winning trades
    fn winning_trades(&self) -> usize {
        self.inner.winning_trades
    }

    /// Number of losing trades
    fn losing_trades(&self) -> usize {
        self.inner.losing_trades
    }

    /// Average win ($)
    fn avg_win(&self) -> f64 {
        self.inner.avg_win
    }

    /// Average loss ($)
    fn avg_loss(&self) -> f64 {
        self.inner.avg_loss
    }

    /// Profit factor (total wins / total losses)
    fn profit_factor(&self) -> f64 {
        self.inner.profit_factor
    }

    fn __repr__(&self) -> String {
        format!(
            "PerformanceMetrics(return={:.2}%, sharpe={:.2}, max_dd={:.2}%, win_rate={:.1}%)",
            self.inner.total_return_pct,
            self.inner.sharpe_ratio,
            self.inner.max_drawdown_pct,
            self.inner.win_rate * 100.0
        )
    }

    fn __str__(&self) -> String {
        format!(
            "Total Return: {:.2}%\n\
             Annual Return: {:.2}%\n\
             Sharpe Ratio: {:.2}\n\
             Sortino Ratio: {:.2}\n\
             Max Drawdown: {:.2}%\n\
             Win Rate: {:.1}%\n\
             Total Trades: {}\n\
             Profit Factor: {:.2}",
            self.inner.total_return_pct,
            self.inner.annual_return_pct,
            self.inner.sharpe_ratio,
            self.inner.sortino_ratio,
            self.inner.max_drawdown_pct,
            self.inner.win_rate * 100.0,
            self.inner.total_trades,
            self.inner.profit_factor
        )
    }
}

/// Backtest result
///
/// Examples:
///     >>> result = backtester.run(signals, start_date, end_date)
///     >>> print(result.metrics())
///     >>> print(f"Final value: ${result.final_portfolio_value()}")
#[pyclass(name = "BacktestResult")]
pub struct PyBacktestResult {
    inner: RustBacktestResult,
}

#[pymethods]
impl PyBacktestResult {
    /// Get performance metrics
    fn metrics(&self) -> PyPerformanceMetrics {
        PyPerformanceMetrics {
            inner: self.inner.metrics.clone(),
        }
    }

    /// Final portfolio value
    fn final_portfolio_value(&self) -> f64 {
        self.inner.final_capital
    }

    /// Number of trades executed
    fn total_trades(&self) -> usize {
        self.inner.trades.len()
    }

    /// Get report summary
    fn summary(&self) -> String {
        format!(
            "Backtest Results:\n\
             Final Value: ${:.2}\n\
             Total Trades: {}\n\
             {}",
            self.inner.final_capital,
            self.inner.trades.len(),
            self.metrics().__str__()
        )
    }

    fn __repr__(&self) -> String {
        format!(
            "BacktestResult(final_value=${:.2}, trades={})",
            self.inner.final_capital,
            self.inner.trades.len()
        )
    }
}

/// Backtester - Test strategies on historical data
///
/// Examples:
///     >>> backtester = Backtester(initial_capital=100000.0)
///     >>> signals = [signal1, signal2, signal3]
///     >>> result = backtester.run(signals)
///     >>> print(result.metrics())
#[pyclass(name = "Backtester")]
pub struct PyBacktester {
    inner: RustBacktester,
}

#[pymethods]
impl PyBacktester {
    /// Create new backtester
    ///
    /// Args:
    ///     initial_capital (float): Starting capital (default: 10000.0)
    ///     commission (float): Commission per trade (default: 0.0)
    ///     slippage (float): Slippage % (default: 0.1)
    #[new]
    #[pyo3(signature = (initial_capital=10000.0, commission=0.0, slippage=0.1))]
    fn new(initial_capital: f64, commission: f64, slippage: f64) -> Self {
        let config = RustBacktestConfig {
            initial_capital,
            commission_per_trade: commission,
            slippage_pct: slippage,
            ..Default::default()
        };

        PyBacktester {
            inner: RustBacktester::new(config),
        }
    }

    /// Run backtest with list of signals
    ///
    /// Args:
    ///     signals: List of Signal objects to backtest
    ///
    /// Returns:
    ///     BacktestResult with performance metrics
    fn run(&mut self, signals: Vec<PySignal>) -> PyResult<PyBacktestResult> {
        // Convert Python signals to Rust signals
        let rust_signals: Vec<RustSignal> = signals.iter().map(|s| s.to_rust()).collect();

        // Run backtest
        let result = self
            .inner
            .run_with_signals(rust_signals)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        Ok(PyBacktestResult { inner: result })
    }

    fn __repr__(&self) -> String {
        format!(
            "Backtester(capital=${:.2})",
            self.inner.config.initial_capital
        )
    }
}

// ============================================================================
// Module Registration
// ============================================================================

/// Python module initialization
///
/// This is called when Python imports the module
#[pymodule]
fn quant_engine(_py: Python, m: &PyModule) -> PyResult<()> {
    // Register core types
    m.add_class::<PyPrice>()?;
    m.add_class::<PyQuantity>()?;
    m.add_class::<PySymbol>()?;
    m.add_class::<PySignalAction>()?;
    m.add_class::<PyConfidence>()?;
    m.add_class::<PySignal>()?;

    // Register backtesting types
    m.add_class::<PyBacktestConfig>()?;
    m.add_class::<PyPerformanceMetrics>()?;
    m.add_class::<PyBacktestResult>()?;
    m.add_class::<PyBacktester>()?;

    // Module metadata
    m.add("__version__", "0.1.0")?;
    m.add("__doc__", "High-performance quant trading engine with human intuition layer")?;

    Ok(())
}
