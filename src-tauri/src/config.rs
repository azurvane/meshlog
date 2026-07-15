use serde::Serialize;

// paths
pub const GIT_PATH: &str = ".git";
pub const LOG_PATH: &str = ".logs";
pub const IMAGE_PATH: &str = ".images";
pub const DB_PATH: &str = ".assets.sqlite";

// SQL table variables
pub const COUNTER_ID: i32 = 0;
pub const ASSETS_TABLE: &str = "assets";
pub const COUNTER_TABLE: &str = "counters";

// columns for ASSETS_TABLE
pub const ASSET_ID: &str = "asset_id";
pub const CURRENT_NAME: &str = "current_name";
pub const CURRENT_PATH: &str = "current_path";
pub const LOG_PATH_SQL: &str = "log_path";
pub const CREATED_AT: &str = "created_at";

// columns for COUNTER_TABLE
pub const ID: &str = "id";
pub const NEXT_ASSET_ID: &str = "next_asset_id";

// git tag error message
pub const NO_TAG_ERROR: &str = "No tag";

// file node data structure 
#[derive(Serialize)]
pub struct FileNode {
    pub name: String,
    pub is_dir: bool,
    pub children: Option<Vec<FileNode>>,
}

// file meta node data structure
#[derive(Serialize)]
pub struct FileMetadata {
    pub name: String,
    pub size_bytes: u64,
    pub modified_ddmmyyyy: String,
    pub created_ddmmyyyy: String,
    pub is_dir: bool,
    pub file_type: String,
    pub current_version: String,
    pub current_hash: String
}