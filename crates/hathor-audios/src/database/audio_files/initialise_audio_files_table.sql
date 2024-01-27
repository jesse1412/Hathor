CREATE TABLE IF NOT EXISTS audio_files (
    file_hash CHAR(64)
    , audio_path VARCHAR(256) -- Windows path limit.
    , img_path VARCHAR(256) -- Windows path limit.
    , PRIMARY KEY (file_hash, audio_path)
) WITHOUT ROWID;
