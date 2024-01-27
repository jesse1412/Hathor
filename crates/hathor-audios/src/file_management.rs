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
/// use hathor_audios::file_management::get_all_audio_file_paths;
/// use std::path::Path;
///
/// let p = Path::new(r"C:\audios\");
/// let audio_file_paths = get_all_audio_file_paths(&p);
pub fn get_all_audio_file_paths(path: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
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
