INSERT OR IGNORE INTO audios VALUES (
    :file_hash
    , :audio_title
    , :album_name
    , :artist_name
    , :track_num
    , :release_year
    , :audio_length_s
);
