use crate::audio::{self, AudioFile};
use crate::database::{query_map_to_audiofiles, INSERT_BATCH_SIZE};
use blake3::Hash;
use rusqlite::{named_params, Connection, Row};
use std::{error::Error, path::PathBuf, str::FromStr, usize};
use time::Duration;

/// Inserts a slice of [AudioFile](super::audio::AudioFile)s into the DB.
///
/// # Arguments
///
/// * `conn` - The open database connection to insert into.
/// * `audios` - Collection of [AudioFile](super::audio::AudioFile)s to insert.
///
/// # Examples
///
/// ```no_run
/// use hathor_audios::audio::AudioFile;
/// use hathor_audios::database::audio_files::insert_audios;
/// use std::path::Path;
/// use rusqlite::Connection;
///
/// let mut conn = Connection::open_in_memory().unwrap();
/// let mut audios = Vec::new();
/// audios.push(AudioFile::default());
/// insert_audios(&mut conn, &audios);
pub fn insert_audios(
    conn: &mut Connection,
    audios: &[AudioFile],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut audio_iter = audios.iter().peekable();
    while audio_iter.peek().is_some() {
        let transaction = conn.transaction()?;
        insert_next_batch_of_audios(&transaction, &mut audio_iter)?;
        transaction.commit()?;
    }
    Ok(())
}

/// Retvieve audio with the given hash from the db.
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
/// use hathor_audios::database::audio_files::get_audio_by_hash;
///
/// let mut conn = Connection::open_in_memory().unwrap();
/// let hash = Hash::from_hex(format!("{:064}", 0)).unwrap();
/// let audio = get_audio_by_hash(&mut conn, &hash);
pub fn get_audio_by_hash(conn: &mut Connection, hash: &Hash) -> AudioFile {
    conn.query_row::<_, _, _>(
        include_str!("audio_files/get_audio_by_hash.sql"),
        named_params! {":file_hash": hash.to_string() },
        audio_select_result_to_audiofile,
    )
    .unwrap()
}

/// Retvieve audios with albums like the given string.
///
/// # Arguments
///
/// * `conn` - The open database connection to insert into.
/// * `album_name` - The title to retrieve.
///
/// Examples
/// ```no_run
/// use rusqlite::Connection;
/// use hathor_audios::database::audio_files::get_audios_by_album_name;
///
/// let mut conn = Connection::open_in_memory().unwrap();
/// let audios = get_audios_by_album_name(&mut conn, "Ablum name");
pub fn get_audios_by_album_name(conn: &mut Connection, album_name: &str) -> Vec<AudioFile> {
    query_map_to_audiofiles(
        conn,
        include_str!("audio_files/get_audios_by_album_name.sql"),
        named_params! {":album_name": album_name.to_string() },
    )
    .unwrap()
}

/// Retvieve audios with artist names like the given string.
///
/// # Arguments
///
/// * `conn` - The open database connection to insert into.
/// * `artist_name` - The title to retrieve.
///
/// Examples
/// ```no_run
/// use rusqlite::Connection;
/// use hathor_audios::database::audio_files::get_audios_by_artist_name;
///
/// let mut conn = Connection::open_in_memory().unwrap();
/// let audios = get_audios_by_artist_name(&mut conn, "Artist name");
pub fn get_audios_by_artist_name(conn: &mut Connection, audio_title: &str) -> Vec<AudioFile> {
    query_map_to_audiofiles(
        conn,
        include_str!("audio_files/get_audios_by_artist_name.sql"),
        named_params! {":artist_name": audio_title.to_string() },
    )
    .unwrap()
}

/// Retvieve audios with titles like the given string.
///
/// # Arguments
///
/// * `conn` - The open database connection to insert into.
/// * `audio_title` - The title to retrieve.
///
/// Examples
/// ```no_run
/// use rusqlite::Connection;
/// use hathor_audios::database::audio_files::get_audios_by_title;
///
/// let mut conn = Connection::open_in_memory().unwrap();
/// let audios = get_audios_by_title(&mut conn, "Audio name");
pub fn get_audios_by_title(conn: &mut Connection, audio_title: &str) -> Vec<AudioFile> {
    query_map_to_audiofiles(
        conn,
        include_str!("audio_files/get_audios_by_title.sql"),
        named_params! {":audio_title": audio_title.to_string() },
    )
    .unwrap()
}

fn audio_select_result_to_audiofile(row: &Row) -> Result<AudioFile, rusqlite::Error> {
    let mut img_path: Option<PathBuf> = None;
    let img_path_str = &row.get::<usize, String>(8);
    if let Ok(img_path_str) = img_path_str {
        if let Ok(p) = PathBuf::from_str(img_path_str) {
            img_path = Some(p);
        }
    }
    let audio_path: PathBuf;
    if let Ok(p) = PathBuf::from_str(&row.get::<usize, String>(7)?) {
        audio_path = p;
    } else {
        return Err(rusqlite::Error::ExecuteReturnedResults);
    }
    Ok(AudioFile {
        file_hash: Hash::from_str(&row.get::<usize, String>(0)?).unwrap(),
        audio_title: row.get(1)?,
        album_name: row.get(2)?,
        artist_name: row.get(3)?,
        track_num: row.get(4)?,
        release_year: row.get(5)?,
        audio_length: Duration::seconds(row.get::<usize, i64>(6)?),
        audio_path,
        img_path,
    })
}

fn insert_next_batch_of_audios(
    transaction: &rusqlite::Transaction<'_>,
    audios_iter: &mut std::iter::Peekable<std::slice::Iter<'_, audio::AudioFile>>,
) -> Result<(), Box<dyn Error>> {
    let mut statement_audios = transaction
        .prepare_cached(include_str!(r"audio_files/insert_audio.sql"))
        .unwrap();
    let mut statement_audio_files = transaction
        .prepare_cached(include_str!(r"audio_files/insert_audio_file.sql"))
        .unwrap();
    for _ in 0..=INSERT_BATCH_SIZE {
        if let Some(audio) = audios_iter.next() {
            let params = named_params! {
                ":file_hash": audio.file_hash.to_string(),
                ":audio_title": audio.audio_title,
                ":album_name": audio.album_name,
                ":artist_name": audio.artist_name,
                ":track_num": audio.track_num,
                ":release_year": audio.release_year,
                ":audio_length_s": audio.audio_length.whole_nanoseconds() as i64,
            };
            statement_audios.execute(params)?;
            let params = named_params! {
                ":file_hash": audio.file_hash.to_string(),
                ":audio_path": audio.audio_path.canonicalize()?.into_os_string().into_string().unwrap(),
                ":img_path": audio.img_path.as_ref().unwrap()
                    .canonicalize()?
                        .into_os_string()
                        .into_string().unwrap()
                ,
            };
            statement_audio_files.execute(params)?;
        } else {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod test_audios_operations {
    use crate::audio::AudioFile;
    use crate::database::audio_files::{
        get_audio_by_hash, get_audios_by_album_name, get_audios_by_artist_name,
        get_audios_by_title, insert_audios,
    };
    use crate::fixtures::{playlist_db_in_memory, TestInMemoryDBContext};
    use rstest::rstest;
    use rusqlite::named_params;

    /// Create a fake test database, insert a batch of audios, and check first inserted.
    #[rstest]
    fn test_insert_audios(playlist_db_in_memory: TestInMemoryDBContext) {
        let album_name = playlist_db_in_memory
            .connection
            .query_row::<String, _, _>(
                r"SELECT album_name FROM audios WHERE audio_title = :audio_title",
                named_params! {":audio_title": playlist_db_in_memory.audios[0].audio_title },
                |row| row.get::<usize, String>(0),
            )
            .unwrap();
        assert_eq!(album_name, playlist_db_in_memory.audios[0].album_name);
    }

    /// Create a fake test database,
    /// insert a batch of two audios with same hash/details but different paths,
    /// and check only one audio inserted
    /// + both audio paths were inserted.
    #[rstest]
    fn test_insert_same_audio_file_two_paths(mut playlist_db_in_memory: TestInMemoryDBContext) {
        let new_audio_with_diff_path = AudioFile {
            audio_path: playlist_db_in_memory.audios[1].audio_path.clone(),
            ..playlist_db_in_memory.audios[0].clone()
        };
        insert_audios(
            &mut playlist_db_in_memory.connection,
            &[new_audio_with_diff_path.clone()],
        )
        .unwrap();

        // First, check both audio file paths were added.
        let file_count = playlist_db_in_memory
            .connection
            .query_row::<usize, _, _>(
                r"SELECT COUNT(audio_files.file_hash) AS AMT 
                FROM audios
                    INNER JOIN audio_files 
                        ON audios.file_hash = audio_files.file_hash 
                WHERE audios.file_hash = :file_hash",
                named_params! {":file_hash": new_audio_with_diff_path.file_hash.to_string() },
                |row| row.get::<usize, usize>(0),
            )
            .unwrap();

        // Second, check only 1 audio entry was added.
        assert_eq!(file_count, 2);
        let audios_count = playlist_db_in_memory
            .connection
            .query_row::<usize, _, _>(
                r"SELECT COUNT(audios.file_hash) AS AMT 
                FROM audios
                WHERE audios.file_hash = :file_hash",
                named_params! {":file_hash": new_audio_with_diff_path.file_hash.to_string() },
                |row| row.get::<usize, usize>(0),
            )
            .unwrap();
        assert_eq!(audios_count, 1);
    }

    /// Create a fake test database, insert a batch of audios,
    /// and check they can be retrieved by hash.
    #[rstest]
    fn test_get_audio_by_hash(mut playlist_db_in_memory: TestInMemoryDBContext) {
        let audiofile_from_db = get_audio_by_hash(
            &mut playlist_db_in_memory.connection,
            &playlist_db_in_memory.audios[0].file_hash,
        );
        assert_eq!(audiofile_from_db, playlist_db_in_memory.audios[0]);
    }

    /// Create a fake test database, insert a batch of audios,
    /// and check multiple can be retrieved by an album name match.
    #[rstest]
    fn test_get_audios_by_album_name_multiple(mut playlist_db_in_memory: TestInMemoryDBContext) {
        let audiofiles_from_db =
            get_audios_by_album_name(&mut playlist_db_in_memory.connection, "album");
        assert_eq!(audiofiles_from_db, playlist_db_in_memory.audios);
    }

    /// Create a fake test database, insert a batch of audios,
    /// and check multiple can be retrieved by an artist name match.
    #[rstest]
    fn test_get_audios_by_artist_name_multiple(mut playlist_db_in_memory: TestInMemoryDBContext) {
        let audiofiles_from_db =
            get_audios_by_artist_name(&mut playlist_db_in_memory.connection, "artist");
        assert_eq!(audiofiles_from_db, playlist_db_in_memory.audios);
    }

    /// Create a fake test database, insert a batch of audios,
    /// and check multiple can be retrieved by an title match.
    #[rstest]
    fn test_get_audios_by_title_multiple(mut playlist_db_in_memory: TestInMemoryDBContext) {
        let audiofiles_from_db =
            get_audios_by_title(&mut playlist_db_in_memory.connection, "title");
        assert_eq!(audiofiles_from_db, playlist_db_in_memory.audios);
    }
}
