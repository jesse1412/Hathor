pub mod audio_files;
mod initialise_db;
pub mod playlists;
pub mod user_media_folders;

use blake3::Hash;
use initialise_db::init_db;
use rusqlite::Connection;
use rusqlite::Params;
use rusqlite::Result;
use std::error::Error;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use time::Duration;

use crate::audio::AudioFile;

const INSERT_BATCH_SIZE: u16 = 64;

/// Connects to SQL database and initialises Hathor tables if needed.
pub fn get_connection(db_path: &Path) -> Result<Connection, Box<dyn std::error::Error>> {
    let conn = Connection::open(db_path)?;
    init_db(&conn)?;
    Ok(conn)
}

pub fn query_map_to_audiofiles<ParamType>(
    conn: &Connection,
    sql: &str,
    parameters: ParamType,
) -> Result<Vec<AudioFile>, Box<dyn Error>>
where
    ParamType: Params,
{
    Ok(conn
        .prepare(sql)?
        .query_map(parameters, |row| {
            let audio_path = PathBuf::from_str(&row.get::<_, String>(7)?).unwrap();
            let img_path = PathBuf::from_str(&row.get::<_, String>(8)?).ok();
            Ok(AudioFile {
                file_hash: Hash::from_str(&row.get::<_, String>(0)?).unwrap(),
                audio_title: row.get(1)?,
                album_name: row.get(2)?,
                artist_name: row.get(3)?,
                track_num: row.get(4)?,
                release_year: row.get(5)?,
                audio_length: Duration::seconds(row.get(6)?),
                audio_path,
                img_path,
            })
        })?
        .filter_map(|v| v.ok())
        .collect())
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
