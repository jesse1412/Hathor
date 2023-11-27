use rusqlite::Connection;

pub(crate) fn init_db(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute(include_str!("initialise_playlists_table.sql"), ())?;
    conn.execute(include_str!("initialise_songs_table.sql"), ())?;
    conn.execute(include_str!("initialise_user_media_folders_table.sql"), ())?;
    Ok(())
}
