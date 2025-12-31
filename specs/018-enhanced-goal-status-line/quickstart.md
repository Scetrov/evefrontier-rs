# Quickstart & Validation Notes

## Regression commands

- Automated tests:
  ```bash
  cargo test -p evefrontier-cli
  ```
- Manual enhanced output check (uses fixture dataset):
  ```bash
  EVEFRONTIER_DATASET_SOURCE=docs/fixtures/minimal_static_data.db \
  EVEFRONTIER_DATASET_CACHE_DIR=/tmp/evefrontier-cache \
  evefrontier-cli --no-logo --format enhanced route --from "Nod" --to "Brana"
  ```

Expected behavior: GOAL step shows a status line (min temp, planets, moons). Black hole systems (IDs 30000001–30000003) display a "Black Hole" badge instead of temperature/planet/moon data. Fuel display should not decrease on gate hops.
