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
pub const LINK_TABLE: &str = "link";

// columns for ASSETS_TABLE
pub const ASSET_ID: &str = "asset_id";
pub const CURRENT_NAME: &str = "current_name";
pub const CURRENT_PATH: &str = "current_path";
pub const LOG_PATH_SQL: &str = "log_path";
pub const CREATED_AT: &str = "created_at";

// columns for COUNTER_TABLE
pub const ID: &str = "id";
pub const NEXT_ASSET_ID: &str = "next_asset_id";

// columns for LINK_TABLE
pub const NEW_PATH: &str = "new_path";
pub const OLD_PATH: &str = "old_path";

// git tag error message
pub const NO_TAG_ERROR: &str = "No tag";
pub const NO_COMMIT_METADATA: &str = "No commit metadata found";

// constant values 
pub const MAX_RETRY_ATTEMPTS: u64 = 100;
pub const RETRY_DELAY: u64 = 150; // millisecnods

// file node data structure 
#[derive(Serialize)]
pub struct FileNode {
    pub name: String,
    pub is_dir: bool,
    pub children: Option<Vec<FileNode>>,
}

// file metadata node data structure
#[derive(Serialize)]
pub struct CommonMetadata {
    pub name: String,
    pub size_bytes: u64,
    pub modified_ddmmyyyy: String,
    pub created_ddmmyyyy: String,
    pub is_dir: bool,
    pub file_type: String,
}

// file metadata node data structure
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

// commit metadata node data structure
#[derive(Serialize)]
pub struct CommitMetadata {
    pub commit_hash: String,
    pub abbreviated_hash: String,
    pub author_name: String,
    pub author_date: String,
    pub subject: String,
    pub body: String,
}

// asset table values
#[derive(Serialize)]
pub struct AssetValues {
    pub asset_id: String,
    pub current_name: String,
    pub current_path: String,
    pub log_path: String,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct TableData {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Serialize)]
pub struct RenameCandidate {
    pub old_path: String,
    pub new_path: String,
    pub score: u8,
}