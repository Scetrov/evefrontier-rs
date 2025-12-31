//! MCP prompt templates for common EVE Frontier navigation scenarios
//!
//! Prompts provide pre-configured templates that guide AI assistants
//! through common tasks like route planning, system exploration, and
//! fleet coordination.

use crate::Result;
use serde::Serialize;

/// Prompt descriptor for MCP prompts/list
#[derive(Debug, Serialize)]
pub struct PromptDescriptor {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<PromptArgument>>,
}

/// Argument definition for prompt templates
#[derive(Debug, Serialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
}

/// Get all available prompt templates
pub fn list_prompts() -> Result<Vec<PromptDescriptor>> {
    Ok(vec![
        PromptDescriptor {
            name: "route_planning".to_string(),
            description: "Plan an optimal route between two systems with safety constraints"
                .to_string(),
            arguments: Some(vec![
                PromptArgument {
                    name: "origin".to_string(),
                    description: "Starting system name".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "destination".to_string(),
                    description: "Destination system name".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "risk_tolerance".to_string(),
                    description: "Risk level: safe (avoid hot systems), balanced, fast".to_string(),
                    required: false,
                },
            ]),
        },
        PromptDescriptor {
            name: "system_exploration".to_string(),
            description:
                "Explore a star system and identify resources, hazards, and strategic value"
                    .to_string(),
            arguments: Some(vec![PromptArgument {
                name: "system_name".to_string(),
                description: "System to explore".to_string(),
                required: true,
            }]),
        },
        PromptDescriptor {
            name: "fleet_planning".to_string(),
            description: "Plan fleet movement with multiple waypoints and rendezvous coordination"
                .to_string(),
            arguments: Some(vec![
                PromptArgument {
                    name: "fleet_origin".to_string(),
                    description: "Fleet starting position".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "waypoints".to_string(),
                    description: "Comma-separated list of waypoint systems".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "rendezvous_point".to_string(),
                    description: "Final rendezvous system".to_string(),
                    required: true,
                },
            ]),
        },
        PromptDescriptor {
            name: "safe_zone_finder".to_string(),
            description: "Find safe systems near a location with temperature constraints"
                .to_string(),
            arguments: Some(vec![
                PromptArgument {
                    name: "center_system".to_string(),
                    description: "Center of search area".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "max_range".to_string(),
                    description: "Maximum search radius in light years".to_string(),
                    required: false,
                },
            ]),
        },
    ])
}

/// Get a specific prompt template by name
pub fn get_prompt(name: &str, arguments: &serde_json::Value) -> Result<String> {
    match name {
        "route_planning" => get_route_planning_prompt(arguments),
        "system_exploration" => get_system_exploration_prompt(arguments),
        "fleet_planning" => get_fleet_planning_prompt(arguments),
        "safe_zone_finder" => get_safe_zone_finder_prompt(arguments),
        _ => Err(crate::Error::invalid_param(
            "name",
            format!("Unknown prompt: {}", name),
        )),
    }
}

fn get_route_planning_prompt(args: &serde_json::Value) -> Result<String> {
    let origin = args["origin"]
        .as_str()
        .ok_or_else(|| crate::Error::invalid_param("origin", "Missing required argument"))?;
    let destination = args["destination"]
        .as_str()
        .ok_or_else(|| crate::Error::invalid_param("destination", "Missing required argument"))?;
    let risk_tolerance = args["risk_tolerance"].as_str().unwrap_or("balanced");

    let constraints = match risk_tolerance {
        "safe" => "Use A* algorithm with max_temperature: 4000 to avoid hot systems. Prioritize gate routes for safety.",
        "balanced" => "Use A* algorithm with max_temperature: 6000. Balance speed and safety.",
        "fast" => "Use Dijkstra algorithm without temperature constraints. Optimize for shortest path.",
        _ => "Use A* algorithm with default settings.",
    };

    Ok(format!(
        r#"# Route Planning Task

Plan a route from **{origin}** to **{destination}** with {risk_tolerance} risk tolerance.

## Constraints
{constraints}

## Steps
1. Use the `route_plan` tool with appropriate algorithm and constraints
2. Analyze the route for:
   - Total distance and jump count
   - Systems with high temperatures (>5000K)
   - Alternative routing options
   - Potential hazards or bottlenecks
3. Provide a summary with:
   - Recommended route with waypoints
   - Total travel time estimate
   - Safety considerations
   - Alternative routes if available

## Expected Output
Provide a clear, actionable route plan with system names, distances, and safety notes."#,
        origin = origin,
        destination = destination,
        risk_tolerance = risk_tolerance,
        constraints = constraints
    ))
}

fn get_system_exploration_prompt(args: &serde_json::Value) -> Result<String> {
    let system_name = args["system_name"]
        .as_str()
        .ok_or_else(|| crate::Error::invalid_param("system_name", "Missing required argument"))?;

    Ok(format!(
        r#"# System Exploration: {system_name}

Conduct a comprehensive exploration and analysis of the **{system_name}** star system.

## Analysis Checklist

1. **System Information** (use `system_info` tool)
   - Coordinates and spatial location
   - External temperature (habitability indicator)
   - Planetary composition (planets and moons)

2. **Connectivity Analysis** (use `gates_from` tool)
   - Jump gate connections
   - Strategic chokepoint assessment
   - Accessibility from major hubs

3. **Neighborhood Analysis** (use `systems_nearby` tool)
   - Systems within 50 light years
   - Temperature distribution in region
   - Resource cluster identification

4. **Strategic Value Assessment**
   - Trade route potential (gate connectivity)
   - Resource extraction viability (moons count)
   - Defensibility (isolation, gate count)
   - Hazard level (temperature)

## Expected Output
Provide a comprehensive report with:
- System specifications and habitability
- Strategic importance rating (1-10)
- Recommended uses (mining, trading, staging, etc.)
- Risks and considerations"#,
        system_name = system_name
    ))
}

fn get_fleet_planning_prompt(args: &serde_json::Value) -> Result<String> {
    let fleet_origin = args["fleet_origin"]
        .as_str()
        .ok_or_else(|| crate::Error::invalid_param("fleet_origin", "Missing required argument"))?;
    let waypoints = args["waypoints"]
        .as_str()
        .ok_or_else(|| crate::Error::invalid_param("waypoints", "Missing required argument"))?;
    let rendezvous = args["rendezvous_point"].as_str().ok_or_else(|| {
        crate::Error::invalid_param("rendezvous_point", "Missing required argument")
    })?;

    Ok(format!(
        r#"# Fleet Movement Planning

Coordinate fleet movement from **{fleet_origin}** through waypoints to **{rendezvous}**.

## Waypoints
{waypoints}

## Planning Steps

1. **Route Calculation**
   - Use `route_plan` for each leg of the journey
   - Optimize for fleet cohesion (all ships take same route)
   - Identify safe staging points between waypoints

2. **Timing Coordination**
   - Calculate travel time for each leg
   - Identify holding positions for synchronization
   - Plan communication checkpoints

3. **Risk Assessment**
   - Identify high-temperature systems (>6000K)
   - Check for route bottlenecks or single points of failure
   - Plan alternative routes for contingencies

4. **Logistics Planning**
   - Fuel requirements estimation
   - Identify resupply points
   - Emergency extraction routes

## Expected Output
Provide a fleet movement plan with:
- Detailed route for each leg with distances
- Estimated travel times and staging points
- Risk mitigation strategies
- Communication and coordination protocol
- Emergency contingency plans"#,
        fleet_origin = fleet_origin,
        waypoints = waypoints,
        rendezvous = rendezvous
    ))
}

fn get_safe_zone_finder_prompt(args: &serde_json::Value) -> Result<String> {
    let center = args["center_system"]
        .as_str()
        .ok_or_else(|| crate::Error::invalid_param("center_system", "Missing required argument"))?;
    let max_range = args["max_range"].as_f64().unwrap_or(100.0);

    Ok(format!(
        r#"# Safe Zone Identification

Find safe systems near **{center}** within {max_range} light years.

## Safety Criteria
- Temperature ≤ 4000K (habitable zone)
- Accessible via gate or spatial jump
- Resource potential (moons for mining)
- Defensible location (limited gate connections)

## Analysis Steps

1. **Initial Scan** (use `systems_nearby` tool)
   - Search radius: {max_range} ly
   - Apply max_temperature: 4000 filter

2. **System Evaluation** (use `system_info` for each candidate)
   - Check temperature (lower is safer)
   - Count moons (more = better resources)
   - Assess isolation (fewer gates = more defensible)

3. **Connectivity Check** (use `gates_from` for each candidate)
   - Identify escape routes
   - Check for strategic positioning
   - Assess trade route access

4. **Ranking and Recommendations**
   - Rank systems by safety score
   - Consider resource potential
   - Evaluate strategic value

## Expected Output
Provide a ranked list of safe systems with:
- System name and coordinates
- Temperature and habitability rating
- Resource potential (moons count)
- Accessibility (distance from center, gate connections)
- Safety score (1-10)
- Recommended use (base, mining, trading post, etc.)"#,
        center = center,
        max_range = max_range
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_list_prompts() {
        let prompts = list_prompts().unwrap();
        assert_eq!(prompts.len(), 4);

        let names: Vec<_> = prompts.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"route_planning"));
        assert!(names.contains(&"system_exploration"));
        assert!(names.contains(&"fleet_planning"));
        assert!(names.contains(&"safe_zone_finder"));
    }

    #[test]
    fn test_route_planning_prompt() {
        let args = json!({
            "origin": "Nod",
            "destination": "Brana",
            "risk_tolerance": "safe"
        });

        let prompt = get_route_planning_prompt(&args).unwrap();
        assert!(prompt.contains("Nod"));
        assert!(prompt.contains("Brana"));
        assert!(prompt.contains("safe"));
        assert!(prompt.contains("max_temperature: 4000"));
    }

    #[test]
    fn test_system_exploration_prompt() {
        let args = json!({ "system_name": "Nod" });
        let prompt = get_system_exploration_prompt(&args).unwrap();
        assert!(prompt.contains("Nod"));
        assert!(prompt.contains("system_info"));
        assert!(prompt.contains("Strategic Value"));
    }

    #[test]
    fn test_fleet_planning_prompt() {
        let args = json!({
            "fleet_origin": "Nod",
            "waypoints": "Brana, H:2L2S",
            "rendezvous_point": "D:2NAS"
        });

        let prompt = get_fleet_planning_prompt(&args).unwrap();
        assert!(prompt.contains("Nod"));
        assert!(prompt.contains("Brana"));
        assert!(prompt.contains("D:2NAS"));
        assert!(prompt.contains("Fleet Movement"));
    }

    #[test]
    fn test_safe_zone_finder_prompt() {
        let args = json!({
            "center_system": "Nod",
            "max_range": 75.0
        });

        let prompt = get_safe_zone_finder_prompt(&args).unwrap();
        assert!(prompt.contains("Nod"));
        assert!(prompt.contains("75"));
        assert!(prompt.contains("4000K"));
    }

    #[test]
    fn test_missing_required_argument() {
        let args = json!({ "destination": "Brana" }); // Missing origin
        let result = get_route_planning_prompt(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_prompt() {
        let args = json!({});
        let result = get_prompt("nonexistent", &args);
        assert!(result.is_err());
    }
}
