use std::error::Error;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const COMPATIBLE_AUDIO_TYPES: &[&str] = &[
    "aac", "adpcm", "alac", "flac", "mkv", "mp1", "mp2", "mp3", "mp4", "ogg", "vorbis", "wav",
    "webm",
];

pub fn get_all_audio_files(path: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut audio_files = Vec::new();
    for entry in WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let extension = entry.path().extension();

        if let Some(extension) = extension {
            if COMPATIBLE_AUDIO_TYPES.contains(&extension.to_str().unwrap().to_lowercase().as_str())
            {
                audio_files.push(PathBuf::from(&entry.path()));
            }
        }
    }

    Ok(audio_files)
}
