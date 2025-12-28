# Lambda Dataset Bundle Directory

This directory contains the dataset files that are bundled into Lambda binaries via `include_bytes!`.
The files are **required** before building any Lambda crate.

## Required Files

| File                         | Description                                      |
| ---------------------------- | ------------------------------------------------ |
| `static_data.db`             | SQLite database with starmap data (SolarSystems, Jumps, etc.) |
| `static_data.db.spatial.bin` | Precomputed KD-tree spatial index for fast neighbor queries |

## Setup Instructions

1. **Download the dataset** using the CLI:

   ```bash
   cargo run -p evefrontier-cli -- download
   ```

2. **Copy the downloaded files** to this directory:

   ```bash
   # Find where the CLI downloaded the dataset
   # Typically ~/.local/share/evefrontier/static_data.db on Linux
   # or ~/Library/Application Support/evefrontier/static_data.db on macOS

   cp ~/.local/share/evefrontier/static_data.db data/
   ```

3. **Build the spatial index** (if not already present):

   ```bash
   cargo run -p evefrontier-cli -- index-build --data-dir data/
   ```

   This creates `data/static_data.db.spatial.bin`.

4. **Verify both files exist**:

   ```bash
   ls -la data/
   # Should show:
   # static_data.db
   # static_data.db.spatial.bin
   ```

## Build Errors

If you see a compile error like:

```
error: Dataset file not found at data/static_data.db
```

Follow the setup instructions above to place the required files in this directory.

## File Sizes

Typical file sizes (may vary by dataset release):

- `static_data.db`: ~5-10 MB
- `static_data.db.spatial.bin`: ~1-3 MB

These files are embedded directly into the Lambda binary, so the final binary size
will include this data. This tradeoff eliminates network latency during cold starts.

## Updating the Dataset

When a new dataset release is available:

1. Download the new release via CLI
2. Replace the files in this directory
3. Rebuild the spatial index
4. Rebuild the Lambda binaries

## Security Note

> [!IMPORTANT]
> These files are gitignored to avoid committing large binary files to the repository. Each
> developer/CI environment must set up these files independently.
