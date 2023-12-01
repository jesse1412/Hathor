SELECT *
FROM SONGS
WHERE ALBUM_NAME LIKE '%' || :album_name || '%';
