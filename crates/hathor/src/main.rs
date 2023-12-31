use hathor_songs::audio::AudioFile;
use hathor_songs::database;

fn main() {
    let mut conn = database::get_connection(std::path::Path::new(".hathor.sqlite3")).unwrap();
    let mut songs = Vec::new();
    for _ in 0..1 {
        songs.push(
            AudioFile::from_file(std::path::Path::new(
                r"C:\Projects\rust\Hathor\test_media_files\audio\albums\album\test.mp3",
            ))
            .unwrap(),
        );
    }
    database::songs::insert_songs(&mut conn, &songs).unwrap();
    database::playlists::insert_songs_into_playlist(&mut conn, "test", &songs).unwrap();
}
