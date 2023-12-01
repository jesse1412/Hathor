SELECT *
FROM SONGS
WHERE SONG_TITLE LIKE "%" || :song_title || "%";
