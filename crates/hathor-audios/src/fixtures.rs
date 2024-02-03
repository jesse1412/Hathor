use std::path::{Path, PathBuf};

use crate::audio::AudioFile;
use crate::database::audio_files::insert_audios;
use crate::database::initialise_db::init_db;
use crate::database::playlists::insert_audios_into_playlist;
use blake3::Hash;
use rstest::fixture;
use rusqlite::Connection;
use std::env::temp_dir;
use std::fs;
use std::thread;

/// Deletes the temporary test path from file-system on drop.
/// So we instantiate it at the start of the tests,
/// otherwise the test files may persist on panic.
pub struct TestContext {
    pub path: PathBuf,
}

impl Drop for TestContext {
    fn drop(&mut self) {
        if self.path.is_file() {
            fs::remove_file(&self.path)
        } else {
            fs::remove_dir_all(&self.path)
        }
        .ok();
    }
}

/// Deletes the temporary test path from file-system on drop.
/// So we instantiate it at the start of the tests,
/// otherwise the test files may persist on panic.
pub struct TestInMemoryDBContext {
    pub temp_audio_dir: PathBuf,
    pub audios: Vec<AudioFile>,
    pub connection: Connection,
}

impl Drop for TestInMemoryDBContext {
    fn drop(&mut self) {
        if self.temp_audio_dir.is_file() {
            fs::remove_file(&self.temp_audio_dir)
        } else {
            fs::remove_dir_all(&self.temp_audio_dir)
        }
        .ok();
    }
}

#[fixture]
pub(crate) fn db_in_file() -> TestContext {
    let test_db_path = Path::new(".test_hathor.sqlite3");
    TestContext {
        path: PathBuf::from(test_db_path),
    }
}

#[fixture]
pub(crate) fn temp_audios_context() -> TestInMemoryDBContext {
    let conn = Connection::open_in_memory().expect("Failed to create test database.");
    init_db(&conn).expect("Failed to initialise test database.");
    let mut audio_temp_dir = temp_dir();
    audio_temp_dir.push(format!("hathor_tests_{:?}", thread::current().id()));
    std::fs::create_dir_all(&audio_temp_dir).unwrap();
    TestInMemoryDBContext {
        temp_audio_dir: audio_temp_dir,
        audios: Vec::new(),
        connection: conn,
    }
}

#[fixture]
pub(crate) fn audio_read_from_file() -> AudioFile {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push(r"../../test_media_files/audio/albums/album/test.mp3");
    AudioFile::from_file(&p).unwrap()
}

#[fixture]
fn audio_fake_multiple(
    #[default(3)] count: usize,
    mut temp_audios_context: TestInMemoryDBContext,
) -> TestInMemoryDBContext {
    for n in 0..count {
        // Create temp files.
        let mut audio_path = temp_audios_context
            .temp_audio_dir
            .clone()
            .canonicalize()
            .unwrap();
        audio_path.push(format!("{}.mp3", n));
        fs::File::create(&audio_path).unwrap();

        let mut img_path = temp_audios_context
            .temp_audio_dir
            .clone()
            .canonicalize()
            .unwrap();
        img_path.push(format!("{}.png", n));
        fs::File::create(&img_path).unwrap();

        temp_audios_context.audios.push(AudioFile {
            file_hash: Hash::from_hex(format!("{:064}", n)).unwrap(),
            audio_title: String::from("test title ") + &n.to_string(),
            artist_name: String::from("test artist ") + &n.to_string(),
            album_name: String::from("test album ") + &n.to_string(),
            audio_path,
            img_path: Some(img_path),
            ..AudioFile::default()
        });
    }
    temp_audios_context
}

/// Inserts 3 fake audios into an in memory db.
/// The first two audios go into `test_playlist_1`.
/// The third audio goes into `test_playlist_2`.
#[fixture]
pub(crate) fn playlist_db_in_memory(
    mut audio_fake_multiple: TestInMemoryDBContext,
) -> TestInMemoryDBContext {
    let (audios_1, audios_2) = audio_fake_multiple.audios.split_at(2);

    insert_audios_into_playlist(
        &mut audio_fake_multiple.connection,
        "test_playlist_1",
        audios_1,
    )
    .unwrap();
    insert_audios_into_playlist(
        &mut audio_fake_multiple.connection,
        "test_playlist_2",
        audios_2,
    )
    .unwrap();
    insert_audios(&mut audio_fake_multiple.connection, audios_1).unwrap();
    insert_audios(&mut audio_fake_multiple.connection, audios_2).unwrap();
    audio_fake_multiple
}
