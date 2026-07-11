use std::path::Path;
use std::fs;
use std::process::Command;
use rusqlite::Connection;
use serde::Serialize;
use chrono::{DateTime, Local};

mod config;

// paths
use config::DB_PATH;
use config::GIT_PATH;
use config::IMAGE_PATH;
use config::LOG_PATH;

// sql
use config::ASSETS_TABLE;
use config::COUNTER_ID;
use config::COUNTER_TABLE;

// columns for ASSETS_TABLE
use config::ASSET_ID;
use config::CURRENT_NAME;
use config::CURRENT_PATH;
use config::LOG_PATH_SQL;
use config::CREATED_AT;

// columns for COUNTER_TABLE
use config::ID;
use config::NEXT_ASSET_ID;

const NO_TAG_ERROR: &str = "No tag";

// file node data structure 
#[derive(Serialize)]
struct FileNode {
    name: String,
    is_dir: bool,
    children: Option<Vec<FileNode>>,
}

// file meta node data structure
#[derive(Serialize)]
struct FileMetadata {
    name: String,
    size_bytes: u64,
    modified_ddmmyyyy: String,
    created_ddmmyyyy: String,
    is_dir: bool,
    file_type: String,
    current_version: String,
    current_hash: String
}

// setup the folder and create necessary folder and files for the app
#[tauri::command]
fn initialize_project(root_path: &str) -> Result<String, String> {
    
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
// keep the path relative so if root folder name or location changed the db is still valid
fn initialise_assets_tables(conn: &Connection) -> Result<(), String> {
    let query = format!(
        "CREATE TABLE IF NOT EXISTS {} (
            {}      TEXT PRIMARY KEY,
            {}  TEXT NOT NULL,
            {}  TEXT NOT NULL,
            {}      TEXT NOT NULL,
            {}    TEXT DEFAULT (strftime('%H:%M:%S', 'now', 'localtime') || '-' || strftime('%d-%m-%Y', 'now', 'localtime'))
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

// return all files in path and subfolders exluding all the hidden files in flat view
#[tauri::command]
fn list_asset_files(absolute_folder_path: &str) -> Result<Vec<String>, String> { 
    let mut names = Vec::new();
    let entries = fs::read_dir(absolute_folder_path).map_err(|e| e.to_string())?;
    
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().into_string().map_err(|_| "bad filename".to_string())?;
        if name.starts_with(".") {
            continue;
        }
        
        let entry_path = entry.path();
        if entry_path.is_dir() {
            let path_str = entry_path.to_str().ok_or("invalid path encoding".to_string())?;
            let mut sub_files = list_asset_files(path_str)?;
            names.append(&mut sub_files);
        }
        else {
            names.push(name);
        }
    }
    
    Ok(names)
}

// return tree view of all files in path and subfolders exluding all the hidden files
#[tauri::command]
fn get_file_tree(absolute_folder_path: &str) -> Result<Vec<FileNode>, String> { 
    let mut nodes: Vec<FileNode> = Vec::new();
    let entries = fs::read_dir(absolute_folder_path).map_err(|e| e.to_string())?;
    
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_name = entry.file_name().into_string().map_err(|_| "bad filename".to_string())?;
        
        if file_name.starts_with(".") {
            continue;
        }
        
        let entry_path = entry.path();
        let is_directory = entry_path.is_dir();
        let mut child_node = None;
        
        if is_directory == true {
            let path_str = entry_path.to_str().ok_or("invalid path encoding".to_string())?;
            child_node = Some(get_file_tree(path_str)?);
        }
        
        let value = FileNode {
            name: file_name,
            is_dir: is_directory,
            children: child_node
        };
        
        nodes.push(value);
    }
    
    Ok(nodes)
}

// add and commit the files
#[tauri::command]
fn stage_commit_tag(root_path: &str, relative_file_path: &str, summary: &str, tag: &str)  -> Result<String, String> { 
    let sub_path = Path::new(root_path).join(relative_file_path);
    
    // git add
    let add_output = Command::new("git")
    .arg("add")
    .arg(&sub_path)
    .current_dir(root_path)
    .output()
    .map_err(|e| e.to_string())?;
    
    // If staging fails, exit early with the error
    if !add_output.status.success() {
        let error_text = String::from_utf8(add_output.stderr).map_err(|e| e.to_string())?;
        return Err(format!("Git add failed: {}", error_text));
    }
    
    // git commit
    let commit_output = Command::new("git")
    .arg("commit")
    .arg("-m")
    .arg(summary)
    .current_dir(root_path)
    .output()
    .map_err(|e| e.to_string())?;
    
    // Check if commit succeeds
    if !commit_output.status.success() {
        let error_text = String::from_utf8(commit_output.stderr).map_err(|e| e.to_string())?;
        return Err(format!("Git commit failed: {}", error_text))
    }
    
    // git tag
    let tag_output = Command::new("git")
    .arg("tag")
    .arg(tag)
    .current_dir(root_path)
    .output()
    .map_err(|e| e.to_string())?;
    
    // Check if commit succeeds
    if tag_output.status.success() {
        let text = String::from_utf8(tag_output.stdout).map_err(|e| e.to_string())?;
        Ok(text)
    } else {    
        let error_text = String::from_utf8(tag_output.stderr).map_err(|e| e.to_string())?;
        Err(format!("Git commit failed: {}", error_text))
    }
}

// generate the asset id
#[tauri::command]
fn mint_asset_id(root_path: &str, filename: &str) -> Result<String, String> {
    let db_path = format!("{}/{}", root_path, DB_PATH);
    let mut conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
    
    let next_id = increment_and_get(&mut conn)?;
    let clean_name = sanitize_name(filename);
    
    Ok(format!("{}-{}", clean_name, next_id))
}

// get the counter value and Atomically reads/increments/writes the next_asset_id counter
fn increment_and_get(conn: &mut Connection) -> Result<i32, String> { // HELPER FUCNTION
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
fn sanitize_name(filename: &str) -> String { // HELPER FUCNTION
    let stem = filename.split('.').next().unwrap_or(filename);
    stem.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

// get all the commits
fn get_commit(root_path: &str) -> Result<Vec<String>, String> { // USELESS FUNCTION FOR NOW AT LEAST
    let output = Command::new("git")
        .args(["log", "--oneline", "--graph", "--decorate"])
        .current_dir(root_path)
        .output() // Executes the command and captures stdout/stderr
        .map_err(|e| e.to_string())?;
    
    if output.status.success() {
        let git_history = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
        let git_history_vec: Vec<String> = git_history.lines().map(String::from).collect();
        // println!("{:?}", git_history_vec);
        Ok(git_history_vec)
    } else {    
        let error_text = String::from_utf8(output.stderr).map_err(|e| e.to_string())?;
        Err(error_text)
    }
}

// get all the tags
#[tauri::command]
fn get_tag(root_path: &str) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .args(["tag", "--list", "--sort=-creatordate"])
        .current_dir(root_path)
        .output()
        .map_err(|e| e.to_string())?;
    
    if output.status.success() {
        let tags = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
        let tag_vec: Vec<String> = tags.lines().map(String::from).collect();
        Ok(tag_vec)
    } else {
        let error_text = String::from_utf8(output.stderr).map_err(|e| e.to_string())?;
        Err(error_text)
    }
}

// get tag for a specific asset id
#[tauri::command]
fn get_tag_assetid(asset_id: &str, root_path: &str) -> Result<Vec<String>, String> {
    let pattern = format!("{}-v*", asset_id);
    
    let output = Command::new("git")
        .args(["tag", "--list", &pattern, "--sort=-creatordate"])
        .current_dir(root_path)
        .output()
        .map_err(|e| e.to_string())?;
    
    if output.status.success() {
        let tags = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
        let tag_vec: Vec<String> = tags.lines().map(String::from).collect();
        Ok(tag_vec)
    } else {
        let error_text = String::from_utf8(output.stderr).map_err(|e| e.to_string())?;
        Err(error_text)
    }
}

// get the latest tag for a specific asset id
#[tauri::command]
fn get_latest_tag_assetid(asset_id: &str, root_path: &str) -> Result<String, String> {
    let pattern = format!("{}-v*", asset_id);
    
    let output = Command::new("git")
        .args(["tag", "--list", &pattern, "--sort=-creatordate"])
        .current_dir(root_path)
        .output()
        .map_err(|e| e.to_string())?;
    
    if output.status.success() {
        let tags = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
        let tag = tags.lines().next().map(String::from).ok_or(NO_TAG_ERROR.to_string())?;
        Ok(tag)
    } else {
        let error_text = String::from_utf8(output.stderr).map_err(|e| e.to_string())?;
        Err(error_text)
    }
}

// get the hash for the commit for a specific tag
fn get_hash_assetid(tag: &str, root_path: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(["rev-list", "-n", "1", tag])
        .current_dir(root_path)
        .output()
        .map_err(|e| e.to_string())?;
    
    if output.status.success() {
        let tag = String::from_utf8(output.stdout)
            .map_err(|e| e.to_string())?
            .trim()
            .to_string();
        Ok(tag)
    } else {
        let error_text = String::from_utf8(output.stderr).map_err(|e| e.to_string())?;
        Err(error_text)
    }
}

// get all the hash for a specific asset id
#[tauri::command]
fn get_all_hash_assetid(asset_id: &str, root_path: &str) -> Result<Vec<String>, String> {
    let tags = get_tag_assetid(asset_id, root_path)?;
    let mut hashes = Vec::new();
    
    for tag in &tags {
        let hash = get_hash_assetid(tag, root_path)?;
        hashes.push(hash);
    };
    
    Ok(hashes)
}

// get the latest hash for a specific asset id
#[tauri::command]
fn get_latest_hash_assetid(asset_id: &str, root_path: &str) -> Result<String, String> {
    let tag = get_latest_tag_assetid(asset_id, root_path)?;
    let hash = get_hash_assetid(&tag, root_path)?;
    Ok(hash)
}

// generate new tag
#[tauri::command]
fn generate_tag(asset_id: &str, root_path: &str) -> Result<String, String> {
    match  get_latest_tag_assetid(asset_id, root_path) {
        Ok(tag) =>  {
            let len = tag.len();
            
            if len < 4 {
                return Err("Tag is too short to parse suffix".to_string());
            }
            
            let suffix = &tag[len - 4..];
            
            if let Ok(mut number) = suffix.parse::<u32>() {
                number += 1;
                
                let prefix = &tag[..len - 4];
                let new_tag = format!("{}{:04}", prefix, number);
                Ok(new_tag)
            } else {
                Err(format!("invalid last 4 digit: {}", suffix).to_string())
            }
        }
        // Match specifically against your constant
        Err(ref err) if err == NO_TAG_ERROR => {
            Ok(format!("{}-v00.00.0001", asset_id))
        }
        // Catch-all for actual Git failures or other errors
        Err(git_err) => {
            Err(format!("Git command failed: {}", git_err))
        }
    }
}

// fetch the version history of the file by asset id

// link the rename or relocated files to the git history

// update md log files after the commit

// create the log file if not exist

// update db after first commit of a file

// update db after nth commit of a file

// update db for rename or relocated file

// get the asset id through path of the file
#[tauri::command]
fn get_assetid_path(relative_path: &str, root_path: &str) -> Result<String, String> {
    let db_path = format!("{}/{}", root_path, DB_PATH);
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

    let query = format!("SELECT asset_id FROM {} WHERE current_path = ?1", ASSETS_TABLE);
    conn.query_row(
        &query, 
        [relative_path],
        |row| row.get(0),
    ).map_err(|e| e.to_string())
}

// get the meta data about the file to be displayed
#[tauri::command]
fn get_file_metadata(absolute_file_path: &str, root_path: &str) -> Result<FileMetadata, String> {
    let full_path = Path::new(absolute_file_path);
    let assets_root = Path::new(root_path);
    let relative_path = full_path.strip_prefix(assets_root).map_err(|e| e.to_string())?;
    let relative_path_str: &str = relative_path.to_str().ok_or("Path contains invalid UTF-8")?;
    let metadata_raw = fs::metadata(absolute_file_path).map_err(|e| e.to_string())?;
    
    // get the name of the file
    let file_name = full_path
        .file_name()
        .map(|os_str| os_str.to_string_lossy().into_owned())
        .unwrap_or_else(|| "".to_string());
    
    // get the size
    let size = metadata_raw.len();
    
    // get the time file was modifies
    // run this in background also so when the user make some changes it will get autoupdate
    let modified_system_time = metadata_raw.modified().map_err(|e| e.to_string())?;
    let modified_chrono: DateTime<Local> = modified_system_time.into();
    let modified_str = modified_chrono.format("%d-%m-%Y").to_string();
    
    // get the time file was created
    let created_system_time = metadata_raw.created().map_err(|e| e.to_string())?;
    let created_chrono: DateTime<Local> = created_system_time.into();
    let created_str = created_chrono.format("%d-%m-%Y").to_string();
    
    // is it directory?
    let is_directory: bool = metadata_raw.is_dir();
    
    // get file type/extension
    let file_type = if is_directory {
        "dir".to_string()
    } else {
        full_path.extension()
            .map(|ext| ext.to_string_lossy().into_owned())
            .unwrap_or_else(|| "file".to_string())
    };
    
    // get asset id 
    let asset_id = get_assetid_path(relative_path_str, root_path)?;
    
    // get the latest version
    let version = get_latest_tag_assetid(&asset_id, root_path)?;

    // get the latest hash
    let hash = get_hash_assetid(&version, root_path)?;
    
    // get latest tag for the file
    let file_metadata = FileMetadata{
        name: file_name,
        size_bytes: size,
        modified_ddmmyyyy: modified_str,
        created_ddmmyyyy: created_str,
        is_dir: is_directory,
        file_type,
        current_version: version,
        current_hash: hash
    };
    
    Ok(file_metadata)
}

// background function which will listen to the os for file name or location change

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            initialize_project,
            list_asset_files,
            get_file_tree,
            stage_commit_tag,
            mint_asset_id,
            get_tag,
            get_tag_assetid,
            get_latest_tag_assetid,
            get_all_hash_assetid,
            get_latest_hash_assetid,
            generate_tag,
            get_assetid_path,
            get_file_metadata
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}