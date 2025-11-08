# evefrontier-pathfinder

A Rust executable that:

1. Downloads the latest `evefrontier_datasets` release from GitHub (if not already cached).
2. Extracts / locates the `c3e6` starmap SQLite database.
3. Loads solar systems and gate jumps.
4. Builds a gate graph.
5. Computes a greedy "optimal" path starting from a given system name that:
   - visits all reachable systems at least once
   - returns to the starting system
   - prints the route in EVE-style `<a href="showinfo:5//ID">NAME</a>` format, marking duplicates
     with `D`.

## Usage

```bash
cargo run --release -- "P:STK3"
```

or another system name, e.g.:

```bash
cargo run --release -- "O.5CD.XNS"
```

The binary will:

- Cache the downloaded release in your OS user cache directory (e.g. `~/.cache/evefrontier_datasets`
  on Linux).
- Reuse the cached asset and extracted DB on subsequent runs.

## Building

```bash
cargo build --release
```

The resulting executable will be at:

- `target/release/evefrontier-pathfinder` (Linux/macOS)
- `target/release/evefrontier-pathfinder.exe` (Windows)
