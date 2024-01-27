use super::playback::do_play_loop;
use super::AudioFile;
use eyre::Result;
use std::error::Error;
use std::sync::mpsc;
use std::sync::mpsc::SendError;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

pub enum AudioCommand {
    ChangeAudio(Box<AudioFile>),
    Pause,
    Play,
    ResetPlayback,
    Seek(u64),
}

/// High level struct to manage an audio thread.
pub struct AudioManager {
    send_to_playback_tx: Sender<AudioCommand>,
    receive_from_playback_rx: Receiver<eyre::Result<()>>,
}

impl AudioManager {
    pub fn new() -> Self {
        let (sender_to_playback, receiver_from_audio_manager) = mpsc::channel();
        let (sender_to_audio_manager, receiver_from_playback) = mpsc::channel();
        thread::spawn(move || do_play_loop(receiver_from_audio_manager, sender_to_audio_manager));
        AudioManager {
            send_to_playback_tx: sender_to_playback,
            receive_from_playback_rx: receiver_from_playback,
        }
    }

    /// Pause audio playback.
    pub fn pause(&self) -> Result<(), Box<dyn Error>> {
        self.send_to_playback_tx.send(AudioCommand::Pause)?;
        self.receive_from_playback_rx.recv()?.unwrap();
        Ok(())
    }

    /// Continue/start audio playback.
    pub fn play(&self) -> Result<(), SendError<AudioCommand>> {
        self.send_to_playback_tx.send(AudioCommand::Play)
    }

    /// Go to the given timestamp in the current audio file.
    pub fn seek(&self, seek_ts_seconds: u64) -> Result<(), SendError<AudioCommand>> {
        self.send_to_playback_tx
            .send(AudioCommand::Seek(seek_ts_seconds))
    }

    /// Go to the start of the current audio file.
    pub fn reset_playback(&self) -> Result<(), SendError<AudioCommand>> {
        self.send_to_playback_tx.send(AudioCommand::Seek(0))
    }

    /// Change the audio to another track.
    pub fn change_audio(&self, audio: Box<AudioFile>) -> Result<(), SendError<AudioCommand>> {
        self.send_to_playback_tx
            .send(AudioCommand::ChangeAudio(audio))
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}
