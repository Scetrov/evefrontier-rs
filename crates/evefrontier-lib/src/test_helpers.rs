// Test-only helpers for `evefrontier-lib` tests
#![allow(dead_code)]
#[cfg(test)]
use crate::output::RouteStep;
use crate::ship::FuelProjection;

/// Builder to create `RouteStep` instances in tests with sensible defaults.
pub struct RouteStepBuilder {
    step: RouteStep,
}

impl RouteStepBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            step: RouteStep {
                index: 1,
                id: 1,
                name: Some("Step 1".to_string()),
                distance: Some(1.0),
                method: Some("jump".to_string()),
                min_external_temp: None,
                planet_count: None,
                moon_count: None,
                fuel: None,
                heat: None,
            },
        }
    }

    pub fn index(mut self, idx: usize) -> Self {
        self.step.index = idx;
        self
    }

    pub fn id(mut self, id: i64) -> Self {
        self.step.id = id;
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.step.name = Some(name.to_string());
        self
    }

    pub fn distance(mut self, d: f64) -> Self {
        self.step.distance = Some(d);
        self
    }

    pub fn method(mut self, method: &str) -> Self {
        self.step.method = Some(method.to_string());
        self
    }

    pub fn min_temp(mut self, t: f64) -> Self {
        self.step.min_external_temp = Some(t);
        self
    }

    pub fn planets(mut self, n: u32) -> Self {
        self.step.planet_count = Some(n);
        self
    }

    pub fn moons(mut self, n: u32) -> Self {
        self.step.moon_count = Some(n);
        self
    }

    pub fn fuel(mut self, hop_cost: f64, cumulative: f64, remaining: Option<f64>) -> Self {
        self.step.fuel = Some(FuelProjection {
            hop_cost,
            cumulative,
            remaining,
            warning: None,
        });
        self
    }

    pub fn with_fuel_projection(mut self, p: FuelProjection) -> Self {
        self.step.fuel = Some(p);
        self
    }

    pub fn build(self) -> RouteStep {
        self.step
    }
}

impl Default for RouteStepBuilder {
    fn default() -> Self {
        Self::new()
    }
}
