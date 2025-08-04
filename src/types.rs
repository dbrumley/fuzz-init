use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct FuzzerOption {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub requires: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FuzzerConfig {
    pub supported: Vec<String>,
    pub default: String,
    pub options: Vec<FuzzerOption>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IntegrationOption {
    pub name: String,
    pub description: String,
    pub files: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IntegrationConfig {
    pub supported: Vec<String>,
    pub default: String,
    pub options: Vec<IntegrationOption>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TemplateMetadata {
    pub template: TemplateInfo,
    pub variables: HashMap<String, VariableConfig>,
    #[serde(default)]
    pub files: Vec<FileConfig>,
    #[serde(default)]
    pub directories: Vec<DirectoryConfig>,
    #[serde(default)]
    pub hooks: HookConfig,
    #[serde(default)]
    pub fuzzers: Option<FuzzerConfig>,
    #[serde(default)]
    pub integrations: Option<IntegrationConfig>,
    #[serde(default)]
    pub file_conventions: FileConventions,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TemplateInfo {
    pub name: String,
    pub description: String,
    pub version: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VariableConfig {
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileConfig {
    pub path: String,
    #[serde(default)]
    pub executable: bool,
    #[serde(default = "default_true")]
    pub template: bool,
    #[serde(default)]
    pub condition: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DirectoryConfig {
    pub path: String,
    #[serde(default)]
    pub create_empty: bool,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct HookConfig {
    pub post_generate: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct FileConventions {
    #[serde(default)]
    pub always_include: Vec<String>,
    #[serde(default)]
    pub full_mode_only: Vec<String>,
    #[serde(default)]
    pub template_extensions: Vec<String>,
    #[serde(default)]
    pub executable_extensions: Vec<String>,
    #[serde(default)]
    pub no_template_extensions: Vec<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone)]
pub enum TemplateSource {
    Local(String),
    GitHubFull(String),
}