# Research: Newton's Law of Cooling for EVE Frontier

## Cooling Model
The game uses Newton's Law of Cooling, which describes the rate of change of temperature of an object as proportional to the difference between its own temperature and the ambient temperature.

**Differential Equation**:
$$\frac{dT}{dt} = -k(T - T_{env})$$

**Solution**:
$$T(t) = T_{env} + (T_0 - T_{env}) e^{-kt}$$

**Solving for Cooldown Time $t$**:
$$t = -\frac{1}{k} \ln\left(\frac{T_{threshold} - T_{env}}{T_0 - T_{env}}\right)$$

## Parameters

### 1. Temperature Threshold ($T_{threshold}$)
- **Nominal Temperature**: `HEAT_NOMINAL` (30.0 K) is the standard resting state.
- **Jump Constraint**: A ship cannot jump if it exceeds the nominal state (or a system-specific threshold if implemented).
- **Default**: 30.0 K.

### 2. Initial Temperature ($T_0$)
- **Ambient + Jump Heat**: $T_0 = T_{ambient} + \Delta T_{jump}$.
- $T_{ambient}$ is the `min_external_temp` of the system.
- $\Delta T_{jump}$ is calculated using `calculate_jump_heat` and ship thermal mass.

### 3. Ambient Temperature ($T_{env}$)
- **Minimum External Temperature**: The coldest temperature in the system's habitable zone (`min_external_temp`).

### 4. Cooling Constant ($k$)
- **Derivation**: $k$ scales with cooling power and inversely with thermal mass.
- In `ship.rs`, we have `BASE_COOLING_POWER` and `compute_zone_factor`.
- We will define $k$ as:
  $$k = \frac{BASE\_COOLING\_POWER \times zone\_factor}{total\_mass\_kg \times specific\_heat}$$
  In the final implementation, the conceptual `SCALE` factor discussed during research was absorbed into the calibrated value of `BASE_COOLING_POWER`. To retune cooldown behavior, adjust `BASE_COOLING_POWER` (and/or `compute_zone_factor`) rather than introducing an additional scale parameter.

## Implementation Details
- **Formatting**: A utility function `format_cooldown(seconds: f64) -> String` to return `2m4s`.
- **Integration**: Add a `cooldown_time_seconds` field to `HeatProjection` (replacing or augmenting `wait_time_seconds`).
- **Edge Cases**:
  - $T_0 \le T_{threshold}$: Cooldown is 0.
  - $T_{env} \ge T_{threshold}$: The ship will never cool to threshold! (Return infinity or a warning). In EVE Frontier, $T_{env}$ is usually < 30K except in very hot systems.

## Rationale
Newton's Law of Cooling is more realistic than the current linear model as it mimics the diminishing return of cooling as the object approaches ambient temperature. This is crucial for high-heat jumps where the initial cooling is fast but the final approach to nominal is slow.

## Alternatives Considered
- **Linear Model**: Existing implementation use `wait_time = delta_T / rate`. Rejected because the user specifically requested Newton's Law of Cooling.
- **Constant Time**: Fixed cooldown regardless of heat. Rejected for lack of realism and non-compliance with game mechanics.
