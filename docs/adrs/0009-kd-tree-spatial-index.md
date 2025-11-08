# ADR 0009: Precompute K‑D Tree spatial index for nearest-neighbour & spatial routing

## Status

Proposed

## Context

Many features (CLI route suggestions, Lambda endpoints that answer "nearest systems" or
spatial jump planning) require efficient nearest-neighbour queries in 2D/3D space for all
solar-system nodes. Running an O(n) scan across thousands of systems on every request adds
latency and higher CPU cost, especially for Lambda cold-starts and latency-sensitive CLI
interactions.

To improve performance and keep response times predictable, we should precompute a spatial index
over system coordinates (K‑D Tree) when preparing the dataset. The precomputed index can be
stored alongside the dataset and loaded quickly by the CLI or Lambda at startup.

## Decision

Precompute a static K‑D Tree spatial index for the starmap at dataset preparation time and ship
it with the dataset release artifacts. On startup, the library will memory-map or deserialize the
index into an in-memory structure that supports fast nearest-neighbour queries and radius searches.

Key points:

- Build the K‑D Tree once when the dataset is produced (download pipeline or a local build step).
- Serialize the index into a compact, versioned binary format (or use an interoperable format)
  and include it in the cached dataset (alongside `static_data.db`).
- Provide a small, idiomatic Rust API in `evefrontier-lib` to load the index and run queries.
- For Lambdas, load the index into the cold-start initializer so handlers can use it without
  recomputing.
- Fall back to a simple in-memory scan if the index is missing, but log a warning and
  recommend re-generating the dataset index.

## Rationale

- Query performance: K‑D Trees provide O(log n) average-case nearest-neighbour queries for
  low-dimensional data (2D/3D), which is sufficient for our use-case and keeps memory use low.
- Cold-start: deserializing a precomputed index is far cheaper than rebuilding it in Lambda cold
  starts.
- Reproducibility: building the index at dataset-prep time ensures deterministic indexing and
  reduces platform-specific differences.

## Approach

1. Data representation
   - Use Cartesian coordinates for systems (x, y, z). If the dataset stores spherical or
     different coordinates, convert to a consistent 3D Cartesian system during index build.
   - Each indexed point stores: system id (integer), x, y, z (f32 by default), and optionally
     a name index for quick debugging. f32 is chosen for compactness; support f64 as an opt-in
     only when necessary.

2. Index structure
   - Use a K‑D Tree (k = 2 or 3 depending on available coordinates). The index must support:
     - k-nearest neighbours (k-NN)
     - radius (ball) queries
     - optional bounding-box queries

3. Serialization
   - Serialize the tree into a versioned binary format. Chosen defaults:
     - serialization: `postcard` + `serde` for compact, deterministic binary encoding
     - compression: `zstd` for good ratio and fast decompression in Lambda
     - header: include magic, version, compression flag, uncompressed size and SHA-256 checksum
     - coordinate precision recorded in header (f32 by default)
   - Use a compact, fast deserialization strategy (minimize heap allocations) so loading is fast
     in constrained environments.

4. Integration points
   - Add an index builder CLI subcommand (or a separate small tool) that reads the DB, builds the
     K‑D Tree, and writes the index file next to the `static_data.db` (or the checked-in
     `minimal_static_data.db` fixture) in the release bundle or cache.
   - Extend `ensure_c3e6_dataset` to look for and return the index path together with the DB
     path, or provide a `load_spatial_index` function in `evefrontier-lib`.
   - Lambda initializer should attempt to load the index via `load_spatial_index` and fall back
     as described above.

## Libraries and tooling

Primary implementation target: Rust (library crate `evefrontier-lib`). We recommend these
current, well-maintained crates for building and using K‑D Trees and serialization.

- kd-tree crate (KD-tree implementation)
  - Option: `kiddo` — fast k-d tree implementation for Rust supporting k-NN and radius queries.
  - Crate: [kiddo on crates.io](https://crates.io/crates/kiddo)
  - Repository: [stainless-steel/kiddo on GitHub](https://github.com/stainless-steel/kiddo)
    - Rationale: small, optimized for k-NN with float coordinates. Last checked (2025-11):
      actively maintained with recent Rust editions support.

- Alternative: `kdtree` crate
  - Crate: [kdtree on crates.io](https://crates.io/crates/kdtree)
  - Repository: [kojix2/kdtree-rs on GitHub](https://github.com/kojix2/kdtree-rs)
  - Rationale: Stable and simple; choose if `kiddo` doesn't meet serialization or API needs.

- Serialization
  - Use `bincode` for compact, fast binary serialization in Rust:
  - Crate: [bincode on crates.io](https://crates.io/crates/bincode)
  - Repo: [bincode-org/bincode on GitHub](https://github.com/bincode-org/bincode)
    - Rationale: fast, portable for same-endian hosts; include versioning in header for safety.
  - Alternately, `postcard` (tiny, no-std friendly) could be considered for even smaller payloads.

- Checksums
  - Use `sha2`/`digest` crates to add an optional checksum field to the serialized artifact.

Client-side (Lambda / CLI)

- Both the CLI (Rust) and Lambda (Rust-based or other runtimes) should load the same index
  format via `evefrontier-lib`.
- For non-Rust Lambdas (for example Node.js handlers), consider producing a companion JSON or
  protobuf index in the release bundle, or provide a tiny WASM-compiled reader that can be used
  cross-platform. However, prefer Rust Lambdas or invoking a small Rust loader process for
  performance and consistency.

## API sketch (Rust)

```rust
// load or return error if missing
fn load_spatial_index(path: &Path) -> Result<SpatialIndex>

impl SpatialIndex {
    fn nearest(&self, point: [f64; 3], k: usize) -> Vec<(SystemId, f64)>; // id and distance
    fn within_radius(&self, point: [f64; 3], r: f64) -> Vec<(SystemId, f64)>;
}
```

## Migration & backward compatibility

- When releasing a new dataset, publish the index with a matching schema version. If a
  consumer finds a dataset without an index, fall back to in-memory scan and emit a log with a
  recommendation to re-generate the dataset index.
- Provide a small `index-version` file or header in the serialized index so loaders can detect
  incompatible changes and fail fast with a clear message.

## Performance considerations

- Choose coordinate precision (f32 vs f64) based on the dataset's numeric fidelity and memory
  constraints — f32 often suffices and reduces size by half.
- Benchmark tree build time and query latency on representative hardware and on Lambda's
  constrained environment (memory/CPU) to select defaults.

## Drawbacks

- Adds a dataset build step and a small binary artifact to releases.
- Requires careful versioning to avoid silent incompatibilities between index and DB.

## Alternatives considered

- R-tree: better for bounding-box queries and spatial indexing for variable-sized objects, but
  for point-based nearest-neighbour in low dimensions a K‑D Tree is simpler and faster.
- VP-tree / Ball-tree: designed for generic metric spaces; overkill for low-dimensional Euclidean
  coordinates.

## References & citations

- kiddo crate — KD-tree implementation for Rust. [crates.io](https://crates.io/crates/kiddo)
- kdtree crate — alternative KD-tree implementation. [crates.io](https://crates.io/crates/kdtree)
- bincode crate — binary serialization for Rust. [crates.io](https://crates.io/crates/bincode)
- postcard crate — tiny binary serialization, alternative. [crates.io](https://crates.io/crates/postcard)
- sha2 crate — hashing for checksums. [crates.io](https://crates.io/crates/sha2)

## Implementation plan

1. Prototype: implement a small index builder in `crates/evefrontier-lib/examples` that reads the
  checked-in fixture `minimal_static_data.db`, converts coordinates, builds a `kiddo` tree,
  serializes with `postcard`, compresses with `zstd`, and writes `spatial_index.bin`.
2. Add a `cargo run -p evefrontier-cli -- index-build` subcommand that runs the builder locally
  and writes the index into the cache dir used by `ensure_c3e6_dataset`.
2b. Release packaging: extend the release (GitHub Actions) pipeline to run `index-build` during
   artifact preparation and attach `spatial_index.bin` and a `spatial_index.meta` JSON (version,
   precision, compression, checksum) to the GitHub release. CI should also publish or cache the
   artifact for downstream jobs and provide a checksum verification step in the release workflow.
3. Update `ensure_c3e6_dataset` to prefer the index alongside the DB and add `load_spatial_index`
  to the public API.
4. Add unit tests using `minimal_static_data.db` to validate nearest-neighbour and radius queries
  and to assert round-trip integrity (checksum) of the serialized index.
5. Measure performance (build time, cold-start load time and query latency) and tune defaults
  (coordinate precision, serialization, compression level).

## Status and next steps

- Proposed: gather feedback and pick the concrete crate (`kiddo` vs `kdtree`) after a tiny
  prototype. The prototype should confirm serialization layout and cold-start deserialization
  times on Lambda.
