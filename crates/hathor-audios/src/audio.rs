mod from_file;
mod output;
mod playback;
pub mod playback_manager;
use blake3::Hash;
use time::Duration;
#[cfg(not(target_os = "linux"))]
mod resampler;

/// A minimal representation of an Audio file for Hathor.
#[derive(Eq, PartialEq, Debug, Hash, Clone)]
pub struct AudioFile {
    pub file_hash: Hash,
    pub audio_title: String,
    pub album_name: String,
    pub artist_name: String,
    pub track_num: u8,
    pub release_year: u16,
    pub audio_length: Duration,
    pub audio_path: std::path::PathBuf,
    pub img_path: Option<std::path::PathBuf>,
}

impl Default for AudioFile {
    fn default() -> Self {
        AudioFile {
            file_hash: Hash::from_hex(format!("{:064}", 0)).unwrap(),
            audio_title: String::default(),
            album_name: String::default(),
            artist_name: String::default(),
            track_num: 1,
            release_year: 1,
            audio_length: Duration::default(),
            audio_path: std::path::PathBuf::default(),
            img_path: None,
        }
    }
}
