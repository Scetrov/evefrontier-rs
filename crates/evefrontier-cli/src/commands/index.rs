//! Index build and verify command handlers.

use std::path::Path;

use anyhow::{Context, Result};

use evefrontier_lib::{
    compute_dataset_checksum, ensure_dataset, load_starmap, read_release_tag, spatial_index_path,
    verify_freshness, DatasetMetadata, DatasetRelease, FreshnessResult, SpatialIndex,
    VerifyDiagnostics, VerifyOutput,
};

/// Arguments for the index-build command.
#[derive(Debug, Clone)]
pub struct IndexBuildArgs {
    /// Force rebuild even if index already exists.
    pub force: bool,
}

/// Arguments for the index-verify command.
#[derive(Debug, Clone)]
pub struct IndexVerifyArgs {
    /// Output in JSON format instead of human-readable text.
    pub json: bool,
    /// Only output on failure (quiet mode for scripts).
    pub quiet: bool,
    /// Require release tag match in addition to checksum (strict mode).
    pub strict: bool,
}

/// Exit codes for index-verify command (per contract).
pub mod exit_codes {
    pub const SUCCESS: i32 = 0;
    pub const STALE: i32 = 1;
    pub const MISSING: i32 = 2;
    pub const FORMAT_ERROR: i32 = 3;
    pub const DATASET_MISSING: i32 = 4;
    pub const ERROR: i32 = 5;
}

/// Handle the index-build subcommand.
///
/// Builds or rebuilds the spatial index for faster routing.
pub fn handle_index_build(
    target_path: Option<&Path>,
    release: DatasetRelease,
    args: &IndexBuildArgs,
) -> Result<()> {
    let paths = tokio::task::block_in_place(|| ensure_dataset(target_path, release))
        .context("failed to locate or download the EVE Frontier dataset")?;

    let index_path = spatial_index_path(&paths.database);

    // Check if index already exists
    if index_path.exists() && !args.force {
        println!(
            "Spatial index already exists at {}\nUse --force to rebuild.",
            index_path.display()
        );
        return Ok(());
    }

    println!("Loading starmap from {}...", paths.database.display());
    let starmap = load_starmap(&paths.database)
        .with_context(|| format!("failed to load dataset from {}", paths.database.display()))?;

    // Compute dataset checksum for freshness verification (v2 format)
    println!("Computing dataset checksum...");
    let checksum =
        compute_dataset_checksum(&paths.database).context("failed to compute dataset checksum")?;

    // Read release tag from marker file if present
    let release_tag = read_release_tag(&paths.database);

    // Create metadata for v2 format
    let metadata = DatasetMetadata {
        checksum,
        release_tag: release_tag.clone(),
        build_timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0),
    };

    println!(
        "Building spatial index (v2) for {} systems...",
        starmap.systems.len()
    );
    let index = SpatialIndex::build_with_metadata(&starmap, metadata);

    let systems_with_temp = starmap
        .systems
        .values()
        .filter(|s| s.metadata.min_external_temp.is_some())
        .count();

    println!("Saving index to {}...", index_path.display());
    index
        .save(&index_path)
        .context("failed to save spatial index")?;

    let file_size = std::fs::metadata(&index_path).map(|m| m.len()).unwrap_or(0);

    println!("Spatial index built successfully:");
    println!("  Path: {}", index_path.display());
    println!("  Format: v2 (with metadata)");
    println!("  Systems indexed: {}", index.len());
    println!("  Systems with temperature: {}", systems_with_temp);
    if let Some(ref tag) = release_tag {
        println!("  Dataset release: {}", tag);
    }
    println!("  Dataset checksum: {}...", hex::encode(&checksum[..8]));
    println!("  File size: {} bytes", file_size);

    Ok(())
}

/// Handle the index-verify subcommand.
///
/// Verifies that the spatial index is fresh (matches the current dataset).
pub fn handle_index_verify(
    target_path: Option<&Path>,
    release: DatasetRelease,
    args: &IndexVerifyArgs,
) -> Result<()> {
    let start = std::time::Instant::now();

    // Resolve paths (run in blocking region to allow internal blocking I/O).
    let paths = tokio::task::block_in_place(|| ensure_dataset(target_path, release))
        .context("failed to locate or download the EVE Frontier dataset")?;
    let index_path = spatial_index_path(&paths.database);

    // Run verification
    let result = verify_freshness(&index_path, &paths.database);

    // Compute diagnostics
    let verification_time_ms = start.elapsed().as_millis() as u64;
    let diagnostics = VerifyDiagnostics {
        dataset_path: paths.database.display().to_string(),
        index_path: index_path.display().to_string(),
        dataset_size: std::fs::metadata(&paths.database).ok().map(|m| m.len()),
        index_size: std::fs::metadata(&index_path).ok().map(|m| m.len()),
        index_version: detect_index_version(&index_path),
        verification_time_ms,
    };

    // Determine freshness and recommended action
    let (is_fresh, recommended_action, exit_code) = match &result {
        FreshnessResult::Fresh { .. } => (true, None, exit_codes::SUCCESS),
        FreshnessResult::Stale { .. } => (
            false,
            Some("evefrontier-cli index-build".to_string()),
            exit_codes::STALE,
        ),
        FreshnessResult::LegacyFormat { .. } => (
            false,
            Some("evefrontier-cli index-build --force".to_string()),
            exit_codes::FORMAT_ERROR,
        ),
        FreshnessResult::Missing { .. } => (
            false,
            Some("evefrontier-cli index-build".to_string()),
            exit_codes::MISSING,
        ),
        FreshnessResult::DatasetMissing { .. } => (
            false,
            Some("evefrontier-cli download".to_string()),
            exit_codes::DATASET_MISSING,
        ),
        FreshnessResult::Error { .. } => (false, None, exit_codes::ERROR),
    };

    // Build output structure
    let output = VerifyOutput {
        result: result.clone(),
        is_fresh,
        recommended_action: recommended_action.clone(),
        diagnostics: Some(diagnostics),
    };

    // Output based on format and quiet mode
    if args.json {
        // JSON output
        let json = serde_json::to_string_pretty(&output)?;
        if !args.quiet || !is_fresh {
            println!("{}", json);
        }
    } else {
        // Human-readable output
        if !args.quiet || !is_fresh {
            print_human_readable_result(&result, &output);
        }
    }

    // Exit with appropriate code
    if !is_fresh {
        std::process::exit(exit_code);
    }

    Ok(())
}

/// Detect the version byte from a spatial index file header.
fn detect_index_version(path: &Path) -> Option<u8> {
    use std::io::Read;
    let mut file = std::fs::File::open(path).ok()?;
    let mut header = [0u8; 16];
    file.read_exact(&mut header).ok()?;
    if &header[0..4] == b"EFSI" {
        Some(header[4])
    } else {
        None
    }
}

/// Print human-readable verification result.
fn print_human_readable_result(result: &FreshnessResult, output: &VerifyOutput) {
    match result {
        FreshnessResult::Fresh {
            checksum,
            release_tag,
        } => {
            println!("✓ Spatial index is fresh");
            if let Some(tag) = release_tag {
                println!("  Dataset:  {} ({}...)", tag, &checksum[..16]);
            } else {
                println!("  Dataset:  {}...", &checksum[..16]);
            }
            if let Some(ref diag) = output.diagnostics {
                if let Some(version) = diag.index_version {
                    println!("  Index:    v{} format", version);
                }
            }
        }
        FreshnessResult::Stale {
            expected_checksum,
            actual_checksum,
            expected_tag,
            actual_tag,
        } => {
            println!("✗ Spatial index is STALE");
            println!("  Dataset checksum:  {}...", &actual_checksum[..16]);
            println!("  Index source:      {}...", &expected_checksum[..16]);
            if expected_tag.is_some() || actual_tag.is_some() {
                println!(
                    "  Expected tag: {:?}, Actual tag: {:?}",
                    expected_tag, actual_tag
                );
            }
            println!();
            if let Some(ref action) = output.recommended_action {
                println!("  Run '{}' to regenerate", action);
            }
        }
        FreshnessResult::LegacyFormat {
            index_path,
            message,
        } => {
            println!("✗ Spatial index uses legacy format (v1)");
            println!("  Index file: {}", index_path);
            println!("  {}", message);
            println!();
            if let Some(ref action) = output.recommended_action {
                println!("  Run '{}' to upgrade to v2", action);
            }
        }
        FreshnessResult::Missing { expected_path } => {
            println!("✗ Spatial index not found");
            println!("  Expected: {}", expected_path);
            println!();
            if let Some(ref action) = output.recommended_action {
                println!("  Run '{}' to create", action);
            }
        }
        FreshnessResult::DatasetMissing { expected_path } => {
            println!("✗ Dataset not found");
            println!("  Expected: {}", expected_path);
            println!();
            if let Some(ref action) = output.recommended_action {
                println!("  Run '{}' to download", action);
            }
        }
        FreshnessResult::Error { message } => {
            println!("✗ Verification error");
            println!("  {}", message);
        }
    }
}
