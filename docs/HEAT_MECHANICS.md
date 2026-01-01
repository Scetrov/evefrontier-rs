# Heat mechanics and fuel range (summary)

This document summarizes community-sourced formulas and reasoning about ship heat and fuel
influence on jump range. It is intended as a research note and a starting point for a
formal ADR or implementation in `ship.rs` (see ADR 0015).

Summary formula (informational):

```
range = fuel_volume × fuel_quality / (FUEL_CONSTANT × ship_mass)

// Where:
// FUEL_CONSTANT = 0.0000001 (1e-7)  # example constant from community sources
// fuel_volume = liters (fuel_quantity × 0.28 m³/unit)
// fuel_quality = 0.1 to 0.9 (10% to 90% purity)
```

Notes:

- This is a community-derived formula and should be validated before use in production.
- Keep the document concise — implementation choices (units, normalization) must be documented
  in the ship data ADR before this formula is used for route fuel/heat calculations.

References
- Jump Calculators: Understanding Your Ships Heat and Fuel Limits — https://ef-map.com/blog/jump-calculators-heat-fuel-range

TODO
- Add a short table of example calculations.
- Link this doc from `docs/USAGE.md` and `docs/ADR` once the formula is agreed and tests are added.
