CREATE TABLE IF NOT EXISTS playlists (
    playlist_name VARCHAR(256)
    , file_hash CHAR(64)
    , PRIMARY KEY (playlist_name, file_hash)
) WITHOUT ROWID;
