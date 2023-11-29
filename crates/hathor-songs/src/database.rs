mod initialise_db;
pub mod playlists;
pub mod songs;
pub mod user_meda_folders;

use initialise_db::init_db;
use rusqlite::Connection;
use rusqlite::Result;
use std::path::Path;

/// Connects to SQL database and initialises Hathor tables if needed.
pub fn get_connection(db_path: &Path) -> Result<Connection, Box<dyn std::error::Error>> {
    let conn = Connection::open(db_path)?;
    init_db(&conn)?;
    Ok(conn)
}

#[cfg(test)]
mod test_db_operations {
    use crate::database::get_connection;
    use std::fs;
    use std::path::{Path, PathBuf};

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

    #[test]
    fn test_get_connection_with_new_db() {
        let test_db_path = Path::new(".test_hathor.sqlite3");
        let _test_db_context = TestContext {
            db_path: PathBuf::from(test_db_path),
        };
        let conn = get_connection(test_db_path).unwrap();
        let test_query_result = conn
            .query_row::<String, _, _>("SELECT \"test\";", (), |row| row.get::<usize, String>(0))
            .unwrap();
        assert_eq!(test_query_result, "test");
    }
}
