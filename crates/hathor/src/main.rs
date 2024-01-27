use hathor_audios::audio::playback_manager::AudioManager;
// use hathor_audios::audio::symphonia_play_main::run;
use hathor_audios::audio::AudioFile;
// use hathor_audios::database;
use std::{thread, time::Duration};

fn main() {
    // let mut conn = database::get_connection(std::path::Path::new(".hathor.sqlite3")).unwrap();
    // let mut audios = Vec::new();
    // for _ in 0..1 {
    //     audios.push(
    //         AudioFile::from_file(std::path::Path::new(
    //             r"C:\Projects\rust\Hathor\test_media_files\audio\albums\album\test.mp3",
    //         ))
    //         .unwrap(),
    //     );
    // }
    // database::audios::insert_audios(&mut conn, &audios).unwrap();
    // database::playlists::insert_audios_into_playlist(&mut conn, "test", &audios).unwrap();
    let a = Box::new(
        AudioFile::from_file(std::path::Path::new(
            r"C:\Projects\rust\Hathor\test_media_files\audio\albums\album\test.mp3",
        ))
        .unwrap(),
    );
    let b = AudioManager::new();
    b.change_audio(a).unwrap();
    b.play().unwrap();
    thread::sleep(Duration::from_millis(2000));
    let a2 = AudioFile::from_file(std::path::Path::new(
        r"C:\Projects\rust\Hathor\test_media_files\audio\albums\album_with_cover_file\test.mp3",
    ))
    .unwrap();
    b.change_audio(Box::new(a2)).unwrap();
    thread::sleep(Duration::from_millis(2000));
    println!("Done");
}
