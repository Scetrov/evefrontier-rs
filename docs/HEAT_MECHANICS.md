According to [Jump Calculators: Understanding Your Ships Heat and Fuel Limits](https://ef-map.com/blog/jump-calculators-heat-fuel-range) the math is:

```
range = fuel_volume × fuel_quality / (FUEL_CONSTANT × ship_mass)

// Where:
// FUEL_CONSTANT = 0.0000001 (1e-7)
// fuel_volume = liters (fuel_quantity × 0.28 m³/unit)
// fuel_quality = 0.1 to 0.9 (10% to 90% purity)
```