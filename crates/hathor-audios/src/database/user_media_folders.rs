use rusqlite::named_params;
use rusqlite::Connection;
use std::error::Error;
use std::path::Path;

pub fn insert_user_media_folder(
    conn: &mut Connection,
    folder_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let transaction = conn.transaction()?;
    let mut statement = transaction.prepare(include_str!(
        r"user_media_folders/insert_user_media_folder.sql"
    ))?;
    let params = named_params! {"folder_path": folder_path.canonicalize()?.into_os_string().into_string().unwrap()};
    statement.execute(params)?;
    Ok(())
}
