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
/// use hathor_audios::file_management::get_all_audio_file_paths_at_path;
/// use std::path::Path;
///
/// let p = Path::new(r"C:\audios\");
/// let audio_file_paths = get_all_audio_file_paths_at_path(&p);
pub fn get_all_audio_file_paths_at_path(path: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
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

#[cfg(test)]
mod file_management_tests {
    const TEST_AUDIO_FOLDER: &str = r"/../../test_media_files/audio/albums";

    use super::{get_all_audio_file_paths_at_path, COMPATIBLE_AUDIO_TYPES};
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
    fn test_get_all_audio_file_paths_in_folders(
        #[case] folder_path: PathBuf,
        #[case] expected_found_file_names: Vec<String>,
    ) {
        let paths = get_all_audio_file_paths_at_path(&folder_path).unwrap();
        let mut file_names = paths
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap())
            .collect::<Vec<&str>>();
        // Sort to prevent none deterministic failures.
        file_names.sort_unstable();
        assert_eq!(file_names, expected_found_file_names);
    }
}
