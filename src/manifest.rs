use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TemplateEntry {
    pub template_name: String,
    pub output_name: String,
    pub context: HashMap<String, String>, // TODO
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StaticWebsiteManifest {
    pub asset_manifest_path: PathBuf,
    pub asset_cache_manifest_path: PathBuf,
    pub asset_directory_path: PathBuf,
    pub internal_directory_path: PathBuf,
    pub static_directory_path: PathBuf,
    pub static_base_url: String,
    pub tera_input_directory: String,
    pub html_output_directory_path: PathBuf,
    pub tera_templates: Vec<TemplateEntry>,
    pub context: HashMap<String, String>, // TODO
    pub copy_output_directory_path: PathBuf,
    pub copy_input_directory_path: PathBuf,
    pub copy_paths: Vec<PathBuf>,
}
