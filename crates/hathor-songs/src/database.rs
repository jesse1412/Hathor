mod initialise_db;

use std::error::Error;

use super::audio;
use initialise_db::init_db;
use rusqlite::named_params;
use rusqlite::Connection;
use rusqlite::Result;

const INSERT_BATCH_SIZE: u16 = 64;

/// Connects to SQL database and initialises Hathor tables if needed.
pub fn get_connection() -> Result<Connection, Box<dyn std::error::Error>> {
    let db_path = std::path::Path::new(".hathor.sqlite3");
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
                ":song_length_ns": song.song_length.whole_nanoseconds() as i64,
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
