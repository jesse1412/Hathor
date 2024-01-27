SELECT
    audios.file_hash
    , audios.audio_title
    , audios.album_name
    , audios.artist_name
    , audios.track_num
    , audios.release_year
    , audios.audio_length_seconds
    , audio_files.audio_path
    , audio_files.img_path
FROM audios
    INNER JOIN audio_files
        ON
            audios.file_hash = :file_hash
            AND audio_files.file_hash = :file_hash
LIMIT 1;
