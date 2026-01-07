# Feature Spec: Cooling Time Calculation

## Overview
Implement a cooling time indicator for ship routes. The game uses Newton's Law of Cooling to model how a ship cools between jumps. The indicator should show the time required (in `mm:ss` format) to cool from the temperature resulting from a jump down to the temperature threshold required for the *next* jump.

## Requirements
- Use Newton's Law of Cooling formula: $T(t) = T_{env} + (T_0 - T_{env}) e^{-kt}$.
- Solve for $t$ given $T(t) = T_{threshold}$ (where $T_{threshold}$ is the jump limit or nominal temperature, whichever applies).
- The "previous jump" temperature is the input (starting temperature $T_0$).
- $T_{env}$ is the ambient temperature of the system.
- $k$ is the cooling constant, likely derived from ship mass and surface area/radiator efficiency.
- Output the time in `2m4s` style format.
- Integrate this into the CLI route output and Lambda responses.

## Success Criteria
- Route steps clearly display cooling time.
- Calculations match game mechanics (Newtons Law of Cooling).
- Ship data includes required parameters for $k$ calculation (or $k$ is explicitly provided).

## Technical Constraints
- Must use existing `ship.rs` and `routing.rs` modules.
- Newton's Law of Cooling requires a cooling constant $k$. I need to identify how $k$ is determined (if it's in `ship_data.csv`).
- Formula solving: $t = -\frac{1}{k} \ln\left(\frac{T_{threshold} - T_{env}}{T_0 - T_{env}}\right)$.
