use blake3::Hash;
use std::error::Error;
use symphonia::core::formats::{FormatOptions, Track};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{MetadataOptions, MetadataRevision, StandardTagKey};
use symphonia::core::probe::Hint;
use time::Duration;

pub struct AudioFile {
    pub file_hash: Hash,
    pub song_title: String,
    pub album_title: String,
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
            album_title: String::default(),
            artist_name: String::default(),
            track_num: 1,
            release_year: 1,
            song_length: Duration::default(),
            song_path: std::path::PathBuf::default(),
            img_path: None,
        }
    }
}

impl AudioFile {
    pub fn from_file(song_path: &std::path::Path) -> Result<AudioFile, Box<dyn Error>> {
        let mut audio_file = AudioFile::default();
        // Open file.
        let file = std::fs::File::open(song_path).expect("failed to open media");
        let mut probe = AudioFile::get_song_probe(file, song_path);

        // Add the metadata we already have
        audio_file.song_path = song_path.to_path_buf();

        // Add metadata from within the file itself.
        if let Some(metadata_rev) = probe.format.metadata().current() {
            audio_file.add_symphonia_medadata(metadata_rev);
        } else if let Some(metadata_rev) = probe.metadata.get().as_ref().and_then(|m| m.current()) {
            audio_file.add_symphonia_medadata(metadata_rev);
        }

        // Add metadata from processing the file.
        // Length.
        let track = &probe.format.tracks()[0];
        audio_file.song_length = AudioFile::get_song_length(track);

        // File hash.
        audio_file.file_hash = AudioFile::get_file_hash(song_path)?;
        println!("{}", audio_file.file_hash);
        Ok(audio_file)
    }

    fn get_song_length(track: &Track) -> Duration {
        let track_length = track
            .codec_params
            .time_base
            .unwrap()
            .calc_time(track.codec_params.n_frames.unwrap());
        Duration::seconds(track_length.seconds as i64)
    }

    fn get_file_hash(song_path: &std::path::Path) -> Result<Hash, Box<dyn Error>> {
        let mut hasher = blake3::Hasher::new();
        let file = std::fs::File::open(song_path)?;
        hasher.update_reader(file)?;
        Ok(hasher.finalize())
    }

    fn add_symphonia_medadata(
        self: &mut AudioFile,
        metadata_rev: &MetadataRevision,
    ) -> &mut AudioFile {
        let tags = metadata_rev.tags();
        for tag in tags.iter() {
            if let Some(key) = tag.std_key {
                match key {
                    StandardTagKey::TrackTitle => self.song_title = tag.value.to_string(),
                    StandardTagKey::Album => self.album_title = tag.value.to_string(),
                    StandardTagKey::Artist => self.artist_name = tag.value.to_string(),
                    StandardTagKey::TrackNumber => {
                        self.track_num = tag.value.to_string().parse::<u8>().unwrap()
                    }
                    StandardTagKey::Date => {
                        self.release_year = tag.value.to_string().parse::<u16>().unwrap()
                    }
                    _ => (),
                }
            }
        }
        self
    }

    fn get_song_probe(
        file: std::fs::File,
        song_path: &std::path::Path,
    ) -> symphonia::core::probe::ProbeResult {
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let mut hint = Hint::new();

        // Provide the file extension as a hint.
        if let Some(extension) = song_path.extension() {
            if let Some(extension_str) = extension.to_str() {
                hint.with_extension(extension_str);
            }
        }
        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();

        symphonia::default::get_probe()
            .format(&hint, mss, &fmt_opts, &meta_opts)
            .expect("unsupported format")
    }
}
