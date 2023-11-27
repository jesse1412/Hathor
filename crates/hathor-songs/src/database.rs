mod initialise_db;

use std::error::Error;

use super::audio;
use initialise_db::init_db;
use rusqlite::named_params;
use rusqlite::Connection;
use rusqlite::Result;
use std::path::Path;

const INSERT_BATCH_SIZE: u16 = 64;

/// Connects to SQL database and initialises Hathor tables if needed.
pub fn get_connection(db_path: &Path) -> Result<Connection, Box<dyn std::error::Error>> {
    let conn = Connection::open(db_path)?;
    init_db(&conn)?;
    Ok(conn)
}

/// Inserts a slice of [AudioFile](super::audio::AudioFile)s into the DB.
///
/// # Arguments
///
/// * `conn` - The open database connection to insert into.
/// * `songs` - Collection of [AudioFile](super::audio::AudioFile)s to insert.
///
/// # Examples
///
/// ```no_run
/// use hathor_songs::audio::AudioFile;
/// use hathor_songs::database::insert_songs;
/// use std::path::Path;
/// use rusqlite::Connection;
///
/// let mut conn = Connection::open_in_memory().unwrap();
/// let mut songs = Vec::new();
/// songs.push(AudioFile::default());
/// insert_songs(&mut conn, &songs);
pub fn insert_songs(
    conn: &mut Connection,
    songs: &[audio::AudioFile],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut songs_iter = songs.iter().peekable();
    while songs_iter.peek().is_some() {
        let transaction = conn.transaction()?;
        insert_next_batch_of_songs(&transaction, &mut songs_iter)?;
        transaction.commit()?;
    }
    Ok(())
}

fn insert_next_batch_of_songs(
    transaction: &rusqlite::Transaction<'_>,
    songs_iter: &mut std::iter::Peekable<std::slice::Iter<'_, audio::AudioFile>>,
) -> Result<(), Box<dyn Error>> {
    let mut statement = transaction
        .prepare_cached(include_str!("database/insert_song.sql"))
        .unwrap();
    for _ in 0..=INSERT_BATCH_SIZE {
        if let Some(song) = songs_iter.next() {
            let params = named_params! {
                ":file_hash": song.file_hash.to_string(),
                ":song_title": song.song_title,
                ":album_title": song.album_title,
                ":artist_name": song.artist_name,
                ":track_num": song.track_num,
                ":release_year": song.release_year,
                ":song_length_s": song.song_length.whole_nanoseconds() as i64,
                ":song_path": song.song_path.canonicalize().unwrap().into_os_string().into_string().unwrap(),
                ":img_path": song.img_path.as_ref().unwrap().canonicalize()?.into_os_string().into_string().unwrap(),
            };
            statement.execute(params)?;
        } else {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod test_db_operations {
    use crate::audio::AudioFile;
    use crate::database::{get_connection, init_db, insert_songs};
    use blake3::Hash;
    use rusqlite::{named_params, Connection};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::str::FromStr;

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

    /// Create a fake test database, insert a batch of songs, and check it inserted.
    /// TODO: Expand/refactor this when more functions are implemented.
    #[test]
    fn test_insert_songs() {
        let mut conn = Connection::open_in_memory().unwrap();
        init_db(&conn).expect("Failed to create test database.");
        let a1 = AudioFile {
            file_hash: Hash::from_hex(format!("{:064}", 0)).unwrap(),
            song_title: String::from("test song title 1"),
            album_title: String::from("test album title 1"),
            song_path: std::path::PathBuf::from_str(
                r"../../test_media_files/audio/albums/album/test.mp3",
            )
            .unwrap(),
            img_path: Some(
                std::path::PathBuf::from_str(
                    r"../../test_media_files/audio/albums/album_with_cover_file/cover.png",
                )
                .unwrap(),
            ),
            ..AudioFile::default()
        };
        let a2 = AudioFile {
            file_hash: Hash::from_hex(format!("{:064}", 1)).unwrap(),
            song_title: String::from("test song title 2"),
            album_title: String::from("test album title 2"),
            song_path: std::path::PathBuf::from_str(
                r"../../test_media_files/audio/albums/album/test.mp3",
            )
            .unwrap(),
            img_path: Some(
                std::path::PathBuf::from_str(
                    r"../../test_media_files/audio/albums/album_with_cover_file/cover.png",
                )
                .unwrap(),
            ),
            ..AudioFile::default()
        };
        let songs = vec![a1, a2];
        insert_songs(&mut conn, &songs).unwrap();
        let album_name = conn
            .query_row::<String, _, _>(
                "SELECT ALBUM_TITLE FROM SONGS WHERE SONG_TITLE = :song_title",
                named_params! {":song_title": songs[0].song_title },
                |row| row.get::<usize, String>(0),
            )
            .unwrap();
        assert_eq!(album_name, songs[0].album_title);
    }
}
