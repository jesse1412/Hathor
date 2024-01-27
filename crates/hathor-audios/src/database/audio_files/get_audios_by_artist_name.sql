SELECT *
FROM audios
WHERE artist_name LIKE '%' || :artist_name || '%';
