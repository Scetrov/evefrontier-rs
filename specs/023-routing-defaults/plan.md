# Implementation Plan - Sensible Routing Defaults

Update the EVE Frontier CLI, Library, and Lambda functions to use more sensible default routing parameters based on common use cases (Reflex ship, fuel optimization, and safety).

## User Review Required

> [!IMPORTANT]
> - Critical state avoidance will now be ENABLED by default. Users must use `--no-avoid-critical-state` (CLI) or set the corresponding field to `false` (API/JSON) to disable it.
| Flag | New Default | Previous Default |
| :--- | :--- | :--- |
| `--format` | `enhanced` | `enhanced` (No change, confirmed in code) |
| `--ship` | `Reflex` | `None` |
| `--fuel-quality` | `10.0` | `10.0` (No change) |
| `--avoid-critical-state` | `true` | `false` |
| `--optimize` | `fuel` | `distance` |
| `--max-spatial-neighbours` | `250` | `0` (unlimited) |

- **Verification Needed**: confirmed that `Reflex` is a valid ship in `ship_data.csv`.

## Proposed Changes

### Library (`evefrontier-lib`)

#### [routing.rs]
- Update `RouteOptimization` enum to derive `Default` with `Fuel` as the default variant.
- Update `RouteConstraints::default()` to set `avoid_critical_state: true`.
- Update `RouteRequest` constructors to use these new defaults.

#### [graph.rs]
- Update `DEFAULT_MAX_SPATIAL_NEIGHBORS` from `0` to `250`.

### CLI (`evefrontier-cli`)

#### [main.rs]
- Update `RouteOptionsArgs` struct:
    - Set `ship` default value to `"Reflex"`.
    - Set `optimize` default to `fuel`.
    - Set `max_spatial_neighbours` default to `250`.
    - Change `avoid_critical_state` to a bool that defaults to `true`.
    - Add `--no-avoid-critical-state` flag to allow disabling the default.
    - Ensure `RouteCommandArgs::to_request` correctly maps the ship name even when using the default.

### Lambda Shared (`evefrontier-lambda-shared`)

#### [requests.rs]
- Update `RouteRequest` struct:
    - Add `#[serde(default = "default_ship")]` for `ship` field (returning `"Reflex"`).
    - Add `#[serde(default = "default_true")]` for `avoid_critical_state` if it exists in the DTO, or ensure it's handled in the conversion to Lib types.
    - Set `max_spatial_neighbors` default to `250` in the DTO parser.

## Verification Plan

### Automated Tests
- **Unit Tests**:
    - Verify `RouteRequest::default()` in `evefrontier-lib` has the correct values.
    - Verify CLI arg parsing defaults via `RouteCommandArgs::derive()`.
    - Verify Lambda request deserialization defaults.
- **Integration Tests**:
    - Run `evefrontier-cli route "Nod" "Brana"` and assert the output includes fuel projections and uses enhanced format.
    - Test overriding defaults: `--ship "None"`, `--optimize distance`, `--no-avoid-critical-state`.

### Manual Verification
- Run CLI with `--help` to ensure default values are correctly documented in the help text.
- Invoke a local Lambda handler with minimal JSON (only `from`/`to`) and check if defaults are applied.
