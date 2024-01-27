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
                ":audio_path": audio.audio_path.canonicalize().unwrap().into_os_string().into_string().unwrap(),
                ":img_path": audio.img_path.as_ref().map(|s| {
                    s.canonicalize()
                        .unwrap()
                        .into_os_string()
                        .into_string()
                        .unwrap()
                }),
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
    use crate::database::init_db;
    use blake3::Hash;
    use rusqlite::{named_params, Connection};
    use std::path::PathBuf;

    /// Create a fake test database, insert a batch of audios, and check it inserted.
    #[test]
    fn test_insert_audios() {
        let mut conn = Connection::open_in_memory().unwrap();
        let audios = init_db_with_two_different_audios(&mut conn);
        let album_name = conn
            .query_row::<String, _, _>(
                r"SELECT album_name FROM audios WHERE audio_title = :audio_title",
                named_params! {":audio_title": audios[0].audio_title },
                |row| row.get::<usize, String>(0),
            )
            .unwrap();
        assert_eq!(album_name, audios[0].album_name);
    }

    /// Create a fake test database,
    /// insert a batch of two audios with same hash/details but different paths,
    /// and check only one audio inserted
    /// + both audio paths were inserted.
    #[test]
    fn test_insert_same_audio_file_two_paths() {
        let mut conn = Connection::open_in_memory().unwrap();
        let audios = init_db_with_two_same_audios(&mut conn);
        let file_count = conn
            .query_row::<usize, _, _>(
                r"SELECT COUNT(audio_files.file_hash) AS AMT 
                FROM audios JOIN audio_files 
                ON audios.file_hash = audio_files.file_hash 
                WHERE audios.audio_title = :audio_title",
                named_params! {":audio_title": audios[0].audio_title },
                |row| row.get::<usize, usize>(0),
            )
            .unwrap();
        assert_eq!(file_count, audios.len());
        let audios_count = conn
            .query_row::<usize, _, _>(
                r"SELECT COUNT(audios.file_hash) AS AMT 
                FROM audios WHERE audios.audio_title = :audio_title",
                named_params! {":audio_title": audios[0].audio_title },
                |row| row.get::<usize, usize>(0),
            )
            .unwrap();
        assert_eq!(audios_count, 1);
    }

    /// Create a fake test database, insert a batch of audios,
    /// and check they can be retrieved by hash.
    #[test]
    fn test_get_audio_by_hash() {
        let mut conn = Connection::open_in_memory().unwrap();
        let audios = init_db_with_two_different_audios(&mut conn);
        let audiofile_from_db = get_audio_by_hash(&mut conn, &audios[0].file_hash);
        assert_eq!(audiofile_from_db, audios[0]);
    }

    /// Create a fake test database, insert a batch of audios,
    /// and check they can be retrieved by an exact album name match.
    #[test]
    fn test_get_audio_by_album_name_exact() {
        let mut conn = Connection::open_in_memory().unwrap();
        let audios = init_db_with_two_different_audios(&mut conn);
        let audiofile_from_db = &get_audios_by_album_name(&mut conn, &audios[0].album_name);
        let audiofile_from_db = &audiofile_from_db[0];
        assert_eq!(*audiofile_from_db, audios[0]);
    }

    /// Create a fake test database, insert a batch of audios,
    /// and check they can be retrieved by a partial album name match.
    #[test]
    fn test_get_audios_by_album_name_partial() {
        let mut conn = Connection::open_in_memory().unwrap();
        let audios = init_db_with_two_different_audios(&mut conn);
        let audiofiles_from_db = &get_audios_by_album_name(&mut conn, "album");
        for (l, r) in audios.iter().zip(audiofiles_from_db) {
            assert_eq!(l, r);
        }
    }

    /// Create a fake test database, insert a batch of audios,
    /// and check they can be retrieved by an exact artist name match.
    #[test]
    fn test_get_audios_by_artist_name_exact() {
        let mut conn = Connection::open_in_memory().unwrap();
        let audios = init_db_with_two_different_audios(&mut conn);
        let audiofile_from_db = &get_audios_by_artist_name(&mut conn, &audios[0].artist_name);
        let audiofile_from_db = &audiofile_from_db[0];
        assert_eq!(*audiofile_from_db, audios[0]);
    }

    /// Create a fake test database, insert a batch of audios,
    /// and check they can be retrieved by a partial artist name match.
    #[test]
    fn test_get_audios_by_artist_name_partial() {
        let mut conn = Connection::open_in_memory().unwrap();
        let audios = init_db_with_two_different_audios(&mut conn);
        let audiofiles_from_db = &get_audios_by_artist_name(&mut conn, "artist");
        for (l, r) in audios.iter().zip(audiofiles_from_db) {
            assert_eq!(l, r);
        }
    }

    /// Create a fake test database, insert a batch of audios,
    /// and check they can be retrieved by an exact title match.
    #[test]
    fn test_get_audio_by_title_exact() {
        let mut conn = Connection::open_in_memory().unwrap();
        let audios = init_db_with_two_different_audios(&mut conn);
        let audiofile_from_db = &get_audios_by_title(&mut conn, &audios[0].audio_title);
        let audiofile_from_db = &audiofile_from_db[0];
        assert_eq!(*audiofile_from_db, audios[0]);
    }

    /// Create a fake test database, insert a batch of audios,
    /// and check they can be retrieved by a partial title match.
    #[test]
    fn test_get_audio_by_title_partial() {
        let mut conn = Connection::open_in_memory().unwrap();
        let audios = init_db_with_two_different_audios(&mut conn);
        let audiofiles_from_db = &get_audios_by_title(&mut conn, "audio");
        for (l, r) in audios.iter().zip(audiofiles_from_db) {
            assert_eq!(l, r);
        }
    }

    /// Initialise a test db, insert two audio files, and return a vec of those audio files.
    /// TODO: Expand/refactor this when more functions are implemented.
    fn init_db_with_two_different_audios(conn: &mut Connection) -> Vec<AudioFile> {
        init_db(&*conn).expect("Failed to create test database.");

        let mut audio_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        audio_path.push(r"../../test_media_files/audio/albums/album/test.mp3");
        audio_path = audio_path.canonicalize().unwrap();
        let mut img_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        img_path.push(r"../../test_media_files/audio/albums/album_with_cover_file/cover.png");
        img_path = img_path.canonicalize().unwrap();

        let a1 = AudioFile {
            file_hash: Hash::from_hex(format!("{:064}", 0)).unwrap(),
            audio_title: String::from("test audio title 1"),
            artist_name: String::from("test artist 1"),
            album_name: String::from("test album title 1"),
            audio_path: audio_path.clone(),
            img_path: Some(img_path.clone()),
            ..AudioFile::default()
        };
        let a2 = AudioFile {
            file_hash: Hash::from_hex(format!("{:064}", 1)).unwrap(),
            audio_title: String::from("test audio title 2"),
            artist_name: String::from("test artist 2"),
            album_name: String::from("test album title 2"),
            audio_path: audio_path.clone(),
            img_path: Some(img_path.clone()),
            ..AudioFile::default()
        };
        let audios = vec![a1, a2];
        insert_audios(conn, &audios).unwrap();
        audios
    }

    /// Initialise a test db, insert two audio files, and return a vec of those audio files.
    /// TODO: Expand/refactor this when more functions are implemented.
    fn init_db_with_two_same_audios(conn: &mut Connection) -> Vec<AudioFile> {
        init_db(&*conn).expect("Failed to create test database.");

        let mut audio_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        audio_path.push(r"../../test_media_files/audio/albums/album/test.mp3");
        audio_path = audio_path.canonicalize().unwrap();
        let mut img_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        img_path.push(r"../../test_media_files/audio/albums/album_with_cover_file/cover.png");
        img_path = img_path.canonicalize().unwrap();

        let a1 = AudioFile {
            file_hash: Hash::from_hex(format!("{:064}", 0)).unwrap(),
            audio_title: String::from("test audio title 1"),
            artist_name: String::from("test artist 1"),
            album_name: String::from("test album title 1"),
            audio_path: audio_path.clone(),
            img_path: Some(img_path.clone()),
            ..AudioFile::default()
        };

        let mut audio_path_2 = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        audio_path_2.push(r"../../test_media_files/audio/albums/album/test2.mp3");
        audio_path_2 = audio_path_2.canonicalize().unwrap();

        let a2 = AudioFile {
            audio_path: audio_path_2,
            ..a1.clone()
        };
        let audios = vec![a1, a2];
        insert_audios(conn, &audios).unwrap();
        audios
    }
}
