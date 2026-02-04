//! Shared CLI argument structs for parameter parity across commands.
//!
//! This module provides reusable argument groups that ensure consistent behavior
//! between `route` and `scout` commands. By using `#[command(flatten)]`, these
//! structs eliminate parameter duplication and enforce identical flag names
//! across all routing commands.
//!
//! # Architecture
//!
//! Each shared struct handles a single concern:
//! - [`CommonRouteConstraints`]: Routing filters (avoidance, distance, temperature)
//! - [`CommonShipConfig`]: Ship and fuel projection settings
//! - [`CommonHeatConfig`]: Heat mechanics and temperature models
//!
//! # Example Usage
//!
//! ```rust,ignore
//! use clap::Args;
//! use crate::common_args::{CommonRouteConstraints, CommonShipConfig, CommonHeatConfig};
//!
//! #[derive(Args, Debug, Clone)]
//! struct MyRouteCommand {
//!     #[command(flatten)]
//!     constraints: CommonRouteConstraints,
//!     
//!     #[command(flatten)]
//!     ship: CommonShipConfig,
//!     
//!     #[command(flatten)]
//!     heat: CommonHeatConfig,
//!     
//!     // Command-specific flags...
//! }
//! ```

use clap::{ArgAction, Args, ValueEnum};

/// Shared routing constraints for pathfinding operations.
///
/// These constraints are used by both `route` and `scout range` commands
/// to control which systems and jump types are considered during path planning.
///
/// # Examples
///
/// ```rust,ignore
/// let constraints = CommonRouteConstraints {
///     max_jump: Some(50.0),
///     avoid: vec!["Brana".to_string(), "H:2L2S".to_string()],
///     avoid_gates: false,
///     max_temp: Some(8000.0),
/// };
/// ```
#[derive(Args, Debug, Clone, Default)]
pub struct CommonRouteConstraints {
    /// Maximum jump distance in light-years.
    ///
    /// When specified, systems beyond this distance cannot be reached via spatial jumps.
    /// Gate jumps are unaffected by this constraint.
    #[arg(long = "max-jump", help_heading = "ROUTING CONSTRAINTS")]
    pub max_jump: Option<f64>,

    /// Systems to avoid when building the path. Repeat for multiple systems.
    ///
    /// The pathfinding algorithm will exclude these systems from all routes
    /// (both gate and spatial jumps).
    ///
    /// # Example
    ///
    /// ```bash
    /// evefrontier-cli route --from Nod --to Brana --avoid "H:2L2S" --avoid "G:3OA0"
    /// ```
    #[arg(long = "avoid", help_heading = "ROUTING CONSTRAINTS")]
    pub avoid: Vec<String>,

    /// Avoid gates entirely (prefer spatial or traversal routes).
    ///
    /// When enabled, the pathfinding algorithm will only consider spatial jumps
    /// and traversal (warp drive) between systems. This is useful for finding
    /// routes through uncharted space or avoiding gate travel fees.
    #[arg(long = "avoid-gates", action = ArgAction::SetTrue, help_heading = "ROUTING CONSTRAINTS")]
    pub avoid_gates: bool,

    /// Maximum system temperature threshold in Kelvin.
    ///
    /// Only applies to spatial jumps - systems with star temperature above this
    /// threshold cannot be reached via spatial jumps (ships would overheat).
    /// Gate jumps are unaffected by temperature.
    ///
    /// # Typical Values
    ///
    /// - 5000K: Cool red dwarfs only
    /// - 8000K: Most player-accessible systems
    /// - 10000K: Includes some hot white/blue stars
    #[arg(long = "max-temp", help_heading = "ROUTING CONSTRAINTS")]
    pub max_temp: Option<f64>,
}

/// Shared ship and fuel configuration for fuel projection.
///
/// These parameters control how fuel consumption is calculated during route planning.
/// Fuel projections require a valid ship name from the ship catalog (`ship_data.csv`).
///
/// # Examples
///
/// ```rust,ignore
/// let ship_config = CommonShipConfig {
///     ship: Some("Reflex".to_string()),
///     fuel_quality: 50.0,
///     cargo_mass: 1000.0,
///     fuel_load: Some(500.0),
///     dynamic_mass: true,
/// };
/// ```
#[derive(Args, Debug, Clone)]
pub struct CommonShipConfig {
    /// Ship name for fuel projection (case-insensitive).
    ///
    /// When specified, the route planner will calculate fuel consumption for each hop
    /// based on the ship's mass, fuel capacity, and cargo. Use `evefrontier-cli ships`
    /// to list available ships.
    ///
    /// # Example
    ///
    /// ```bash
    /// evefrontier-cli route --from Nod --to Brana --ship Reflex
    /// ```
    #[arg(long = "ship", help_heading = "SHIP & FUEL")]
    pub ship: Option<String>,

    /// Fuel quality rating (1-100). Higher quality = more efficient jumps.
    ///
    /// Fuel quality directly affects fuel consumption. Higher quality fuel reduces
    /// the amount burned per light-year. Default is 10 (standard fuel).
    ///
    /// # Formula
    ///
    /// `fuel_cost = (total_mass_kg / 100000) * (fuel_quality / 100) * distance_ly`
    #[arg(long = "fuel-quality", default_value = "10", value_parser = parse_fuel_quality, help_heading = "SHIP & FUEL")]
    pub fuel_quality: f64,

    /// Cargo mass in kilograms.
    ///
    /// Added to the ship's hull mass and fuel mass to calculate total mass for
    /// fuel consumption. Default is 0 (empty cargo hold).
    #[arg(long = "cargo-mass", default_value = "0", value_parser = parse_non_negative, help_heading = "SHIP & FUEL")]
    pub cargo_mass: f64,

    /// Initial fuel load in units. Defaults to full capacity.
    ///
    /// When specified, the route planner starts with this fuel amount instead of
    /// the ship's maximum capacity. Useful for planning routes with partial fuel loads.
    #[arg(long = "fuel-load", value_parser = parse_non_negative, help_heading = "SHIP & FUEL")]
    pub fuel_load: Option<f64>,

    /// Recalculate mass after each hop as fuel is consumed.
    ///
    /// When enabled (dynamic mass mode), the route planner recalculates fuel consumption
    /// for each hop based on the remaining fuel. This produces more accurate projections
    /// for long routes but increases computation time.
    ///
    /// When disabled (static mass mode), fuel consumption is calculated once using the
    /// initial mass (hull + cargo + full fuel load).
    #[arg(long = "dynamic-mass", action = ArgAction::SetTrue, help_heading = "SHIP & FUEL")]
    pub dynamic_mass: bool,
}

/// Shared heat configuration for temperature-aware routing.
///
/// These parameters control how the route planner handles heat buildup during spatial jumps.
/// Heat mechanics model the thermal stress on ships when jumping through stellar radiation.
///
/// # Examples
///
/// ```rust,ignore
/// let heat_config = CommonHeatConfig {
///     avoid_critical_state: true,
///     no_avoid_critical_state: false,
///     sys_temp_curve: TemperatureCurveArg::Flux,
/// };
/// ```
#[derive(Args, Debug, Clone)]
pub struct CommonHeatConfig {
    /// Heat-aware routing (rejects jumps reaching critical temperature ≥150K).
    ///
    /// When enabled, the route planner will avoid spatial jumps that would cause
    /// the ship to overheat beyond the critical threshold. Gate jumps are unaffected.
    #[arg(long = "avoid-critical-state", action = ArgAction::SetTrue, help_heading = "HEAT MECHANICS")]
    pub avoid_critical_state: bool,

    /// Disable temperature constraints for gate-only networks or high-risk planning.
    ///
    /// When enabled, this flag explicitly disables heat constraints. If both
    /// `--avoid-critical-state` and `--no-avoid-critical-state` are specified,
    /// this flag takes precedence (explicit disable).
    #[arg(long = "no-avoid-critical-state", action = ArgAction::SetTrue, help_heading = "HEAT MECHANICS")]
    pub no_avoid_critical_state: bool,

    /// Temperature calculation model: 'flux' or 'logistic'.
    ///
    /// - **flux**: Inverse-tangent model using radiative flux (L/d²) with inverse-square law.
    ///   Physically interpretable and validated (~1.2K MAE). Default.
    /// - **logistic**: Logistic curve model. Empirically fitted sigmoid function.
    ///   Validated alternative (~1.2K MAE).
    #[arg(long = "sys-temp-curve", value_enum, default_value_t = TemperatureCurveArg::default(), help_heading = "HEAT MECHANICS")]
    pub sys_temp_curve: TemperatureCurveArg,
}

impl CommonHeatConfig {
    /// Resolve the final heat constraint state.
    ///
    /// If both `avoid_critical_state` and `no_avoid_critical_state` are true,
    /// `no_avoid_critical_state` takes precedence (explicit disable wins).
    pub fn should_avoid_critical_state(&self) -> bool {
        if self.no_avoid_critical_state {
            false // Explicit disable overrides enable
        } else {
            self.avoid_critical_state
        }
    }

    /// Get the effective temperature limit based on heat config.
    ///
    /// Returns `Some(150.0)` if heat avoidance is enabled, otherwise returns
    /// the provided fallback (typically from --max-temp flag).
    pub fn effective_max_temp(&self, fallback: Option<f64>) -> Option<f64> {
        if self.should_avoid_critical_state() {
            Some(150.0) // Critical temperature threshold
        } else {
            fallback
        }
    }
}

/// Temperature calculation model selection for heat mechanics.
///
/// This enum provides two validated models for calculating system temperature
/// during routing, both with approximately 1.2K mean absolute error.
#[derive(ValueEnum, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TemperatureCurveArg {
    /// Flux-based inverse-tangent model (default). Uses radiative flux (L/d²) with
    /// inverse-square law. Physically interpretable and validated (~1.2K MAE).
    #[default]
    Flux,
    /// Logistic curve model. Empirically fitted sigmoid function.
    /// Validated alternative (~1.2K MAE).
    Logistic,
}

impl From<TemperatureCurveArg> for evefrontier_lib::temperature::TemperatureMethod {
    fn from(arg: TemperatureCurveArg) -> Self {
        match arg {
            TemperatureCurveArg::Flux => {
                evefrontier_lib::temperature::TemperatureMethod::InverseTangent
            }
            TemperatureCurveArg::Logistic => {
                evefrontier_lib::temperature::TemperatureMethod::LogisticCurve
            }
        }
    }
}

// Value parser helpers for clap f64 validation

/// Parse fuel quality, validating range 1.0-100.0
fn parse_fuel_quality(s: &str) -> Result<f64, String> {
    let val: f64 = s
        .parse()
        .map_err(|_| format!("'{}' is not a valid number", s))?;
    if !(1.0..=100.0).contains(&val) {
        return Err(format!(
            "fuel quality must be between 1 and 100, got {}",
            val
        ));
    }
    Ok(val)
}

/// Parse non-negative f64 values (for cargo_mass, fuel_load)
fn parse_non_negative(s: &str) -> Result<f64, String> {
    let val: f64 = s
        .parse()
        .map_err(|_| format!("'{}' is not a valid number", s))?;
    if val < 0.0 {
        return Err(format!("value must be non-negative, got {}", val));
    }
    Ok(val)
}
