use rusqlite::Connection;
use std::path::Path;
use std::collections::HashSet;

// path 
use crate::config::DB_PATH;

// sql
use crate::config::ASSETS_TABLE;
use crate::config::COUNTER_ID;
use crate::config::COUNTER_TABLE;

// columns for ASSETS_TABLE
use crate::config::ASSET_ID;

// columns for COUNTER_TABLE
use crate::config::ID;
use crate::config::NEXT_ASSET_ID;

// get the counter value and Atomically reads/increments/writes the next_asset_id counter
pub fn increment_and_get(conn: &mut Connection) -> Result<i32, String> { // HELPER FUCNTION
    // Start an exclusive SQLite transaction to prevent race conditions
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    // 1. Get the current counter value (Targeting the single row where id = 0)
    let select_query = format!("SELECT {} FROM {} WHERE {} = ?1;", NEXT_ASSET_ID, COUNTER_TABLE, ID);
    let current: i32 = tx.query_row(
        &select_query,
        [COUNTER_ID], 
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    
    let next = current + 1;
    
    // 2. Update the counter value to the incremented one
    let update_query = format!("UPDATE {} SET {} = ?1 WHERE {} = ?2;", NEXT_ASSET_ID, COUNTER_TABLE, ID);
    tx.execute(
        &update_query,
        [next, COUNTER_ID], 
    ).map_err(|e| e.to_string())?;
    
    // 3. Commit the transaction to save it to disk
    tx.commit().map_err(|e| e.to_string())?;
    
    Ok(next)
}

// Filename sanitization
pub fn sanitize_name(filename: &str) -> String { // HELPER FUCNTION
    let stem = filename.split('.').next().unwrap_or(filename);
    stem.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

// Identifies committed Git assets that have not yet been registered in the database.
pub fn get_missing_db_assets(root_path: &str) -> Result<Vec<(String, String, String, String, String,)>, String> {
    let db_path = Path::new(root_path)
        .join(DB_PATH)
        .to_string_lossy()
        .into_owned();
    let conn = Connection::open(&db_path).map_err(|e: rusqlite::Error| e.to_string())?;    
    let query = format!("SELECT {} FROM {};", ASSET_ID, ASSETS_TABLE);
    
    let commit_files_paths = crate::git::get_commited_files(root_path)?;
    
    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
    let id_iter = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    
    let mut asset_ids_db = HashSet::new();
    for id in id_iter {
        asset_ids_db.insert(id.map_err(|e| e.to_string())?);
    }
    
    let mut asset_ids_missing = Vec::new();
    for relative_file_path in commit_files_paths {
        let (asset_id, _) = crate::git::get_assetid_version(&relative_file_path, root_path)?;
        if !asset_ids_db.contains(&asset_id) {
            let (name, created_at) = crate::file_system::get_filename_createdat(&relative_file_path, root_path)?;
            let log_path = crate::file_system::get_log_path(&relative_file_path, root_path)?;
            asset_ids_missing.push((asset_id, name, relative_file_path, log_path, created_at));
        }
    }
    
    Ok(asset_ids_missing)
}
