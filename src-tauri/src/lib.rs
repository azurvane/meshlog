use std::sync::Mutex;
use notify_debouncer_mini::{
    Debouncer, 
    notify::{
        RecommendedWatcher
    }
};

mod initialize;
use initialize::initialize_project;
mod system;
use system::get_user_info;
mod git;
use git::{
    stage_commit_tag, 
    get_uncommited_files,
    get_existing_uncommited_files,
    detect_renamed_files,
    get_tag, 
    get_tag_assetid, 
    get_latest_tag_assetid, 
    get_all_hash_assetid,
    get_latest_hash_assetid,
    generate_tag,
    get_latest_tag_relative_path,
};
mod stamp;
use stamp::commit_stamp;
mod file_system;
use file_system::{
    get_log_files,
    get_file_flat, 
    get_file_tree, 
    get_file_metadata,
    get_directory_metadata,
};
mod database;
use database::{
    get_table_name,
    get_table_entries,
    get_new_asset_id,
    view_new_asset_id,
    update_db,
    populate_db,
    get_assetid_path,
    update_link,
    delete_link,
    get_missing_path,
    get_old_path
};
mod log_manager;
use log_manager::{
    get_log_content,
    populate_log_md,
    update_log_md,
};
mod string_formating;
use string_formating::{
    stamp_version,
};
mod watcher;
use watcher::{
    start_watching,
    stop_watching
};

mod config;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(Mutex::new(None::<Debouncer<RecommendedWatcher>>))
        .invoke_handler(tauri::generate_handler![
            get_user_info,
            initialize_project,
            get_log_files,
            get_file_flat,
            get_file_tree,
            stage_commit_tag,
            get_table_name,
            get_table_entries,
            get_new_asset_id,
            view_new_asset_id,
            update_db,
            get_uncommited_files,
            get_existing_uncommited_files,
            detect_renamed_files,
            get_tag,
            get_tag_assetid,
            get_latest_tag_assetid,
            get_all_hash_assetid,
            get_latest_hash_assetid,
            generate_tag,
            get_latest_tag_relative_path,
            commit_stamp,
            populate_db,
            get_assetid_path,
            update_link,
            delete_link,
            get_missing_path,
            get_old_path,
            get_file_metadata,
            get_directory_metadata,
            get_log_content,
            populate_log_md,
            update_log_md,
            stamp_version,
            start_watching,
            stop_watching
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}