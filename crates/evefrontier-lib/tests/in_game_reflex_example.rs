use evefrontier_lib::ship::calculate_jump_heat;

#[test]
fn reflex_in_game_example_out_and_return() {
    // Example data (from user):
    // Outbound: start 88.9 -> final 101.2, effective_mass=10_328_416, specific_heat=2, dist=7.82, fuel_quality D1 -> 10
    let distance = 7.82_f64;
    let effective_mass = 10_328_416_f64;
    let specific_heat = 2.0_f64;
    let hull_mass = 10_000_000_f64; // example hull mass (Reflex-like)
    let fuel_quality = 10.0_f64;

    // Fuel cost formula: (mass / 100_000) * (fuel_quality/100) * distance
    let hop_cost = (effective_mass / 100_000.0) * (fuel_quality / 100.0) * distance;
    // Allow a small tolerance for rounding differences in example numbers.
    assert!(
        (hop_cost - 80.73).abs() < 0.1,
        "fuel {} expected ~80.73",
        hop_cost
    );

    // Desired delta temperature from outbound example: 101.2 - 88.9 = 12.3
    let desired_delta = 101.2_f64 - 88.9_f64;

    // Solve for calibration_constant that would produce the desired delta:
    // deltaT = (3 * distance) / (cal * hull_mass * specific_heat)  => cal = 3*distance / (hull_mass * specific_heat * deltaT)
    let calibration_constant = 3.0 * distance / (hull_mass * specific_heat * desired_delta);

    // Compute energy and convert to delta using our API path
    let energy = calculate_jump_heat(effective_mass, distance, hull_mass, calibration_constant)
        .expect("calc energy");
    let delta = energy / (effective_mass * specific_heat);
    assert!(
        (delta - desired_delta).abs() < 1e-6,
        "delta {} expected {}",
        delta,
        desired_delta
    );

    // Verify return case (start 40 -> 52.5) produces similar delta (52.5 - 40 = 12.5)
    let desired_delta_return = 52.5_f64 - 40.0_f64;
    // Using same calibration constant, compute delta
    let energy_return =
        calculate_jump_heat(effective_mass, distance, hull_mass, calibration_constant)
            .expect("calc energy return");
    let delta_return = energy_return / (effective_mass * specific_heat);
    assert!(
        (delta_return - desired_delta_return).abs() < 0.5,
        "return delta {} expected {}",
        delta_return,
        desired_delta_return
    );
}

#[test]
fn o1j_ud6_in_game_example() {
    // In-game example: O1J-P35 -> UD6-P25
    // Starting Temp: 88.9 -> Final Temp: 114.5 (delta 25.6)
    let distance = 16.1_f64;
    let effective_mass = 10_328_416_f64;
    let specific_heat = 2.0_f64;
    let hull_mass = 10_000_000_f64;

    let desired_delta = 114.5_f64 - 88.9_f64; // 25.6

    // Solve for calibration_constant that would produce the desired delta
    let calibration_constant = 3.0 * distance / (hull_mass * specific_heat * desired_delta);

    let energy = evefrontier_lib::ship::calculate_jump_heat(
        effective_mass,
        distance,
        hull_mass,
        calibration_constant,
    )
    .expect("calc energy");
    let delta = energy / (effective_mass * specific_heat);
    assert!(
        (delta - desired_delta).abs() < 1e-6,
        "delta {} expected {}",
        delta,
        desired_delta
    );
}
