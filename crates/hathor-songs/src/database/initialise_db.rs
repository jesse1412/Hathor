use rusqlite::Connection;

pub(crate) fn init_db(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute(include_str!("initialise_playlists_table.sql"), ())?;
    conn.execute(include_str!("initialise_songs_table.sql"), ())?;
    conn.execute(include_str!("initialise_user_media_folders_table.sql"), ())?;
    Ok(())
}

#[cfg(test)]
mod database_connect {
    use super::init_db;
    use rusqlite::Connection;

    #[test]
    fn test_table_creation_doesnt_fail_on_connect() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).expect("Database setup error");
    }
}
