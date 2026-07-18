use std::path::Path;
use std::fs;
use chrono::{DateTime, Local};

use crate::config::FileMetadata;
use crate::config::FileNode;

// return flat view of all files in path and subfolders exluding all the hidden files 
#[tauri::command]
pub fn get_file_flat(absolute_folder_path: &str) -> Result<Vec<String>, String> { 
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
            let mut sub_files = get_file_flat(path_str)?;
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
pub fn get_file_tree(absolute_folder_path: &str) -> Result<Vec<FileNode>, String> { 
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

// get the meta data about the file to be displayed
#[tauri::command]
pub fn get_file_metadata(absolute_file_path: &str, root_path: &str) -> Result<FileMetadata, String> {
    let full_path = Path::new(absolute_file_path);
    let assets_root = Path::new(root_path);
    let relative_file_path = full_path.strip_prefix(assets_root).map_err(|e| e.to_string())?;
    let relative_file_path_str: &str = relative_file_path.to_str().ok_or("Path contains invalid UTF-8")?;
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
    let asset_id = crate::database::get_assetid_path(relative_file_path_str, root_path)?;
    
    // get the latest version
    let version = crate::git::get_latest_tag_assetid(&asset_id, root_path)?;
    
    // get the latest hash
    let hash = crate::git::get_hash_assetid(&version, root_path)?;
    
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
