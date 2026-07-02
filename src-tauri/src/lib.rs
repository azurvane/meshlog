use core::num;
use std::path::Path;
use std::fs;
use std::process::Command;
use rusqlite::Connection;

mod config;

use config::COUNTER_ID;

// setup the folder and create necessary folder and files for the app
#[tauri::command]
fn initialize_project(path: &str) -> Result<String, String> {
    
    let git_path = format!("{}/.git", path);
    if !Path::new(&git_path).exists() {
        run_git_init(path)?;
    }
    
    let log_path = format!("{}/.logs", path);
    if !Path::new(&log_path).exists() {
        fs::create_dir(&log_path).map_err(|e| e.to_string())?;
    }
    
    let image_path = format!("{}/.images", path);
    if !Path::new(&image_path).exists() {
        fs::create_dir(&image_path).map_err(|e| e.to_string())?;
    }
    
    let db_path = format!("{}/.assets.sqlite", path);
    let db = Connection::open(&db_path).map_err(|e| e.to_string())?;
    if !Path::new(&db_path).exists() {
        fs::File::create(&db_path).map_err(|e| e.to_string())?;
        initialise_assets_tables(&db).map_err(|e| e.to_string())?;
        initialise_counters_tables(&db).map_err(|e| e.to_string())?;
    }
    else {
        verify_database_state(&db).map_err(|e| e.to_string())?;
    }
    
    Ok("Project initialized".to_string())
}

// initialise the asset table for the assets.sqlite
fn initialise_assets_tables(conn: &Connection) -> Result<(), String> { // HELPER FUCNTION
    // asset table 
    conn.execute(
        "CREATE TABLE IF NOT EXISTS assets (
            asset_id      INTEGER PRIMARY KEY AUTOINCREMENT,
            original_name TEXT NOT NULL,
            current_path  TEXT NOT NULL,
            log_path      TEXT NOT NULL,
            created_at    TEXT DEFAULT (strftime('%H:%M:%S', 'now', 'localtime') || '-' || strftime('%d-%m-%Y', 'now', 'localtime'))
        );",
        [], // Empty parameters because this query is static
    ).map_err(|e| e.to_string())?;
    
    Ok(())
}

// initialise the counter table for the assets.sqlite
fn initialise_counters_tables(conn: &Connection) -> Result<(), String> { // HELPER FUCNTION
    // counter table (single row)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS counters (
            id INTEGER PRIMARY KEY CHECK (id = ?1),
            next_asset_id INTEGER NOT NULL
        );",
        [COUNTER_ID], // Empty parameters because this query is static
    ).map_err(|e| e.to_string())?;

    // initialise the table 
    conn.execute(
        "INSERT OR IGNORE INTO counters (id, next_asset_id) VALUES (0, 1)",
        [], // Empty parameters because this query is static
    ).map_err(|e| e.to_string())?;

    Ok(())
}

// verify if both tables are present 
fn verify_database_state(conn: &Connection) -> Result<(), String> { // HELPER FUCNTION
    // 1. Check if 'assets' table exists
    let assets_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='assets';",
        [],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    if assets_exists == 0 {
        println!("'assets' table missing! Initialising...");
        initialise_assets_tables(conn)?;
    }

    // 2. Check if 'counters' table exists
    let counters_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='counters';",
        [],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    if counters_exists == 0 {
        println!("'counters' table missing! Initialising...");
        initialise_counters_tables(conn)?;
    } else {
        // 3. If counters exists, validate its contents
        let (row_count, counter_value): (i64, Option<i64>) = conn.query_row(
            "SELECT COUNT(*), MAX(next_asset_id) FROM counters;", 
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).map_err(|e| e.to_string())?;

        if row_count == 0 {
            println!("'counters' table is empty! Dropping and re-initialising...");
            conn.execute("DROP TABLE counters;", []).map_err(|e| e.to_string())?;
            initialise_counters_tables(conn)?;
        } else if row_count > 1 {
            return Err(format!("Database corruption: 'counters' table has {} rows (expected exactly 1).", row_count));
        } else {
            // Exactly 1 row exists, ensure it contains a positive value
            match counter_value {
                Some(val) if val > 0 => {
                    println!("Verification successful! next_asset_id is valid: {}", val);
                }
                Some(val) => {
                    return Err(format!("Validation failed: next_asset_id is not positive ({})", val));
                }
                None => {
                    return Err("Validation failed: next_asset_id is NULL.".to_string());
                }
            }
        }
    }

    Ok(())
}

// initialise the git if not present 
fn run_git_init(path: &str) -> Result<String, String> { // HELPER FUCNTION
    let output = Command::new("git")
    .arg("init")
    .current_dir(path)
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

// return all the files in path and subfolders exluding all the hidden files
#[tauri::command]
fn list_asset_files(path: &str) -> Result<Vec<String>, String> { 
    let mut names = Vec::new();
    let entries = fs::read_dir(path).map_err(|e| e.to_string())?;
    
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

// add and commit the files
#[tauri::command]
fn stage_commit_tag(path: &str, file_path: &str, summary: &str, tag: &str)  -> Result<String, String> { 
    let sub_path = Path::new(path).join(file_path);
    
    // git add
    let add_output = Command::new("git")
    .arg("add")
    .arg(&sub_path)
    .current_dir(path)
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
    .current_dir(path)
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
    .current_dir(path)
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
fn mint_asset_id(project_path: &str, filename: &str) -> Result<String, String> {
    let db_path = format!("{}/.assets.sqlite", project_path);
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
    let current: i32 = tx.query_row(
        "SELECT next_asset_id FROM counters WHERE id = ?1;",
        [COUNTER_ID], 
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    let next = current + 1;

    // 2. Update the counter value to the incremented one
    tx.execute(
        "UPDATE counters SET next_asset_id = ?1 WHERE id = ?2;",
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
fn get_commit(file_path: &str) -> Result<Vec<String>, String> { // USELESS FUNCTION FOR NOW AT LEAST
    let output = Command::new("git")
        .args(["log", "--oneline", "--graph", "--decorate"])
        .current_dir(file_path)
        .output() // Executes the command and captures stdout/stderr
        .expect("Failed to execute git command");
    
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
fn get_tag(file_path: &str) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .args(["tag", "--list"])
        .current_dir(file_path)
        .output()
        .expect("Failed to execute git command");
    
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
fn get_tag_assetid(asset_id: &str, file_path: &str) -> Result<Vec<String>, String> {
    let tags = get_tag(file_path)?;
    
    let filtered_tags: Vec<String> = tags
        .into_iter() // consumes the vector so we don't have to manually clone strings
        .filter(|tag| tag.contains(asset_id))
        .collect();
    
    Ok(filtered_tags)
}

// generate new tag
#[tauri::command]
fn generate_tag(asset_id: &str, file_path: &str) -> Result<String, String> {
    let tags = get_tag_assetid(asset_id, file_path)?;
    
    if let Some(lastest_tag) = tags.last() {
        let len = lastest_tag.len();
        let suffix = &lastest_tag[len - 4..];
        
        if let Ok(mut number) = suffix.parse::<u32>() {
            number += 1;
            
            let prefix = &lastest_tag[..len - 4];
            let new_tag = format!("{}{:4}", prefix, number);
            
            Ok(new_tag)
        } else {
            Err(format!("invalid last 4 digit: {}", suffix).to_string())
        }
    } else {
        Ok(format!("{}-v0.0.0001", asset_id))
    }
}

// // fetch the version history of the file by asset id
// fn get_version_history(conn: &Connection, asset_id: &str) -> Result<Vec<String>, String> {
// }

// // link the rename or relocated files to the git history
// fn link_renamed_asset(conn: &Connection, old_path: &str, new_path: &str, asset_id: &str) -> Result<String, String> {
// }

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            initialize_project,
            list_asset_files,
            stage_commit_tag,
            mint_asset_id,
            get_tag,
            get_tag_assetid,
            generate_tag
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}



// test cases to check the code without running the ui
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_function() {
        assert!(initialize_project("/Users/apple/Desktop/3d_version_test").is_ok());
    }

    #[test]
    fn run_listasset() {
        assert!(list_asset_files("/Users/apple/Desktop/3d_version_test").is_ok())
    }

    #[test]
    fn run_get_commit() {
        // Change the path below to your actual repository path
        let test_path = "/Users/apple/Documents/programing/projects/othello/";
        
        println!("\n--- STARTING GIT LOG CAPTURE TEST ---");
        match get_commit(test_path) {
            Ok(lines) => {
                println!("Successfully retrieved {} lines from git log.", lines.len());
                // The function itself already calls println!("{:?}", lines);
                // but let's make sure it's valid data by asserting it's not empty
                assert!(!lines.is_empty(), "Git log returned 0 lines. Is this a git repo with commits?");
            }
            Err(e) => {
                panic!("Test failed with Git error: {}", e);
            }
        }
        println!("--- END OF TEST ---\n");
    }

    #[test]
    fn run_get_tag() {
        // Change the path below to your actual repository path
        let test_path = "/Users/apple/Documents/programing/projects/othello/";
        
        println!("\n--- STARTING GIT TAG CAPTURE TEST ---");
        match get_tag(test_path) {
            Ok(lines) => {
                println!("Successfully retrieved {} lines from git tag.", lines.len());
                assert!(!lines.is_empty(), "Git tag returned 0 lines. Is this a git repo with commits?");
            }
            Err(e) => {
                panic!("Test failed with Git error: {}", e);
            }
        }
        println!("--- END OF TEST ---\n");
    }

    #[test]
    fn run_get_tag_assetid() {
        // Change the path below to your actual repository path
        let test_path = "/Users/apple/Desktop/3d_version_test/main_folder";
        let target_id = "file_009";
        
        println!("\n--- STARTING GIT TAG CAPTURE TEST ---");
        match get_tag_assetid(target_id,test_path) {
            Ok(lines) => {
                println!("Successfully checked tags for asset '{}'. Found {} matches.", target_id, lines.len());
                println!("Matching tags: {:?}", lines);
                }
            Err(e) => {
                panic!("Test failed with Git error: {}", e);
            }
        }
        println!("--- END OF TEST ---\n");
    }
}