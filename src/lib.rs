use serde::Deserialize;

mod client;
mod error;

pub use client::*;
pub use error::*;

#[derive(Debug, Clone)]
pub struct Session {
    pub cookie: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub rid: i64,
    pub fixture: String,
    // Maybe enum
    pub manufacturer: String,
    // is this valid field?
    pub description: Option<String>,
    pub revision: String,
    pub creation_date: i64,
    pub last_modified: i64,
    pub uploader: String,
    pub rating: String,
    pub version: String,
    pub creator: String,
    pub uuid: Option<String>,
    pub filesize: i64,
    pub modes: Vec<GdtfMode>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GdtfMode {
    pub name: String,
    pub dmxfootprint: i32,
}
