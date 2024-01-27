SELECT *
FROM audios
WHERE audio_title LIKE '%' || :audio_title || '%';
