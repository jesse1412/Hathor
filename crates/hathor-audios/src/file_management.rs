use crate::audio::AudioFile;
use std::error::Error;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const COMPATIBLE_AUDIO_TYPES: &[&str] = &[
    "aac", "adpcm", "alac", "flac", "mkv", "mp1", "mp2", "mp3", "mp4", "ogg", "vorbis", "wav",
    "webm",
];

/// Recursively finds audio file paths.
///
/// # Arguments
///
/// * `path` - Path to a file or directory containing files.
///
/// # Examples
///
/// ```
/// use hathor_audios::file_management::get_all_audio_files_at_path;
/// use std::path::Path;
///
/// let p = Path::new(r"C:\audios\");
/// let audio_file_paths = get_all_audio_files_at_path(&p);
pub fn get_all_audio_files_at_path(path: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut audio_file_paths = Vec::new();
    for entry in WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let extension = entry.path().extension();

        if let Some(extension) = extension {
            if COMPATIBLE_AUDIO_TYPES.contains(&extension.to_str().unwrap().to_lowercase().as_str())
            {
                audio_file_paths.push(PathBuf::from(&entry.path()));
            }
        }
    }

    Ok(audio_file_paths)
}

pub fn get_audios_from_paths(paths: &[PathBuf]) -> Vec<AudioFile> {
    paths
        .iter()
        .filter_map(|p| AudioFile::from_file(p).ok())
        .collect()
}

#[cfg(test)]
const TEST_AUDIO_FOLDER: &str = r"../../test_media_files/audio/albums";

#[cfg(test)]
use blake3::Hash;
#[cfg(test)]
use std::str::FromStr;
#[cfg(test)]
use time::Duration;

#[test]
fn test_get_all_audio_file_paths_in_direct_folder() {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push(TEST_AUDIO_FOLDER);
    p.push("album");
    let paths = get_all_audio_files_at_path(&p).unwrap();
    assert_eq!(paths.len(), 2);
    let mut file_names = paths
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap())
        .collect::<Vec<&str>>();
    // Sort to prevent none deterministic failures.
    file_names.sort_unstable();
    assert_eq!(file_names, vec!["test.mp3", "test2.mp3"]);
}

#[test]
fn test_get_all_audio_file_paths_in_indirect_folder() {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push(TEST_AUDIO_FOLDER);
    p.push("album_split");
    let paths = get_all_audio_files_at_path(&p).unwrap();
    assert_eq!(paths.len(), 2);
    let mut file_names = paths
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap())
        .collect::<Vec<&str>>();
    // Sort to prevent none deterministic failures.
    file_names.sort_unstable();
    assert_eq!(file_names, vec!["test.mp3", "test2.mp3"]);
}

#[test]
fn test_get_all_audio_file_paths_all_file_types() {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push(TEST_AUDIO_FOLDER);
    p.push("album_bad_files");
    let paths = get_all_audio_files_at_path(&p).unwrap();
    assert_eq!(paths.len(), COMPATIBLE_AUDIO_TYPES.len());
    let mut file_names = paths
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap())
        .collect::<Vec<&str>>();
    // Sort to prevent none deterministic failures.
    file_names.sort_unstable();
    // We expect test cases for every compatible file type.
    let mut expected_file_names = COMPATIBLE_AUDIO_TYPES
        .iter()
        .map(|s| String::from("bad.") + s)
        .collect::<Vec<String>>();
    expected_file_names.sort_unstable();
    assert_eq!(file_names, expected_file_names);
}

#[test]
fn test_get_all_audio_file_paths_skips_none_audio_files() {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push(TEST_AUDIO_FOLDER);
    p.push("album_with_cover_file");
    let paths = get_all_audio_files_at_path(&p).unwrap();
    assert_eq!(paths.len(), 1);
    let mut file_names = paths
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap())
        .collect::<Vec<&str>>();
    // Sort to prevent none deterministic failures.
    file_names.sort_unstable();
    assert_eq!(file_names, vec!["test.mp3"]);
}

#[test]
fn test_get_audios_from_paths_works() {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push(TEST_AUDIO_FOLDER);
    p.push(r"album\test.mp3");
    let paths = vec![p];
    let audio = get_audios_from_paths(&paths).first().unwrap().clone();
    assert_eq!(
        audio,
        AudioFile {
            file_hash: Hash::from_str(
                "0955ffa35bfeabf7a0140a3199791c9a5e175d672a1f3317497bc1c962a0ddf5"
            )
            .unwrap(),
            audio_title: String::from("test song name"),
            album_name: String::from("test album"),
            artist_name: String::from("test artist"),
            track_num: 1,
            release_year: 2023,
            audio_length: Duration::seconds(20),
            audio_path: audio.audio_path.clone(),
            img_path: None
        }
    )
}

#[test]
fn test_get_audios_from_paths_works_multiple_files() {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push(TEST_AUDIO_FOLDER);
    p.push(r"album\test.mp3");
    let mut p2 = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p2.push(TEST_AUDIO_FOLDER);
    p2.push(r"album\test2.mp3");
    let paths = vec![p, p2];
    let audios = get_audios_from_paths(&paths);
    assert_eq!(
        audios,
        vec![
            AudioFile {
                file_hash: Hash::from_str(
                    "0955ffa35bfeabf7a0140a3199791c9a5e175d672a1f3317497bc1c962a0ddf5"
                )
                .unwrap(),
                audio_title: String::from("test song name"),
                album_name: String::from("test album"),
                artist_name: String::from("test artist"),
                track_num: 1,
                release_year: 2023,
                audio_length: Duration::seconds(20),
                audio_path: audios[0].audio_path.clone(),
                img_path: None
            },
            AudioFile {
                file_hash: Hash::from_str(
                    "0955ffa35bfeabf7a0140a3199791c9a5e175d672a1f3317497bc1c962a0ddf5"
                )
                .unwrap(),
                audio_title: String::from("test song name"),
                album_name: String::from("test album"),
                artist_name: String::from("test artist"),
                track_num: 1,
                release_year: 2023,
                audio_length: Duration::seconds(20),
                audio_path: audios[1].audio_path.clone(),
                img_path: None
            },
        ]
    )
}

#[test]
fn test_get_audios_from_paths_skips_bad_files() {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push(TEST_AUDIO_FOLDER);
    p.push(r"album_bad_files/bad.");
    let bad_files = COMPATIBLE_AUDIO_TYPES
        .iter()
        .map(|s| PathBuf::from_str(&(p.to_str().unwrap().to_owned() + s)).unwrap())
        .collect::<Vec<PathBuf>>();
    let audios = get_audios_from_paths(&bad_files);
    assert_eq!(audios.len(), 0)
}
