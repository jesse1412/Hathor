use crate::audio::{self, AudioFile};
use blake3::Hash;
use rusqlite::{Connection, Row};
use std::{error::Error, str::FromStr, usize};
const INSERT_BATCH_SIZE: u16 = 64;
use rusqlite::named_params;
use std::path::PathBuf;
use time::Duration;

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
/// use hathor_songs::database::songs::insert_songs;
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

/// Retvieve song with the given hash from the db.
///
/// # Arguments
///
/// * `conn` - The open database connection to insert into.
/// * `hash` - The hash to retrieve.
///
/// Examples
/// ```no_run
/// use rusqlite::Connection;
/// use blake3::Hash;
/// use hathor_songs::database::songs::get_song_by_hash;
///
/// let mut conn = Connection::open_in_memory().unwrap();
/// let hash = Hash::from_hex(format!("{:064}", 0)).unwrap();
/// let song = get_song_by_hash(&mut conn, &hash);
pub fn get_song_by_hash(conn: &mut Connection, hash: &Hash) -> AudioFile {
    conn.query_row::<_, _, _>(
        include_str!("songs/get_song_by_hash.sql"),
        named_params! {":file_hash": hash.to_string() },
        song_select_result_to_audiofile,
    )
    .unwrap()
}

fn song_select_result_to_audiofile(row: &Row) -> Result<AudioFile, rusqlite::Error> {
    let mut img_path: Option<PathBuf> = None;
    let img_path_str = &row.get::<usize, String>(8);
    if let Ok(img_path_str) = img_path_str {
        if let Ok(p) = PathBuf::from_str(img_path_str) {
            img_path = Some(p);
        }
    }
    let song_path: PathBuf;
    if let Ok(p) = PathBuf::from_str(&row.get::<usize, String>(7)?) {
        song_path = p;
    } else {
        return Err(rusqlite::Error::ExecuteReturnedResults);
    }
    Ok(AudioFile {
        file_hash: Hash::from_str(&row.get::<usize, String>(0)?).unwrap(),
        song_title: row.get(1)?,
        album_title: row.get(2)?,
        artist_name: row.get(3)?,
        track_num: row.get(4)?,
        release_year: row.get(5)?,
        song_length: Duration::seconds(row.get::<usize, i64>(6)?),
        song_path,
        img_path,
    })
}

fn insert_next_batch_of_songs(
    transaction: &rusqlite::Transaction<'_>,
    songs_iter: &mut std::iter::Peekable<std::slice::Iter<'_, audio::AudioFile>>,
) -> Result<(), Box<dyn Error>> {
    let mut statement = transaction
        .prepare_cached(include_str!(r"songs/insert_song.sql"))
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
mod test_songs_operations {
    use crate::audio::AudioFile;
    use crate::database::init_db;
    use crate::database::songs::{get_song_by_hash, insert_songs};
    use blake3::Hash;
    use rusqlite::{named_params, Connection};
    use std::fs;
    use std::path::PathBuf;

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

    /// Create a fake test database, insert a batch of songs, and check it inserted.
    #[test]
    fn test_insert_songs() {
        let mut conn = Connection::open_in_memory().unwrap();
        let songs = init_db_with_two_songs(&mut conn);
        let album_name = conn
            .query_row::<String, _, _>(
                r"SELECT ALBUM_TITLE FROM SONGS WHERE SONG_TITLE = :song_title",
                named_params! {":song_title": songs[0].song_title },
                |row| row.get::<usize, String>(0),
            )
            .unwrap();
        assert_eq!(album_name, songs[0].album_title);
    }

    /// Create a fake test database, insert a batch of songs,
    /// and check they can be retrieved by hash.
    #[test]
    fn test_get_song_by_hash() {
        let mut conn = Connection::open_in_memory().unwrap();
        let songs = init_db_with_two_songs(&mut conn);
        let audiofile_from_db = get_song_by_hash(&mut conn, &songs[0].file_hash);
        assert_eq!(audiofile_from_db, songs[0]);
    }

    /// Initialise a test db, insert two audio files, and return a vec of those audio files.
    /// TODO: Expand/refactor this when more functions are implemented.
    fn init_db_with_two_songs(conn: &mut Connection) -> Vec<AudioFile> {
        init_db(&*conn).expect("Failed to create test database.");

        let mut song_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        song_path.push(r"../../test_media_files/audio/albums/album/test.mp3");
        song_path = song_path.canonicalize().unwrap();
        let mut img_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        img_path.push(r"../../test_media_files/audio/albums/album_with_cover_file/cover.png");
        img_path = img_path.canonicalize().unwrap();

        let a1 = AudioFile {
            file_hash: Hash::from_hex(format!("{:064}", 0)).unwrap(),
            song_title: String::from("test song title 1"),
            album_title: String::from("test album title 1"),
            song_path: song_path.clone(),
            img_path: Some(img_path.clone()),
            ..AudioFile::default()
        };
        let a2 = AudioFile {
            file_hash: Hash::from_hex(format!("{:064}", 1)).unwrap(),
            song_title: String::from("test song title 2"),
            album_title: String::from("test album title 2"),
            song_path: song_path.clone(),
            img_path: Some(img_path.clone()),
            ..AudioFile::default()
        };
        let songs = vec![a1, a2];
        insert_songs(conn, &songs).unwrap();
        songs
    }
}