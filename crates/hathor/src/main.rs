use hathor_songs::audio::AudioFile;
use hathor_songs::database;

fn main() {
    let mut conn = database::get_connection().unwrap();
    let mut songs = Vec::new();
    for _ in 0..=100000 {
        songs.push(
            AudioFile::from_file(std::path::Path::new(r"C:\Projects\rust\Hathor\test3.txt"))
                .unwrap(),
        );
    }
    database::insert_songs(&mut conn, &songs).unwrap();
}
