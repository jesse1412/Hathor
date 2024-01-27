WITH MATCHING_PLAYLISTS AS (
    SELECT PLAYLISTS.FILE_HASH
    FROM PLAYLISTS
    WHERE PLAYLISTS.PLAYLIST_NAME = :playlist_name
)

SELECT
    AUDIOS.FILE_HASH
    , AUDIOS.AUDIO_TITLE
    , AUDIOS.ALBUM_NAME
    , AUDIOS.ARTIST_NAME
    , AUDIOS.TRACK_NUM
    , AUDIOS.RELEASE_YEAR
    , AUDIOS.AUDIO_LENGTH_SECONDS
    , AUDIOS.AUDIO_PATH
    , AUDIOS.IMG_PATH
FROM MATCHING_PLAYLISTS
INNER JOIN AUDIOS
    ON
        MATCHING_PLAYLISTS.FILE_HASH = AUDIOS.FILE_HASH;