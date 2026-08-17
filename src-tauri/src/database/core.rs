use rusqlite::Connection;
use rusqlite::params;
use std::path::Path;
use rusqlite::OptionalExtension;
use rusqlite::types::ValueRef;

// path 
use crate::config::DB_PATH;

// sql tables
use crate::config::ASSETS_TABLE;
use crate::config::LINK_TABLE;

// columns for ASSETS_TABLE
use crate::config::ASSET_ID;
use crate::config::CURRENT_NAME;
use crate::config::CURRENT_PATH;
use crate::config::LOG_PATH_SQL;
use crate::config::CREATED_AT;

// colums for LINK_TABLE
use crate::config::NEW_PATH;
use crate::config::OLD_PATH;

use crate::config::TableData;

// get all the table names 
#[tauri::command]
pub fn get_table_name(root_path: &str) -> Result<Vec<String>, String> {
    let mut tables= Vec::new();
    let db_path = std::path::Path::new(root_path)
        .join(DB_PATH)
        .to_string_lossy()
        .into_owned();
    
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare(
        "
        SELECT name
        FROM sqlite_schema
        WHERE type = 'table'
        AND name NOT LIKE 'sqlite_%'
        ORDER BY name;
        ",
    ).map_err(|e| e.to_string())?;
    
    let table_iter = stmt.query_map([], |row| {
        row.get::<_, String>(0)
    }).map_err(|e| e.to_string())?;
    
    for table in table_iter {
        tables.push(table.map_err(|e| e.to_string())?);
    }
    
    Ok(tables)
}

// get the entries in the table
#[tauri::command]
pub fn get_table_entries(root_path: &str, table_name: &str) -> Result<TableData, String> {
    let db_path = std::path::Path::new(root_path).join(DB_PATH);
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

    let query = format!("SELECT * FROM \"{}\"", table_name.replace('"', "\"\""));
    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

    // 1. Extract column names directly from the prepared query statement
    let columns = stmt.column_names().into_iter().map(String::from).collect();

    // 2. Map each row into a Vec<String>
    let rows = stmt
        .query_map([], |row| {
            (0..row.as_ref().column_count())
                .map(|i| match row.get_ref(i)? {
                    ValueRef::Null => Ok(String::new()), // or "NULL"
                    ValueRef::Text(t) => String::from_utf8(t.to_vec()).map_err(|_| rusqlite::Error::InvalidQuery),
                    ValueRef::Integer(n) => Ok(n.to_string()),
                    ValueRef::Real(f) => Ok(f.to_string()),
                    ValueRef::Blob(b) => Ok(format!("<BLOB {} B>", b.len())),
                })
                .collect()
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<Vec<String>>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(TableData { columns, rows })
}

// generate the asset id
#[tauri::command]
pub fn  get_new_asset_id(root_path: &str, filename: &str) -> Result<String, String> {
    let db_path = std::path::Path::new(root_path)
        .join(DB_PATH)
        .to_string_lossy()
        .into_owned();
    let mut conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
    
    let next_id = super::helper::increment_and_get_counter(&mut conn)?;
    let clean_name = super::helper::sanitize_name(filename);
    
    Ok(format!("{}_{}", clean_name, next_id))
}

// get new asset id for the view only no increment 
#[tauri::command]
pub fn  view_new_asset_id(root_path: &str, filename: &str) -> Result<String, String> {
    let db_path = std::path::Path::new(root_path)
        .join(DB_PATH)
        .to_string_lossy()
        .into_owned();
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
    
    let next_id = super::helper::get_counter(&conn)?;
    let clean_name = super::helper::sanitize_name(filename);
    
    Ok(format!("{}_{}", clean_name, next_id))
}

// add or update db for one asset id
#[tauri::command]
pub fn update_db(root_path: &str, relative_file_path: &str) -> Result<(), String> {
    let db_path = std::path::Path::new(root_path)
        .join(DB_PATH)
        .to_string_lossy()
        .into_owned();
    let conn = Connection::open(&db_path).map_err(|e: rusqlite::Error| e.to_string())?;    

    let (asset_id, _) = crate::string_formating::get_assetid_version_path(relative_file_path, root_path)?;
    let name = crate::file_system::get_filename(root_path, &relative_file_path)?;
    let created_at = crate::git::get_first_commit_creation_date(root_path,&asset_id)?;
    let log_path = crate::file_system::get_log_path(&asset_id)?;

    let query = format!(
        "INSERT INTO {} ({}, {}, {}, {}, {}) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT({}) DO UPDATE SET {} = ?2, {} = ?3, {} = ?4;",
        ASSETS_TABLE,
        ASSET_ID, CURRENT_NAME, CURRENT_PATH, LOG_PATH_SQL, CREATED_AT,
        ASSET_ID,
        CURRENT_NAME, CURRENT_PATH, LOG_PATH_SQL,
    );

    conn.execute(
        &query,
        params![asset_id, name, relative_file_path, log_path, created_at],
    ).map_err(|e| e.to_string())?;

    Ok(())
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
        
        for (i, asset) in chunk.iter().enumerate() {
            let base_idx = i * 5;
            
            values_clauses.push(format!(
                "(?{}, ?{}, ?{}, ?{}, ?{})", 
                base_idx + 1, base_idx + 2, base_idx + 3, base_idx + 4, base_idx + 5
            ));
            
            params.push(asset.asset_id.clone());
            params.push(asset.current_name.clone());
            params.push(asset.current_path.clone());
            params.push(asset.log_path.clone());
            params.push(asset.created_at.clone());
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
pub fn get_assetid_path(root_path: &str, relative_file_path: &str) -> Result<String, String> {
    let db_path = Path::new(root_path)
        .join(DB_PATH)
        .to_string_lossy()
        .into_owned();
    let conn = Connection::open(&db_path).map_err(|e: rusqlite::Error| e.to_string())?;

    let query = format!("SELECT {} FROM {} WHERE {} = ?1", ASSET_ID, ASSETS_TABLE, CURRENT_PATH);
    conn.query_row(
        &query, 
        [relative_file_path],
        |row| row.get(0),
    ).map_err(|e| e.to_string())
}

// update link table 
#[tauri::command]
pub fn update_link(root_path: &str, old_relative_file_path: &str, new_relative_file_path: &str) -> Result<(), String> {
    let db_path = Path::new(root_path).join(DB_PATH);
    let mut conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // 1. Remove any stale row currently claiming the new_path
    let clear_new_path_query = format!("DELETE FROM {} WHERE {} = ?1;", LINK_TABLE, NEW_PATH);
    tx.execute(&clear_new_path_query, params![new_relative_file_path])
        .map_err(|e| e.to_string())?;

    // 2. Insert or update the new 1-to-1 link for old_path
    let upsert_query = format!(
        "INSERT INTO {} ({}, {}) VALUES (?1, ?2)
         ON CONFLICT({}) DO UPDATE SET {} = ?2;",
        LINK_TABLE, OLD_PATH, NEW_PATH, OLD_PATH, NEW_PATH
    );
    tx.execute(&upsert_query, params![old_relative_file_path, new_relative_file_path])
        .map_err(|e| e.to_string())?;

    // 3. Commit the transaction
    tx.commit().map_err(|e| e.to_string())?;

    Ok(())
}

// delete a link from the table 
#[tauri::command]
pub fn delete_link(root_path: &str, new_relative_file_path: &str) -> Result<(), String> {
    let db_path = Path::new(root_path)
        .join(DB_PATH)
        .to_string_lossy()
        .into_owned();
    let conn = Connection::open(&db_path).map_err(|e: rusqlite::Error| e.to_string())?;
    
    let query = format!("DELETE FROM {} WHERE {} = ?1", LINK_TABLE, NEW_PATH);
    conn.execute(&query, [new_relative_file_path])
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

// get the paths in db which does not exist
#[tauri::command]
pub fn get_missing_path(root_path: &str) -> Result<Vec<String>, String> {
    let root = Path::new(root_path);
    let db_path = root.join(DB_PATH);
    let conn = Connection::open(&db_path).map_err(|e: rusqlite::Error| e.to_string())?;

    let query = format!("SELECT {} FROM {};", CURRENT_PATH, ASSETS_TABLE);
    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

    let missing_paths = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .filter_map(|res| res.ok()) // Ignore SQLite read errors or unwrap them
        .filter(|rel_path| !root.join(rel_path).exists()) // Keep only if missing on disk
        .collect();
    
    Ok(missing_paths)
}

// get the old path for assetid from new path
#[tauri::command]
pub fn get_old_path(root_path: &str, new_relative_file_path: &str) -> Result<Option<String>, String> {
    let db_path = Path::new(root_path)
        .join(DB_PATH)
        .to_string_lossy()
        .into_owned();
    let conn = Connection::open(&db_path).map_err(|e: rusqlite::Error| e.to_string())?;

    let query = format!("SELECT {} FROM {} WHERE {} = ?1", OLD_PATH, LINK_TABLE, NEW_PATH);
    let old_path: Option<String> = conn
    .query_row(&query, [new_relative_file_path], |row| row.get(0),)
    .optional()
    .map_err(|e| e.to_string())?;

    Ok(old_path)
}