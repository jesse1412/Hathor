use crate::audio::{self, AudioFile};
use crate::database::{query_map_to_audiofiles, INSERT_BATCH_SIZE};
use rusqlite::named_params;
use rusqlite::Connection;
use std::error::Error;

/// Inserts a slice of [AudioFile](super::audio::AudioFile)s into a playlist in the DB.
///
/// # Arguments
///
/// * `conn` - The open database connection to insert into.
/// * `playlist_title` - Title of the playlist.
/// * `audios` - Collection of [AudioFile](super::audio::AudioFile)s to insert.
///
/// # Examples
///
/// ```no_run
/// use hathor_audios::audio::AudioFile;
/// use hathor_audios::database::playlists::insert_audios_into_playlist;
/// use std::path::Path;
/// use rusqlite::Connection;
///
/// let mut conn = Connection::open_in_memory().unwrap();
/// let mut audios = Vec::new();
/// audios.push(AudioFile::default());
/// insert_audios_into_playlist(&mut conn, &"my_playlist", &audios);
pub fn insert_audios_into_playlist(
    conn: &mut Connection,
    playlist_title: &str,
    audios: &[AudioFile],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut audios_iter = audios.iter().peekable();
    while audios_iter.peek().is_some() {
        let transaction = conn.transaction()?;
        insert_next_batch_of_audios_into_playlist(&transaction, playlist_title, &mut audios_iter)?;
        transaction.commit()?;
    }
    Ok(())
}

/// Retvieve audios from the named playlist.
///
/// # Arguments
///
/// * `conn` - The open database connection to insert into.
/// * `playlist_name` - The playlist to retrieve (exact match only).
///
/// Examples
/// ```no_run
/// use rusqlite::Connection;
/// use hathor_audios::database::playlists::get_audios_from_playlist;
///
/// let mut conn = Connection::open_in_memory().unwrap();
/// let audios = get_audios_from_playlist(&mut conn, "Playlist name");
pub fn get_audios_from_playlist(conn: &mut Connection, playlist_name: &str) -> Vec<AudioFile> {
    query_map_to_audiofiles(
        conn,
        include_str!("playlists/get_audios_from_playlist.sql"),
        named_params! {":playlist_name": playlist_name.to_string() },
    )
    .unwrap()
}

fn insert_next_batch_of_audios_into_playlist(
    transaction: &rusqlite::Transaction<'_>,
    playlist_name: &str,
    audios_iter: &mut std::iter::Peekable<std::slice::Iter<'_, audio::AudioFile>>,
) -> Result<(), Box<dyn Error>> {
    let mut statement = transaction
        .prepare_cached(include_str!(r"playlists/add_audio_to_playlist.sql"))
        .unwrap();
    for _ in 0..=INSERT_BATCH_SIZE {
        if let Some(audio) = audios_iter.next() {
            let params = named_params! {
                ":playlist_name": playlist_name,
                ":file_hash": audio.file_hash.to_string(),
            };
            statement.execute(params)?;
        } else {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod test_playlists_operations {
    use crate::audio::AudioFile;
    use crate::database::audio_files::get_audios_by_title;
    use crate::database::playlists::get_audios_from_playlist;
    use crate::fixtures::{playlist_db_in_memory, TestInMemoryDBContext};
    use rstest::rstest;

    /// Create a fake test database, insert a batch of audios into two playlists, and check it inserted.
    #[rstest]
    fn test_insert_audios_into_playlist(mut playlist_db_in_memory: TestInMemoryDBContext) {
        let actual_audios = get_audios_by_title(&mut playlist_db_in_memory.connection, "test");
        assert_eq!(actual_audios, playlist_db_in_memory.audios);
    }

    /// Create a fake test database, insert a batch of audios into playlist,
    /// and check that they can be retrieved + reconstructed via the playlist name.
    #[rstest]
    fn test_get_audios_from_playlist(mut playlist_db_in_memory: TestInMemoryDBContext) {
        let playlist_1_audios: Vec<AudioFile> =
            get_audios_from_playlist(&mut playlist_db_in_memory.connection, "test_playlist_1");
        assert_eq!(playlist_1_audios, playlist_db_in_memory.audios[0..2]);
    }
}
