use crate::audio::AudioFile;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const COMPATIBLE_AUDIO_TYPES: &[&str] = &[
    "aac", "adpcm", "alac", "flac", "mkv", "mp1", "mp2", "mp3", "mp4", "ogg", "vorbis", "wav",
    "webm",
];

/// Recursively finds audio files at the given path.
/// [AudioFile]s are then created from all found files.
/// Invalid/unreadable audio files are skipped.
pub fn get_all_audios_at_path(path: &Path) -> Vec<AudioFile> {
    let paths = get_all_audio_file_paths_at_path(path);
    get_audios_from_paths(&paths)
}

/// Recursively finds audio file paths.
fn get_all_audio_file_paths_at_path(path: &Path) -> Vec<PathBuf> {
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

    audio_file_paths
}

fn get_audios_from_paths(paths: &[PathBuf]) -> Vec<AudioFile> {
    paths
        .iter()
        .filter_map(|p| AudioFile::from_file(p).ok())
        .collect()
}

#[cfg(test)]
mod file_management_tests {
    const TEST_AUDIO_FOLDER: &str = r"/../../test_media_files/audio/albums";

    use super::{
        get_all_audio_file_paths_at_path, get_all_audios_at_path, get_audios_from_paths,
        COMPATIBLE_AUDIO_TYPES,
    };
    use rstest::rstest;
    use std::path::PathBuf;

    #[rstest]
    // Directly in folder, just mp3.
    #[case(
        PathBuf::from(env!("CARGO_MANIFEST_DIR").to_owned() + TEST_AUDIO_FOLDER + "/album"),
        vec![String::from("test.mp3"), String::from("test2.mp3")])
    ]
    // Indirectly in folder, just mp3.
    #[case(
        PathBuf::from(env!("CARGO_MANIFEST_DIR").to_owned() + TEST_AUDIO_FOLDER + "/album_split"),
        vec![String::from("test.mp3"), String::from("test2.mp3")])
    ]
    // Directly in folder, all file types.
    #[case(
        PathBuf::from(env!("CARGO_MANIFEST_DIR").to_owned() + TEST_AUDIO_FOLDER + "/album_bad_files"),
        COMPATIBLE_AUDIO_TYPES
            .iter()
            .map(|s| String::from("bad.") + s)
            .collect::<Vec<String>>())
    ]
    // Directly in folder, contains none valid/audio files.
    #[case(
        PathBuf::from(env!("CARGO_MANIFEST_DIR").to_owned() + TEST_AUDIO_FOLDER + "/album_with_cover_file"),
        vec![String::from("test.mp3")])
    ]
    fn test_get_all_audio_file_paths_in_folders_works(
        #[case] folder_path: PathBuf,
        #[case] expected_found_file_names: Vec<String>,
    ) {
        let paths = get_all_audio_file_paths_at_path(&folder_path);
        let mut file_names = paths
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap())
            .collect::<Vec<&str>>();
        // Sort to prevent none deterministic failures.
        file_names.sort_unstable();
        assert_eq!(file_names, expected_found_file_names);
    }

    #[rstest]
    // Valid mp3s.
    #[case(
        get_all_audio_file_paths_at_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR").to_owned() + TEST_AUDIO_FOLDER + "/album")),
        vec![String::from("test.mp3"), String::from("test2.mp3")])
    ]
    // Bad files.
    #[case(
        get_all_audio_file_paths_at_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR").to_owned() + TEST_AUDIO_FOLDER + "/album_bad_files")),
        Vec::new()
    )]
    // Mix of good and bad files.
    #[case(
        get_all_audio_file_paths_at_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR").to_owned() + TEST_AUDIO_FOLDER + "/album_with_cover_file")),
        vec![String::from("test.mp3")])
    ]
    fn test_get_audios_from_paths_works(
        #[case] folder_path: Vec<PathBuf>,
        #[case] expected_found_file_names: Vec<String>,
    ) {
        let audios = get_audios_from_paths(&folder_path);
        let mut file_names = audios
            .iter()
            .map(|a| a.audio_path.file_name().unwrap().to_str().unwrap())
            .collect::<Vec<&str>>();
        // Sort to prevent none deterministic failures.
        file_names.sort_unstable();
        assert_eq!(file_names, expected_found_file_names);
    }

    #[rstest]
    // Valid mp3s.
    #[case(
        PathBuf::from(env!("CARGO_MANIFEST_DIR").to_owned() + TEST_AUDIO_FOLDER + "/album"),
        vec![String::from("test.mp3"), String::from("test2.mp3")])
    ]
    // Bad files.
    #[case(
        PathBuf::from(env!("CARGO_MANIFEST_DIR").to_owned() + TEST_AUDIO_FOLDER + "/album_bad_files"),
        Vec::new()
    )]
    // Mix of good and bad files.
    #[case(
        PathBuf::from(env!("CARGO_MANIFEST_DIR").to_owned() + TEST_AUDIO_FOLDER + "/album_with_cover_file"),
        vec![String::from("test.mp3")])
    ]
    fn test_get_all_audios_at_path_works(
        #[case] folder_path: PathBuf,
        #[case] expected_found_file_names: Vec<String>,
    ) {
        let audios = get_all_audios_at_path(&folder_path);
        let mut file_names = audios
            .iter()
            .map(|a| a.audio_path.file_name().unwrap().to_str().unwrap())
            .collect::<Vec<&str>>();
        // Sort to prevent none deterministic failures.
        file_names.sort_unstable();
        assert_eq!(file_names, expected_found_file_names);
    }
}
