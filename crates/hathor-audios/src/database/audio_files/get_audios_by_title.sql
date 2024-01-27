SELECT *
FROM AUDIOS
WHERE AUDIO_TITLE LIKE '%' || :audio_title || '%';
