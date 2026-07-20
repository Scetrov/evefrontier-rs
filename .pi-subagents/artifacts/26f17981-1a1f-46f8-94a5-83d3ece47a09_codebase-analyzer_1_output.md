## Analysis: OpenSSF Fuzzing Finding

### Overview
The OpenSSF Fuzzing finding is substantiated: no cargo-fuzz/libFuzzer harness, property-test framework, or fuzzing CI integration was found. This matters because the Rust code accepts compressed tokens, ZIP datasets, spatial-index binaries, and network-facing route parameters.

Existing tests are conventional unit/integration tests; `docs/TESTING.md:272` explicitly lists property-based graph testing as unfinished.

### Existing Coverage
- No fuzz/property dependencies are declared in `crates/evefrontier-lib/Cargo.toml:10-30` or root workspace dependencies (`Cargo.toml:86-105`).
- No fuzz target directory or `cargo-fuzz`, `proptest`, `quickcheck`, or `arbitrary` usage was found.
- Existing negative coverage includes:
  - fmap invalid Base64/truncation examples: `crates/evefrontier-lib/tests/fmap_decode.rs:72-121`.
  - spatial-index checksum corruption and large `k`: `crates/evefrontier-lib/tests/spatial_index.rs:214-309`.
  - normal ZIP extraction only: `crates/evefrontier-lib/tests/dataset_download.rs:50-80`.
  - request-field validation: `crates/evefrontier-service-shared/src/request.rs:75-108,155-195` and `crates/evefrontier-lambda-shared/src/requests.rs:133-207,251-291`.

## High-Value Targets

### 1. Spatial-index binary loader — **High**
**Harness boundary:** `SpatialIndex::load_from_bytes(&[u8])` and `SpatialIndex::load(&Path)`.

**Data flow**
1. Services load `{database}.spatial.bin` through `AppState::load()` at `crates/evefrontier-service-shared/src/state.rs:89-106`.
2. `try_load_spatial_index()` calls `SpatialIndex::load()` at `crates/evefrontier-lib/src/spatial.rs:1299-1312`.
3. Lambda startup loads bundled index bytes through `SpatialIndex::load_from_bytes()` at `crates/evefrontier-lambda-shared/src/runtime.rs:157-162,259-262`.
4. The loader parses header/metadata, allocates compressed bytes, decompresses zstd, deserializes postcard nodes, then rebuilds KD-tree and hash maps at `crates/evefrontier-lib/src/spatial.rs:901-1081` and `1114-1263`.

**Concrete finding**
- `SpatialIndex::load()` subtracts fixed/header metadata sizes from `file_metadata.len()` without first checking that the file is large enough: `spatial.rs:1015-1018`.
- A syntactically plausible but truncated file can underflow `usize` in debug builds or request an enormous allocation in unchecked release arithmetic before checksum verification.
- The byte-reader path separately uses unbounded `read_to_end()` and unbounded zstd decompression at `spatial.rs:1195-1228`.

**Fuzz/property oracle**
- Mutate valid serialized fixture indexes plus arbitrary byte inputs.
- Assert that both entry points return `Ok` or crate `Error`, never panic/abort.
- For successful loads, assert node count/lookups are coherent and nearest-query output IDs exist in the loaded node set.
- Seeds: valid v1/v2 index, header-only index, metadata flag/version mismatches, inflated tag length, missing checksum, checksum-valid zstd payloads.

**Expected benefit**
Detects startup denial-of-service, allocator hazards, and parser panics in a custom binary format before a service or CLI consumes a malformed sidecar.

### 2. fmap token codec — **High**
**Harness boundary:** `decode_fmap_token(&str)`; paired property boundary `encode_fmap_token(&[Waypoint]) -> decode_fmap_token`.

**Data flow**
1. A user-controlled CLI positional token is accepted by `FmapDecodeArgs` at `crates/evefrontier-cli/src/main.rs:282-289`.
2. `handle_fmap_decode()` directly passes it to `decode_fmap_token()` at `main.rs:1269-1272`.
3. The decoder Base64-decodes, gzip-decompresses into an unbounded `Vec`, parses a four-byte header, reserves waypoint capacity from a `u16`, and bit-decodes each waypoint at `crates/evefrontier-lib/src/fmap.rs:307-365`.

**Concrete findings**
- Gzip output is read with unbounded `read_to_end()` at `fmap.rs:313-318`; a small compressed token can consume substantial memory/CPU.
- Encoding narrows `waypoints.len()` to `u16` without a bound check at `fmap.rs:220`; more than 65,535 inputs encode a wrapped count, while decoding trusts that truncated count at `fmap.rs:330,348-365`.

**Fuzz/property oracle**
- Fuzz arbitrary UTF-8/byte-derived tokens through `decode_fmap_token`; assert error-or-valid result with no panic and bounded resource use.
- Property-test generated waypoint vectors:
  - for lengths `<= u16::MAX`, decode(encode(points)) exactly preserves IDs and waypoint types;
  - for larger vectors, encoding must reject rather than silently produce a non-round-trippable token.
- Include malformed gzip, invalid Base64, invalid version/bit width/type, truncated bit payloads, and maximum-count headers.

**Expected benefit**
Protects a public CLI decoder from decompression-driven availability failures and detects token corruption caused by count truncation.

### 3. ZIP dataset extraction — **High**
**Harness boundary:** a temporary `.zip` passed to public `download_latest_from_source_with_cache()`.

**Data flow**
1. Production release selection dispatches archive assets to `download_archive_asset()` at `crates/evefrontier-lib/src/github.rs:252,726-731`.
2. The archive is opened with `ZipArchive`, iterated, path-checked with `enclosed_name()`, and the first `.db` entry is copied to a temporary output at `github.rs:931-971`.
3. The public local-source path exercises closely related ZIP parsing/extraction at `github.rs:345-435` and `436-530`.

**Concrete finding**
- Both remote and local ZIP paths use `io::copy()` on selected entries without compressed-size, decompressed-size, entry-count, or total-extraction limits (`github.rs:392-399,462-480,960-967`).
- Path traversal is explicitly skipped via `enclosed_name()` (`github.rs:948-958`), but that does not constrain ZIP-bomb disk/CPU consumption.

**Fuzz/property oracle**
- Generate malformed ZIP bytes and structured ZIPs containing directories, traversal names, duplicate `.db` entries, CSV companions, and highly compressible database-like entries.
- Invoke the public local-source helper in an isolated temporary target/cache directory.
- Assert no panic, no writes outside the temporary directory, and either a valid target DB or a typed error.
- A resource-limited corpus can establish a maximum extracted-byte/entry behavior once such limits exist.

**Expected benefit**
Exercises the same archive parser family used for GitHub release downloads and targets availability/integrity failures at the dataset trust boundary identified in `docs/threat-model.md`.

### 4. Lambda route request → graph construction — **Medium**
**Harness boundary:** deserialize `evefrontier_lambda_shared::RouteRequest`, call `validate()`, then invoke the route handler against the fixture runtime.

**Data flow**
1. Lambda route handler deserializes event JSON and calls request validation at `crates/evefrontier-lambda-route/src/lib.rs:68-93`.
2. It forwards `max_spatial_neighbors` directly into the library request at `lib.rs:119-121`.
3. `plan_route()` forwards this value to graph selection at `crates/evefrontier-lib/src/routing/mod.rs:487-518`.
4. Indexed graph construction calculates `max_neighbors + 1` at `crates/evefrontier-lib/src/graph.rs:372-378`.

**Concrete finding**
- `RouteRequest.max_spatial_neighbors: Option<usize>` is externally deserializable (`crates/evefrontier-lambda-shared/src/requests.rs:23-85`) but has no range validation in `Validate` (`requests.rs:133-207`).
- `usize::MAX` reaches unchecked `max_neighbors + 1` in graph construction. This panics in overflow-checked builds; unchecked builds wrap to zero and alter routing behavior.
- The downstream spatial query caps `k` to 10,000 (`spatial.rs:700-721`), but that mitigation occurs after the addition.

**Fuzz/property oracle**
- Fuzz JSON payloads into the public Lambda handler using a one-time fixture runtime.
- Focus structured mutations on `max_spatial_neighbors`, `avoid`, route names, algorithm, and numeric constraints.
- Assert every accepted request produces a success/problem response rather than a panic; compare accepted normal-range values against direct planner invocation.
- Boundary seeds: `0`, `1`, `250`, `10_000`, `usize::MAX`, missing/invalid enum fields, and repeated avoidance names.

**Expected benefit**
Covers the only externally exposed path that forwards a caller-selected graph fan-out. It detects overflow/panic behavior and guards routing availability and response correctness.

### Scope Notes
- SQLite files are another untrusted-input boundary (`crates/evefrontier-lib/src/db.rs:244-281`), but parsing is primarily delegated to `rusqlite`/SQLite; the custom compressed/binary/archive parsers above offer higher project-specific fuzz value.
- HTTP service request validation already constrains range-query `limit` to 1–100 (`crates/evefrontier-service-shared/src/request.rs:155-195`), making it lower priority than the Lambda route fan-out field.