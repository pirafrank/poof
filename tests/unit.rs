//! Unit tests for standalone commands
//! This file serves as the main entry point for all unit tests

#[path = "unit/github_client.rs"]
mod github_client;

#[path = "unit/files/utils/find_similar_repos.rs"]
mod find_similar_repos;

#[path = "unit/files/utils/get_file_extension.rs"]
mod utils_get_file_extension;

#[path = "unit/files/utils/get_file_name.rs"]
mod utils_get_file_name;

#[path = "unit/files/utils/get_stem_name_trimmed_at_first_separator.rs"]
mod utils_get_stem_name_trimmed_at_first_separator;

#[path = "unit/files/utils/strip_supported_extensions.rs"]
mod utils_strip_supported_extensions;

#[path = "unit/files/archives/validate_magic_bytes.rs"]
mod archives_validate_magic_bytes;

#[path = "unit/files/archives/common.rs"]
mod archives_common;

#[path = "unit/files/archives/extract_to_dir.rs"]
mod archives_extract_to_dir;
