SELECT *
FROM SONGS
WHERE ARTIST_NAME LIKE '%' || :artist_name || '%';
