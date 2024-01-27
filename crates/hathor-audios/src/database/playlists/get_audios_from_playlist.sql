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
    , audios.audio_path
    , audios.img_path
FROM matching_playlists
    INNER JOIN audios
        ON
            matching_playlists.file_hash = audios.file_hash;
