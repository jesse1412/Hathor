use crate::audio::{self, AudioFile};
use crate::database::INSERT_BATCH_SIZE;
use rusqlite::named_params;
use rusqlite::Connection;
use std::error::Error;

/// Inserts a slice of [AudioFile](super::audio::AudioFile)s into a playlist in the DB.
///
/// # Arguments
///
/// * `conn` - The open database connection to insert into.
/// * `playlist_title` - Title of the playlist.
/// * `songs` - Collection of [AudioFile](super::audio::AudioFile)s to insert.
///
/// # Examples
///
/// ```no_run
/// use hathor_songs::audio::AudioFile;
/// use hathor_songs::database::playlists::insert_songs_into_playlist;
/// use std::path::Path;
/// use rusqlite::Connection;
///
/// let mut conn = Connection::open_in_memory().unwrap();
/// let mut songs = Vec::new();
/// songs.push(AudioFile::default());
/// insert_songs_into_playlist(&mut conn, &"my_playlist", &songs);
pub fn insert_songs_into_playlist(
    conn: &mut Connection,
    playlist_title: &str,
    songs: &[AudioFile],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut songs_iter = songs.iter().peekable();
    while songs_iter.peek().is_some() {
        let transaction = conn.transaction()?;
        insert_next_batch_of_songs_into_playlist(&transaction, playlist_title, &mut songs_iter)?;
        transaction.commit()?;
    }
    Ok(())
}

fn insert_next_batch_of_songs_into_playlist(
    transaction: &rusqlite::Transaction<'_>,
    playlist_title: &str,
    songs_iter: &mut std::iter::Peekable<std::slice::Iter<'_, audio::AudioFile>>,
) -> Result<(), Box<dyn Error>> {
    let mut statement = transaction
        .prepare_cached(include_str!(r"playlists/add_song_to_playlist.sql"))
        .unwrap();
    for _ in 0..=INSERT_BATCH_SIZE {
        if let Some(song) = songs_iter.next() {
            let params = named_params! {
                ":file_hash": song.file_hash.to_string(),
                ":playlist_name": playlist_title,
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
    use crate::database::init_db;
    use crate::database::playlists::insert_songs_into_playlist;
    use blake3::Hash;
    use rusqlite::{named_params, Connection};
    use std::path::PathBuf;

    /// Create a fake test database, insert a batches of songs into playlist, and check it inserted.
    #[test]
    fn test_insert_songs_into_playlist() {
        let mut conn = Connection::open_in_memory().unwrap();
        init_db(&conn).expect("Failed to create test database.");

        let mut song_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        song_path.push(r"../../test_media_files/audio/albums/album/test.mp3");
        song_path = song_path.canonicalize().unwrap();
        let mut img_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        img_path.push(r"../../test_media_files/audio/albums/album_with_cover_file/cover.png");
        img_path = img_path.canonicalize().unwrap();
        let songs_1 = vec![
            AudioFile {
                file_hash: Hash::from_hex(format!("{:064}", 0)).unwrap(),
                song_title: String::from("test song title 1"),
                artist_name: String::from("test artist 1"),
                album_name: String::from("test album title 1"),
                song_path: song_path.clone(),
                img_path: Some(img_path.clone()),
                ..AudioFile::default()
            },
            AudioFile {
                file_hash: Hash::from_hex(format!("{:064}", 1)).unwrap(),
                song_title: String::from("test song title 2"),
                artist_name: String::from("test artist 2"),
                album_name: String::from("test album title 2"),
                song_path: song_path.clone(),
                img_path: Some(img_path.clone()),
                ..AudioFile::default()
            },
        ];
        let songs_2 = vec![AudioFile {
            file_hash: Hash::from_hex(format!("{:064}", 2)).unwrap(),
            song_title: String::from("test song title 3"),
            artist_name: String::from("test artist 4"),
            album_name: String::from("test album title 5"),
            song_path: song_path.clone(),
            img_path: Some(img_path.clone()),
            ..AudioFile::default()
        }];
        let playlist_name_1 = "test_playlist_1";
        let playlist_name_2 = "test_playlist_2";
        insert_songs_into_playlist(&mut conn, playlist_name_1, &songs_1).unwrap();
        insert_songs_into_playlist(&mut conn, playlist_name_2, &songs_2).unwrap();
        let mut stmt = conn
            .prepare(r"SELECT SONG_HASH FROM PLAYLISTS WHERE PLAYLIST_NAME = :playlist_name")
            .unwrap();
        let playlist_1_hashes: Vec<String> = stmt
            .query_map(named_params! {":playlist_name": playlist_name_1}, |r| {
                r.get::<usize, String>(0)
            })
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert_eq!(playlist_1_hashes[0], songs_1[0].file_hash.to_string());
        assert_eq!(playlist_1_hashes[1], songs_1[1].file_hash.to_string());
        assert_eq!(playlist_1_hashes.len(), songs_1.len());
    }
}
