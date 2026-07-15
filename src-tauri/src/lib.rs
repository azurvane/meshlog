mod initialize;
use initialize::initialize_project;
mod system;
use system::get_user_info;
mod git;
use git::{
    stage_commit_tag, 
    get_commit, 
    get_tag, 
    get_tag_assetid, 
    get_latest_tag_assetid, 
    get_all_hash_assetid,
    get_latest_hash_assetid,
    generate_tag,
};
mod file_system;
use file_system::{
    list_asset_files, 
    get_file_tree, 
    get_file_metadata,
};
mod database;
use database::{
    mint_asset_id,
    populate_db,
    get_assetid_path
};

mod config;

// fetch the version history of the file by asset id

// link the rename or relocated files to the git history

// update md log files after the commit

// create the log file if not exist

// update db after first commit of a file

// update db after nth commit of a file

// update db for rename or relocated file

// background function which will listen to the os for file name or location change

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            get_user_info,
            initialize_project,
            list_asset_files,
            get_file_tree,
            stage_commit_tag,
            mint_asset_id,
            get_commit,
            get_tag,
            get_tag_assetid,
            get_latest_tag_assetid,
            get_all_hash_assetid,
            get_latest_hash_assetid,
            generate_tag,
            populate_db,
            get_assetid_path,
            get_file_metadata
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}