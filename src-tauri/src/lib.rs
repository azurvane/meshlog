mod initialize;
use initialize::initialize_project;
mod system;
use system::get_user_info;
mod git;
use git::{
    stage_commit_tag, 
    get_uncommited_files,
    get_tag, 
    get_tag_assetid, 
    get_latest_tag_assetid, 
    get_all_hash_assetid,
    get_latest_hash_assetid,
    generate_tag,
};
mod file_system;
use file_system::{
    get_file_flat, 
    get_file_tree, 
    get_file_metadata,
};
mod database;
use database::{
    get_new_asset_id,
    view_new_asset_id,
    update_db,
    populate_db,
    get_assetid_path
};
mod log_manager;
use log_manager::{
    populate_log_md,
    populate_log_md_assetid,
};
mod string_formating;
use string_formating::{
    stamp_version,
};

mod config;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            get_user_info,
            initialize_project,
            get_file_flat,
            get_file_tree,
            stage_commit_tag,
            get_new_asset_id,
            view_new_asset_id,
            update_db,
            get_uncommited_files,
            get_tag,
            get_tag_assetid,
            get_latest_tag_assetid,
            get_all_hash_assetid,
            get_latest_hash_assetid,
            generate_tag,
            populate_db,
            get_assetid_path,
            get_file_metadata,
            populate_log_md,
            populate_log_md_assetid,
            stamp_version
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}