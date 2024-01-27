WITH matching_playlists AS (
    SELECT playlists.file_hash
    FROM playlists
    WHERE playlists.playlist_name = :playlist_name
)

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
FROM matching_playlists
    INNER JOIN audios
        ON
            matching_playlists.file_hash = audios.file_hash
    INNER JOIN audio_files
        ON matching_playlists.file_hash = audio_files.file_hash;
