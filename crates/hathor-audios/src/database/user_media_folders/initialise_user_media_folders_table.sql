CREATE TABLE IF NOT EXISTS user_media_folders (
    folder_path VARCHAR(256) PRIMARY KEY -- Windows path limit.
) WITHOUT ROWID;
