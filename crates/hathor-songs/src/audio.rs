mod from_file;
use blake3::Hash;
use time::Duration;

/// A minimal representation of an Audio file for Hathor.
#[derive(Eq, PartialEq, Debug, Hash)]
pub struct AudioFile {
    pub file_hash: Hash,
    pub song_title: String,
    pub album_name: String,
    pub artist_name: String,
    pub track_num: u8,
    pub release_year: u16,
    pub song_length: Duration,
    pub song_path: std::path::PathBuf,
    pub img_path: Option<std::path::PathBuf>,
}

impl Default for AudioFile {
    fn default() -> Self {
        AudioFile {
            file_hash: Hash::from_hex(format!("{:064}", 0)).unwrap(),
            song_title: String::default(),
            album_name: String::default(),
            artist_name: String::default(),
            track_num: 1,
            release_year: 1,
            song_length: Duration::default(),
            song_path: std::path::PathBuf::default(),
            img_path: None,
        }
    }
}
