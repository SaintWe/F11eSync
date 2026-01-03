#![allow(non_snake_case)]
// NOTE: The iOS Scripting protocol uses camelCase JSON fields (e.g. `fileId`),
// keep struct field names aligned to avoid breaking the on-wire format.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ClientConfig {
    pub enableFileSizeLimit: Option<bool>,
    pub maxFileSize: Option<u64>,
    pub pathRegex: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConnectionRejected {
    pub message: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateFile {
    pub path: String,
    pub content: String,
    pub encoding: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateDir {
    pub path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeletePayload {
    pub action: String,
    pub path: String,
    pub content: Option<String>,
    pub isDir: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChunkStart {
    pub path: String,
    pub fileId: String,
    pub totalChunks: u32,
    pub totalSize: Option<u64>,
    pub isDir: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChunkData {
    pub fileId: String,
    pub chunkIndex: u32,
    pub content: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChunkComplete {
    pub fileId: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChunkAck {
    pub fileId: String,
    pub chunkIndex: u32,
    pub success: Option<bool>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerLog {
    pub action: String,
    pub path: String,
    pub status: String,
    pub message: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SyncControl {
    pub action: String,
    pub path: String,
    pub content: Option<String>,
    pub isDir: bool,
}

#[derive(Debug, Clone)]
pub struct ChunkReceiveState {
    pub abs_path: std::path::PathBuf,
    pub rel_path: String,
    pub received_chunks: u32,
    pub total_chunks: u32,
    pub reject_reason: Option<String>,
}
