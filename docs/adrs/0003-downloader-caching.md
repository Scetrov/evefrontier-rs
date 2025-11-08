# ADR 0003: Downloader caching and atomic writes

## Status

Accepted

## Context

The project must download relatively-large dataset releases from GitHub and cache them locally to
avoid repeated downloads. Partial or corrupted downloads must not overwrite a valid cached asset.

## Decision

Use the OS cache directory to store release assets under `evefrontier_datasets/`. Write downloads to
a temporary file and atomically rename to the final path once the download completes successfully.

## Rationale

- Using an OS cache directory follows platform conventions and avoids polluting user data folders.
- Atomic rename prevents partially-written files from being used by other processes.

## Consequences

- Cached assets persist between runs and automatically save network bandwidth.
- Tests that require determinism may provide an explicit path to `ensure_c3e6_dataset`.
