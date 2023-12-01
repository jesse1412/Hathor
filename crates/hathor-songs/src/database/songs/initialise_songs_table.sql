CREATE TABLE IF NOT EXISTS SONGS (
    FILE_HASH CHAR(64) PRIMARY KEY
    , SONG_TITLE VARCHAR(256)
    , ALBUM_NAME VARCHAR(256)
    , ARTIST_NAME VARCHAR(256)
    , TRACK_NUM INT(8)
    , RELEASE_YEAR INT(16)
    , SONG_LENGTH_SECONDS INT(64)
    , SONG_PATH VARCHAR(256) -- Windows path limit.
    , IMG_PATH VARCHAR(256) -- Windows path limit.
) WITHOUT ROWID;
