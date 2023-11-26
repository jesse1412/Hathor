use rusqlite::Connection;

pub fn init_db(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let sql = include_str!("initialise_db.sql");
    conn.execute(sql, ())?;
    Ok(())
}
