use super::AudioFile;

use std::error::Error;

use blake3::Hash;
use symphonia::core::formats::{FormatOptions, Track};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{MetadataOptions, MetadataRevision, StandardTagKey};
use symphonia::core::probe::Hint;
use time::Duration;

impl AudioFile {
    /// Returns an [AudioFile](super::audio::AudioFile) populated from the file at the given path.
    ///
    /// # Arguments
    ///
    /// * `audio_path` - Path to the target audio file.
    ///
    /// # Examples
    /// ```no_run
    /// use hathor_audios::audio::AudioFile;
    /// use std::path::Path;
    ///
    /// let p = Path::new(r"../test.mp3");
    /// let audio = AudioFile::from_file(p);
    pub fn from_file(audio_path: &std::path::Path) -> Result<AudioFile, Box<dyn Error>> {
        let mut audio_file = AudioFile::default();
        // Open file.

        let mut probe = AudioFile::get_audio_probe(audio_path);

        // Add the metadata we already have
        audio_file.audio_path = audio_path.to_path_buf().canonicalize().unwrap();

        // Add metadata from within the file itself.
        if let Some(metadata_rev) = probe.format.metadata().current() {
            audio_file.add_symphonia_metadata(metadata_rev);
        } else if let Some(metadata_rev) = probe.metadata.get().as_ref().and_then(|m| m.current()) {
            audio_file.add_symphonia_metadata(metadata_rev);
        }

        // Add metadata from processing the file.
        // Length.
        let track = &probe.format.tracks()[0];
        audio_file.audio_length = AudioFile::get_audio_length(track);

        // File hash.
        audio_file.file_hash = AudioFile::get_file_hash(audio_path)?;
        Ok(audio_file)
    }

    /// Not intended for external use as it has to read entire track.
    /// After initialisation via from_file, self.audio_length will contain this.
    fn get_audio_length(track: &Track) -> Duration {
        let track_length = track
            .codec_params
            .time_base
            .unwrap()
            .calc_time(track.codec_params.n_frames.unwrap());
        Duration::seconds(track_length.seconds as i64)
    }

    /// Not intended for external use as it has to read entire file.
    /// After initialisation via from_file, self.file_hash will contain this.
    fn get_file_hash(audio_path: &std::path::Path) -> Result<Hash, Box<dyn Error>> {
        let mut hasher = blake3::Hasher::new();
        let file = std::fs::File::open(audio_path)?;
        hasher.update_reader(file)?;
        Ok(hasher.finalize())
    }

    fn add_symphonia_metadata(
        self: &mut AudioFile,
        metadata_rev: &MetadataRevision,
    ) -> &mut AudioFile {
        let tags = metadata_rev.tags();
        for tag in tags.iter() {
            if let Some(key) = tag.std_key {
                match key {
                    StandardTagKey::TrackTitle => self.audio_title = tag.value.to_string(),
                    StandardTagKey::Album => self.album_name = tag.value.to_string(),
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

    fn get_audio_probe(audio_path: &std::path::Path) -> symphonia::core::probe::ProbeResult {
        let file = std::fs::File::open(audio_path).unwrap_or_else(|_| {
            panic!("failed to open file {}", audio_path.to_str().unwrap());
        });
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let mut hint = Hint::new();
        // Provide the file extension as a hint.
        if let Some(extension) = audio_path.extension() {
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

#[cfg(test)]
mod audio_file_tests {
    use crate::audio::AudioFile;
    use std::path::PathBuf;
    use time::Duration;

    const TEST_AUDIO_PATH: &str = r"../../test_media_files/audio/albums/album/test.mp3";

    fn read_audio_file() -> AudioFile {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push(TEST_AUDIO_PATH);
        AudioFile::from_file(&p).unwrap()
    }

    #[test]
    fn test_audio_file_from_file_hashing() {
        assert_eq!(
            read_audio_file().file_hash.to_string(),
            "0955ffa35bfeabf7a0140a3199791c9a5e175d672a1f3317497bc1c962a0ddf5"
        )
    }

    #[test]
    fn test_audio_file_from_file_audio_title() {
        assert_eq!(read_audio_file().audio_title, "test song name")
    }

    #[test]
    fn test_audio_file_from_file_album_name() {
        assert_eq!(read_audio_file().album_name, "test album")
    }

    #[test]
    fn test_audio_file_from_file_artist_name() {
        assert_eq!(read_audio_file().artist_name, "test artist")
    }

    #[test]
    fn test_audio_file_from_file_track_num() {
        assert_eq!(read_audio_file().track_num, 1)
    }

    #[test]
    fn test_audio_file_from_file_release_year() {
        assert_eq!(read_audio_file().release_year, 2023)
    }

    #[test]
    fn test_audio_file_from_file_audio_length() {
        assert_eq!(read_audio_file().audio_length, Duration::new(20, 0))
    }

    #[test]
    fn test_audio_file_from_file_path() {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push(TEST_AUDIO_PATH);
        p = p.canonicalize().unwrap();
        assert_eq!(read_audio_file().audio_path, p)
    }

    #[test]
    fn test_audio_file_from_file_img_path() {
        assert_eq!(read_audio_file().img_path, None)
    }
}
