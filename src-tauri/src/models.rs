use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Registry,
    Folder,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    CurrentUser,
    AllUsers,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EntryType {
    Run,
    RunOnce,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupEntry {
    pub id: String,
    pub name: String,
    pub command: String,
    pub exe_path: String,
    pub args: String,
    pub source: Source,
    pub source_id: String,
    pub scope: Scope,
    pub enabled: bool,
    pub file_exists: bool,
    pub entry_type: EntryType,
}
