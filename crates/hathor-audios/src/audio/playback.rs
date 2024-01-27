use super::playback_manager::AudioCommand;
use super::AudioFile;
use crate::audio::output;
use log::info;
use log::warn;
use std::borrow::BorrowMut;
use std::fs::File;
use std::sync::mpsc::{Receiver, Sender};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::codecs::{Decoder, CODEC_TYPE_NULL};
use symphonia::core::formats::FormatOptions;
use symphonia::core::formats::FormatReader;
use symphonia::core::formats::Track;
use symphonia::core::formats::{SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::Time;

/// Main loop of the audio playback thread.
pub(crate) fn do_play_loop(
    receiver_from_audio_manager: Receiver<AudioCommand>,
    sender_to_audio_manager: Sender<eyre::Result<()>>,
) -> eyre::Result<()> {
    let mut playback = Playback::new(receiver_from_audio_manager, sender_to_audio_manager);
    // Handle incoming commands from other threads.
    // Repeat until we get an audio file and can start the playback loop.
    while playback.format_reader.is_none() && playback.decoder.is_none() {
        playback.try_consume_next_audio_command()?;
    }
    loop {
        // Handle incoming commands from other threads.
        playback.try_consume_next_audio_command()?;
        if !playback.play {
            continue;
        }
        // Get the next packet from the format reader.
        let packet = match playback.format_reader.as_mut().unwrap().next_packet() {
            Ok(packet) => packet,
            Err(err) => break Err(err.into()),
        };

        // If the packet does not belong to the selected track, skip it.
        if packet.track_id() != playback.track_id {
            continue;
        }

        // Decode the packet into audio samples.
        match playback.decoder.as_mut().unwrap().decode(&packet) {
            Ok(decoded) => {
                // If the audio output is not open, try to open it.
                if playback.audio_output.is_none() {
                    // Get the audio buffer specification. This is a description of the decoded
                    // audio buffer's sample format and sample rate.
                    let spec = *decoded.spec();

                    // Get the capacity of the decoded buffer. Note that this is capacity, not
                    // length! The capacity of the decoded buffer is constant for the life of the
                    // decoder, but the length is not.
                    let duration = decoded.capacity() as u64;

                    // Try to open the audio output.
                    playback
                        .audio_output
                        .replace(output::try_open(spec, duration).unwrap());
                } else {
                    // TODO: Check the audio spec. and duration hasn't changed.
                }

                // Write the decoded audio samples to the audio output if the presentation timestamp
                // for the packet is >= the seeked position (0 if not seeking).
                if packet.ts() >= playback.seek_ts_seconds {
                    if let Some(audio_output) = playback.audio_output.as_mut() {
                        audio_output.write(decoded).unwrap()
                    }
                }
            }
            Err(symphonia::core::errors::Error::DecodeError(err)) => {
                // Decode errors are not fatal. Print the error message and try to decode the next
                // packet as usual.
                warn!("decode error: {}", err);
            }
            Err(err) => break Err(eyre::Error::from(err)),
        }
    }
}

struct Playback {
    receiver_from_audio_manager: Receiver<AudioCommand>,
    sender_to_audio_manager: Sender<Result<(), eyre::Error>>,
    format_reader: Option<Box<dyn FormatReader>>,
    decoder: Option<Box<dyn Decoder>>,
    audio_output: Option<Box<dyn output::AudioOutput>>,
    play: bool,
    seek_ts_seconds: u64,
    track_id: u32,
}

impl Playback {
    pub fn new(
        receiver_from_audio_manager: Receiver<AudioCommand>,
        sender_to_audio_manager: Sender<Result<(), eyre::Error>>,
    ) -> Self {
        Playback {
            receiver_from_audio_manager,
            sender_to_audio_manager,
            format_reader: None,
            decoder: None,
            audio_output: None,
            play: true,
            seek_ts_seconds: 0,
            track_id: 0,
        }
    }

    /// Checks for the next command in self.receiver_from_audio_manager.
    /// If a command is in the queue:
    ///     Handle it and return Ok(()).
    ///     Send Ok(()) to the AudioManager thread.
    /// Else if no errors:
    ///     Do nothing and return Ok(()).
    ///     Send Ok(()) to the AudioManager thread.
    /// Else:
    ///     Return the error to the AudioManager thread.
    pub fn try_consume_next_audio_command(&mut self) -> Result<(), eyre::Error> {
        if let Ok(cmd) = self.receiver_from_audio_manager.try_recv() {
            let response = match cmd {
                AudioCommand::Pause => {
                    self.play = false;
                    Ok::<(), symphonia::core::errors::Error>(())
                }
                AudioCommand::Play => {
                    self.play = true;
                    Ok(())
                }
                AudioCommand::Seek(ts) => {
                    self.seek(ts);
                    Ok(())
                }
                AudioCommand::ResetPlayback => {
                    self.seek(0);
                    Ok(())
                }
                AudioCommand::ChangeAudio(audio) => {
                    self.change_audio(audio)?;
                    Ok(())
                }
            };
            if response.is_ok() {
                self.sender_to_audio_manager.send(Ok(())).unwrap();
            } else {
                self.sender_to_audio_manager
                    .send(Err(eyre::Error::from(response.err().unwrap())))
                    .unwrap();
            }
        };
        Ok(())
    }

    fn change_audio(
        &mut self,
        audio: Box<AudioFile>,
    ) -> Result<(), symphonia::core::errors::Error> {
        let format_reader = get_format_reader(&audio);
        if let Ok(mut format_reader) = format_reader {
            let decoder = get_decoder(&mut format_reader);
            if let Ok(decoder) = decoder {
                self.track_id = get_first_supported_track(format_reader.tracks())
                    .unwrap()
                    .id;
                self.format_reader = Some(format_reader);
                self.decoder = Some(decoder);
                Ok(())
            } else {
                Err(decoder.err().unwrap())
            }
        } else {
            Err(format_reader.err().unwrap())
        }
    }

    fn seek(&mut self, ts: u64) {
        let seek_to = SeekTo::Time {
            time: Time::from(ts),
            track_id: Some(self.track_id),
        };
        self.seek_ts_seconds = if let (Some(format_reader), Some(decoder)) =
            (self.format_reader.borrow_mut(), self.decoder.borrow_mut())
        {
            decoder.reset();
            match format_reader.seek(SeekMode::Accurate, seek_to) {
                Ok(seeked_to) => seeked_to.required_ts,
                Err(symphonia::core::errors::Error::ResetRequired) => 0,
                Err(err) => {
                    // Don't give-up on a seek error.
                    warn!("seek error: {}", err);
                    0
                }
            }
        } else {
            0
        }
    }
}

fn get_format_reader(audio: &AudioFile) -> symphonia::core::errors::Result<Box<dyn FormatReader>> {
    // Create a hint to help the format registry guess what format reader is appropriate.
    let mut hint = Hint::new();

    // Provide the file extension as a hint.
    if let Some(extension) = audio.audio_path.extension() {
        if let Some(extension_str) = extension.to_str() {
            hint.with_extension(extension_str);
        }
    }

    let source = Box::new(File::open(&audio.audio_path)?);

    // Create the media source stream using the boxed media source from above.
    let mss = MediaSourceStream::new(source, Default::default());

    // Use the default options for format readers other than for gapless playback.
    let format_opts = FormatOptions {
        enable_gapless: true,
        ..Default::default()
    };

    // Use the default options for metadata readers.
    let metadata_opts: MetadataOptions = Default::default();

    // Probe the media source stream for metadata and get the format reader.
    match symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts) {
        Ok(probed) => Ok(probed.format),
        Err(err) => {
            // The input was not supported by any format reader.
            info!("the input is not supported");
            Err(err)
        }
    }
}

fn get_decoder(
    reader: &mut Box<dyn FormatReader>,
) -> symphonia::core::errors::Result<Box<dyn Decoder>> {
    let track = get_first_supported_track(reader.tracks()).unwrap();
    // Set the decoder options.
    let decode_opts = DecoderOptions { verify: true };
    symphonia::default::get_codecs().make(&track.codec_params, &decode_opts)
}

fn get_first_supported_track(tracks: &[Track]) -> Option<&Track> {
    tracks
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
}
