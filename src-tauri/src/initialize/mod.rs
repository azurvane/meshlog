use std::path::Path;
use std::fs;
use std::process::Command;
use rusqlite::Connection;

// Import constants from the parent/crate config module
use crate::config::{
    DB_PATH, GIT_PATH, IMAGE_PATH, LOG_PATH,
    ASSETS_TABLE, COUNTER_ID, COUNTER_TABLE,
    ASSET_ID, CURRENT_NAME, CURRENT_PATH, LOG_PATH_SQL, CREATED_AT,
    ID, NEXT_ASSET_ID
};

// setup the folder and create necessary folder and files for the app
#[tauri::command]
pub fn initialize_project(root_path: &str) -> Result<String, String> {
    
    let git_path = format!("{}/{}", root_path, GIT_PATH);
    if !Path::new(&git_path).exists() {
        run_git_init(root_path)?;
    }
    
    let log_path = format!("{}/{}", root_path, LOG_PATH);
    if !Path::new(&log_path).exists() {
        fs::create_dir(&log_path).map_err(|e| e.to_string())?;
    }
    
    let image_path = format!("{}/{}", root_path, IMAGE_PATH);
    if !Path::new(&image_path).exists() {
        fs::create_dir(&image_path).map_err(|e| e.to_string())?;
    }
    
    let db_path = format!("{}/{}", root_path, DB_PATH);
    if !Path::new(&db_path).exists() {
        let db = Connection::open(&db_path).map_err(|e| e.to_string())?;
        initialise_assets_tables(&db).map_err(|e| e.to_string())?;
        initialise_counters_tables(&db).map_err(|e| e.to_string())?;
    }
    else {
        let db = Connection::open(&db_path).map_err(|e| e.to_string())?;
        verify_database_state(&db).map_err(|e| e.to_string())?;
    }
    
    Ok("Project initialized".to_string())
}

// initialise the asset table for the assets.sqlite
fn initialise_assets_tables(conn: &Connection) -> Result<(), String> {
    let query = format!(
        "CREATE TABLE IF NOT EXISTS {} (
            {}      TEXT PRIMARY KEY,
            {}      TEXT NOT NULL,
            {}      TEXT NOT NULL,
            {}      TEXT NOT NULL,
            {}      TEXT DEFAULT (strftime('%H:%M:%S', 'now', 'localtime') || '-' || strftime('%d-%m-%Y', 'now', 'localtime'))
        );",
        ASSETS_TABLE,
        ASSET_ID,
        CURRENT_NAME,
        CURRENT_PATH,
        LOG_PATH_SQL,
        CREATED_AT
    );

    conn.execute(&query, []).map_err(|e| e.to_string())?;
    
    Ok(())
}

// initialise the counter table for the assets.sqlite
fn initialise_counters_tables(conn: &Connection) -> Result<(), String> {
    let create_table_query = format!(
        "CREATE TABLE IF NOT EXISTS {} (
            {} INTEGER PRIMARY KEY CHECK ({} = {}),
            {} INTEGER NOT NULL
        );",
        COUNTER_TABLE, 
        ID,
        ID,
        COUNTER_ID,
        NEXT_ASSET_ID
    );
    conn.execute(&create_table_query, []).map_err(|e| e.to_string())?;

    let insert_query = format!("INSERT OR IGNORE INTO {} ({}, {}) VALUES (0, 1)", COUNTER_TABLE, ID, NEXT_ASSET_ID);
    conn.execute(&insert_query, []).map_err(|e| e.to_string())?;

    Ok(())
}

// verify if both tables are present 
fn verify_database_state(conn: &Connection) -> Result<(), String> {
    // 1. Check if 'ASSETS_TABLE' table exists
    let assets_query = format!("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='{}';", ASSETS_TABLE);
    let assets_exists: i64 = conn.query_row(&assets_query, [], |row| row.get(0)).map_err(|e| e.to_string())?;
    
    if assets_exists == 0 {
        println!("'{}' table missing! Initialising...", ASSETS_TABLE);
        initialise_assets_tables(conn)?;
    }
    
    // 2. Check if 'COUNTER_TABLE' table exists
    let counters_query = format!("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='{}';", COUNTER_TABLE);
    let counters_exists: i64 = conn.query_row(&counters_query, [], |row| row.get(0)).map_err(|e| e.to_string())?;
    
    if counters_exists == 0 {
        println!("'{}' table missing! Initialising...", COUNTER_TABLE);
        initialise_counters_tables(conn)?;
    } else {
        // 3. If COUNTER_TABLE exists, validate its contents
        let validate_query = format!("SELECT COUNT(*), MAX({}) FROM {};", NEXT_ASSET_ID, COUNTER_TABLE);
        let (row_count, counter_value): (i64, Option<i64>) = conn.query_row(
            &validate_query, 
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).map_err(|e| e.to_string())?;
        
        if row_count == 0 {
            println!("'{}' table is empty! Dropping and re-initialising...", COUNTER_TABLE);
            let drop_query = format!("DROP TABLE {};", COUNTER_TABLE);
            conn.execute(&drop_query, []).map_err(|e| e.to_string())?;
            initialise_counters_tables(conn)?;
        } else if row_count > 1 {
            return Err(format!("Database corruption: '{}' table has {} rows (expected exactly 1).", COUNTER_TABLE, row_count));
        } else {
            match counter_value {
                Some(val) if val > 0 => {
                    println!("Verification successful! {} is valid: {}", NEXT_ASSET_ID, val);
                }
                Some(val) => {
                    return Err(format!("Validation failed: {} is not positive ({})", NEXT_ASSET_ID, val));
                }
                None => {
                    return Err(format!("Validation failed: {} is NULL.", NEXT_ASSET_ID));
                }
            }
        }
    }
    
    Ok(())
}

// initialise the git if not present 
fn run_git_init(root_path: &str) -> Result<String, String> { // HELPER FUCNTION
    let output = Command::new("git")
    .arg("init")
    .current_dir(root_path)
    .output()
    .map_err(|e| e.to_string())?;

    if output.status.success() {
        let text = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
        Ok(text)
    } else {    
        let error_text = String::from_utf8(output.stderr).map_err(|e| e.to_string())?;
        Err(error_text)
    }
}
