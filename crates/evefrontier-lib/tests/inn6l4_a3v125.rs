use std::collections::HashMap;

use evefrontier_lib::db::{Starmap, System, SystemId, SystemMetadata, SystemPosition};
use evefrontier_lib::graph::{build_hybrid_graph_indexed, GraphBuildOptions};
use evefrontier_lib::routing::{plan_route, RouteAlgorithm, RouteConstraints, RouteRequest};
use evefrontier_lib::ship::{FuelConfig, ShipAttributes, ShipLoadout};

/// Build a small, deterministic in-memory starmap that reproduces the reported
/// behaviour: distance optimization prefers a short spatial hop, while fuel
/// optimization prefers a gate-connected path (gate hops cost zero fuel).
fn build_test_starmap() -> Starmap {
    // IDs chosen arbitrarily
    let inn: SystemId = 1;
    let a3v: SystemId = 2;
    let gate1: SystemId = 3;

    let mut systems = HashMap::new();

    systems.insert(
        inn,
        System {
            id: inn,
            name: "INN-6L4".to_string(),
            metadata: SystemMetadata {
                ..SystemMetadata {
                    constellation_id: None,
                    constellation_name: None,
                    region_id: None,
                    region_name: None,
                    security_status: None,
                    star_temperature: None,
                    star_luminosity: None,
                    min_external_temp: None,
                    planet_count: None,
                    moon_count: None,
                }
            },
            position: SystemPosition::new(0.0, 0.0, 0.0),
        },
    );

    systems.insert(
        a3v,
        System {
            id: a3v,
            name: "A3V-125".to_string(),
            metadata: SystemMetadata {
                ..SystemMetadata {
                    constellation_id: None,
                    constellation_name: None,
                    region_id: None,
                    region_name: None,
                    security_status: None,
                    star_temperature: None,
                    star_luminosity: None,
                    min_external_temp: None,
                    planet_count: None,
                    moon_count: None,
                }
            },
            // Place far enough that direct spatial hop is shorter than gate chain by distance
            position: SystemPosition::new(100.0, 0.0, 0.0),
        },
    );

    // Gate node placed far away so distance-opt prefers direct spatial hop but
    // fuel-opt will prefer gate hops (which cost zero fuel in optimizer)
    systems.insert(
        gate1,
        System {
            id: gate1,
            name: "GATE-1".to_string(),
            metadata: SystemMetadata {
                ..SystemMetadata {
                    constellation_id: None,
                    constellation_name: None,
                    region_id: None,
                    region_name: None,
                    security_status: None,
                    star_temperature: None,
                    star_luminosity: None,
                    min_external_temp: None,
                    planet_count: None,
                    moon_count: None,
                }
            },
            position: SystemPosition::new(1000.0, 0.0, 0.0),
        },
    );

    let mut name_to_id = HashMap::new();
    name_to_id.insert("INN-6L4".to_string(), inn);
    name_to_id.insert("A3V-125".to_string(), a3v);
    name_to_id.insert("GATE-1".to_string(), gate1);

    // Gate adjacency (directed edges as jumps)
    let mut adjacency: HashMap<SystemId, Vec<SystemId>> = HashMap::new();
    adjacency.insert(inn, vec![gate1]);
    adjacency.insert(gate1, vec![a3v]);
    adjacency.insert(a3v, vec![]);

    Starmap {
        systems,
        name_to_id,
        adjacency: std::sync::Arc::new(adjacency),
    }
}

// helper removed: not needed for final test assertions

/// Same as `plan_distances` but zeroes distances for gate hops so they are treated
/// as zero-fuel transitions when projecting fuel consumption.
// plan_distances_for_fuel removed: fuel projection handles gates as zero-cost explicitly

#[test]
fn inn6l4_to_a3v125_distance_vs_fuel_optimization() {
    let starmap = build_test_starmap();

    // Distance-optimized plan (no ship provided)
    let request_distance = RouteRequest {
        start: "INN-6L4".to_string(),
        goal: "A3V-125".to_string(),
        algorithm: RouteAlgorithm::AStar,
        constraints: RouteConstraints::default(),
        spatial_index: None,
        max_spatial_neighbors: GraphBuildOptions::default().max_spatial_neighbors,
        optimization: evefrontier_lib::routing::RouteOptimization::Distance,
        fuel_config: evefrontier_lib::ship::FuelConfig::default(),
    };

    let plan_distance = plan_route(&starmap, &request_distance).expect("distance plan exists");

    // Fuel-optimized plan (requires ship/loadout)
    // Construct a minimal ShipAttributes-like struct for testing
    let ship = ShipAttributes {
        name: "TestShip".to_string(),
        base_mass_kg: 80000.0,
        specific_heat: 1000.0,
        fuel_capacity: 2000.0,
        cargo_capacity: 100.0,
    };
    let loadout = ShipLoadout::new(&ship, 1750.0, 0.0).expect("valid loadout");
    let fuel_cfg = FuelConfig {
        quality: 10.0,
        dynamic_mass: true,
    };

    let constraints_fuel = RouteConstraints {
        ship: Some(ship.clone()),
        loadout: Some(loadout),
        ..Default::default()
    };

    let request_fuel = RouteRequest {
        start: "INN-6L4".to_string(),
        goal: "A3V-125".to_string(),
        algorithm: RouteAlgorithm::AStar,
        constraints: constraints_fuel,
        spatial_index: None,
        max_spatial_neighbors: GraphBuildOptions::default().max_spatial_neighbors,
        optimization: evefrontier_lib::routing::RouteOptimization::Fuel,
        fuel_config: fuel_cfg,
    };

    let plan_fuel = plan_route(&starmap, &request_fuel).expect("fuel plan exists");

    // Reconstruct the hybrid graph used by the planner and compute per-hop distances
    let graph = build_hybrid_graph_indexed(&starmap, &GraphBuildOptions::default());

    // per-hop distances are available via `plan_distances` if needed for debugging
    // Compute total fuel for each plan while treating gate hops as zero-fuel.
    fn total_fuel_for_plan(
        ship: &ShipAttributes,
        loadout: &ShipLoadout,
        plan: &evefrontier_lib::routing::RoutePlan,
        graph: &evefrontier_lib::graph::Graph,
        fuel_cfg: &FuelConfig,
    ) -> f64 {
        let mut cumulative = 0.0;
        let mut dynamic_fuel = loadout.fuel_load;
        for pair in plan.steps.windows(2) {
            let u = pair[0];
            let v = pair[1];
            let chosen = graph
                .neighbours(u)
                .iter()
                .filter(|e| e.target == v)
                .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap())
                .expect("edge present for hop");

            if chosen.kind == evefrontier_lib::graph::EdgeKind::Gate {
                // Gate hops cost zero fuel
                continue;
            }

            let effective_fuel = if fuel_cfg.dynamic_mass {
                dynamic_fuel
            } else {
                loadout.fuel_load
            };

            let mass = ship.base_mass_kg
                + loadout.cargo_mass_kg
                + (effective_fuel * evefrontier_lib::ship::FUEL_MASS_PER_UNIT_KG);
            let hop_cost =
                evefrontier_lib::ship::calculate_jump_fuel_cost(mass, chosen.distance, fuel_cfg)
                    .expect("valid hop fuel");
            cumulative += hop_cost;
            if fuel_cfg.dynamic_mass {
                dynamic_fuel = (dynamic_fuel - hop_cost).max(0.0);
            }
        }
        cumulative
    }

    let total_distance_fuel =
        total_fuel_for_plan(&ship, &loadout, &plan_distance, &graph, &fuel_cfg);
    let total_fuel_fuel = total_fuel_for_plan(&ship, &loadout, &plan_fuel, &graph, &fuel_cfg);

    // Distance plan should be a direct spatial hop (one jump, zero gates)
    assert_eq!(plan_distance.jumps, 1);
    assert_eq!(plan_distance.gates, 0);

    // Fuel plan should prefer the gate chain (zero fuel hops) and therefore have at least
    // one gate hop and lower or equal total fuel.
    assert!(
        plan_fuel.gates >= 1,
        "expected fuel plan to include gate hops"
    );
    assert!(
        total_fuel_fuel <= total_distance_fuel + 1e-9,
        "fuel-optimized plan uses more fuel ({} > {})",
        total_fuel_fuel,
        total_distance_fuel
    );
}
