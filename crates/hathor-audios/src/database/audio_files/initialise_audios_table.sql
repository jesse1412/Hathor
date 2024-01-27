CREATE TABLE IF NOT EXISTS audios (
    file_hash CHAR(64) PRIMARY KEY
    , audio_title VARCHAR(256)
    , album_name VARCHAR(256)
    , artist_name VARCHAR(256)
    , track_num INT(8)
    , release_year INT(16)
    , audio_length_seconds INT(64)
) WITHOUT ROWID;
