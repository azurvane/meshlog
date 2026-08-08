pub mod core;
pub mod helper;

pub use core::get_log_files;
pub use core::get_file_flat;
pub use core::get_file_tree;
pub use core::get_file_metadata;
pub use core::get_directory_metadata;

pub use helper::get_filename_createdat;
pub use helper::get_log_path;
pub use helper::create_log_md;
pub use helper::append_log_md;
pub use helper::insert_log_entry;
