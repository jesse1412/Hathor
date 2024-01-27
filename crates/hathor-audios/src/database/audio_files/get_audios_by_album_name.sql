SELECT *
FROM audios
WHERE album_name LIKE '%' || :album_name || '%';
