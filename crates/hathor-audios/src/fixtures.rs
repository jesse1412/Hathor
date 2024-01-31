use std::path::{Path, PathBuf};

use rstest::fixture;
use std::fs;

/// Deletes the temporary test DB from file-system on drop.
/// So we instantiate it at the start of the tests,
/// otherwise the test DB may persist on panic.
struct TestContext {
    db_path: PathBuf,
}

impl Drop for TestContext {
    fn drop(&mut self) {
        fs::remove_file(&self.db_path).ok();
    }
}

#[fixture]
fn test_db() -> TestContext {
    let test_db_path = Path::new(".test_hathor.sqlite3");
    TestContext {
        db_path: PathBuf::from(test_db_path),
    }
}
