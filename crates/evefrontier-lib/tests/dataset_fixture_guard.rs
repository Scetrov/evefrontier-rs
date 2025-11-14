use std::path::PathBuf;

use evefrontier_lib::{ensure_dataset, DatasetRelease, Error};

fn fixture_dataset_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/fixtures/minimal_static_data.db")
        .canonicalize()
        .expect("fixture dataset present")
}

#[test]
fn refuses_to_overwrite_fixture_dataset() {
    let fixture = fixture_dataset_path();
    let error = ensure_dataset(Some(&fixture), DatasetRelease::latest())
        .expect_err("fixture path should be rejected");

    match error {
        Error::ProtectedFixturePath { path } => {
            assert_eq!(path, fixture);
        }
        other => panic!(
            "expected ProtectedFixturePath error, received: {:?}",
            other
        ),
    }
}
