use rusqlite::Connection;
use std::path::Path;

// path 
use crate::config::DB_PATH;

// sql
use crate::config::ASSETS_TABLE;

// columns for ASSETS_TABLE
use crate::config::ASSET_ID;
use crate::config::CURRENT_NAME;
use crate::config::CURRENT_PATH;
use crate::config::LOG_PATH_SQL;
use crate::config::CREATED_AT;

// generate the asset id
#[tauri::command]
pub fn mint_asset_id(root_path: &str, filename: &str) -> Result<String, String> {
    let db_path = std::path::Path::new(root_path)
        .join(DB_PATH)
        .to_string_lossy()
        .into_owned();
    let mut conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
    
    let next_id = super::helper::increment_and_get(&mut conn)?;
    let clean_name = super::helper::sanitize_name(filename);
    
    Ok(format!("{}_{}", clean_name, next_id))
}

// populate db with all or missing files in the assets folder
#[tauri::command]
pub fn populate_db(root_path: &str) -> Result<(), String> {
    let missing_assets = super::helper::get_missing_db_assets(root_path)?;
    if missing_assets.is_empty() {
        return Ok(());
    }
    
    let db_path = std::path::Path::new(root_path)
        .join(DB_PATH)
        .to_string_lossy()
        .into_owned();
    
    let mut conn = Connection::open(&db_path).map_err(|e: rusqlite::Error| e.to_string())?;    
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    for chunk in missing_assets.chunks(100) {
        let mut values_clauses = Vec::new();
        let mut params: Vec<String> = Vec::new();
        
        for (i, (asset_id,  name, relative_file_path, log_path, created_at)) in chunk.iter().enumerate() {
            let base_idx = i * 5;
            
            values_clauses.push(format!(
                "(?{}, ?{}, ?{}, ?{}, ?{})", 
                base_idx + 1, base_idx + 2, base_idx + 3, base_idx + 4, base_idx + 5
            ));
            
            params.push(asset_id.clone());
            params.push(name.clone());
            params.push(relative_file_path.clone());
            params.push(log_path.clone());
            params.push(created_at.clone());
        }
        
        let query = format!(
            "INSERT INTO {} ({}, {}, {}, {}, {}) VALUES {};",
            ASSETS_TABLE,
            ASSET_ID,
            CURRENT_NAME,
            CURRENT_PATH,
            LOG_PATH_SQL,
            CREATED_AT,
            values_clauses.join(", ")
        );
        
        let sql_params: Vec<&dyn rusqlite::ToSql> = params
            .iter()
            .map(|s| s as &dyn rusqlite::ToSql)
            .collect();
        
        tx.execute(&query, sql_params.as_slice()).map_err(|e| e.to_string())?;
    }
    
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

// get the asset id through path of the file
#[tauri::command]
pub fn get_assetid_path(relative_file_path: &str, root_path: &str) -> Result<String, String> {
    let db_path = Path::new(root_path)
        .join(DB_PATH)
        .to_string_lossy()
        .into_owned();
    let conn = Connection::open(&db_path).map_err(|e: rusqlite::Error| e.to_string())?;

    let query = format!("SELECT asset_id FROM {} WHERE current_path = ?1", ASSETS_TABLE);
    conn.query_row(
        &query, 
        [relative_file_path],
        |row| row.get(0),
    ).map_err(|e| e.to_string())
}
