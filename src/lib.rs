//! Library for interacting with the GDTF Share API.

#![warn(missing_docs)]

use std::collections::{BTreeMap, HashMap};

#[cfg(feature = "client")]
mod client;
mod error;

#[cfg(feature = "client")]
pub use client::*;
pub use error::*;

pub use ::uuid::Uuid;

/// Represents an active authenticated session.
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Session {
    /// The session cookie string returned by the server.
    pub cookie: String,
}

/// Represents a catalog of GDTF entries, with helpers for fast, indexed lookups.
pub struct Catalog {
    entries: Vec<Entry>,
    by_rid: HashMap<u32, usize>,
    by_uuid: HashMap<uuid::Uuid, usize>,
    by_fixture: HashMap<String, Vec<usize>>,
    by_manufacturer: HashMap<String, Vec<usize>>,
    by_version: HashMap<GdtfVersion, Vec<usize>>,
    by_rating: BTreeMap<Rating, Vec<usize>>,
    by_uploader: [Vec<usize>; 2],
}

impl Catalog {
    /// Constructs a new catalog and indexes all entries.
    pub fn new(entries: Vec<Entry>) -> Self {
        let mut by_rid = HashMap::with_capacity(entries.len());
        let mut by_uuid = HashMap::<uuid::Uuid, usize>::new();
        let mut by_fixture: HashMap<String, Vec<usize>> = HashMap::new();
        let mut by_manufacturer: HashMap<String, Vec<usize>> = HashMap::new();
        let mut by_version: HashMap<GdtfVersion, Vec<usize>> = HashMap::new();
        let mut by_rating: BTreeMap<Rating, Vec<usize>> = BTreeMap::new();
        let mut by_uploader: [Vec<usize>; 2] = [Vec::new(), Vec::new()];

        for (idx, entry) in entries.iter().enumerate() {
            by_rid.insert(entry.rid, idx);
            if let Some(uuid) = &entry.uuid {
                by_uuid.insert(uuid.clone(), idx);
            }
            by_fixture.entry(entry.fixture.to_lowercase()).or_default().push(idx);
            by_manufacturer.entry(entry.manufacturer.to_lowercase()).or_default().push(idx);
            by_version.entry(entry.version).or_default().push(idx);
            by_rating.entry(entry.rating).or_default().push(idx);

            match entry.uploader {
                Uploader::User => by_uploader[0].push(idx),
                Uploader::Manufacturer => by_uploader[1].push(idx),
            }
        }

        Self {
            entries,
            by_rid,
            by_uuid,
            by_fixture,
            by_manufacturer,
            by_version,
            by_rating,
            by_uploader,
        }
    }

    /// Gets an entry reference by its index.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&Entry> {
        self.entries.get(index)
    }

    /// Returns a slice of all entries.
    #[inline]
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Look up an entry by its unique Revision ID (RID).
    pub fn get_by_rid(&self, rid: u32) -> Option<&Entry> {
        self.by_rid.get(&rid).map(|&i| &self.entries[i])
    }

    /// Look up an entry by its UUID.
    pub fn get_by_uuid(&self, uuid: &uuid::Uuid) -> Option<&Entry> {
        self.by_uuid.get(uuid).map(|&i| &self.entries[i])
    }

    /// Find entries matching a fixture model name (case-insensitive).
    pub fn by_fixture<'a>(&'a self, fixture: &str) -> impl Iterator<Item = &'a Entry> {
        self.resolve_indices(self.by_fixture.get(&fixture.to_lowercase()))
    }

    /// Find entries matching a manufacturer name (case-insensitive).
    pub fn by_manufacturer<'a>(&'a self, manufacturer: &str) -> impl Iterator<Item = &'a Entry> {
        self.resolve_indices(self.by_manufacturer.get(&manufacturer.to_lowercase()))
    }

    /// Find entries by GDTF version.
    pub fn by_version<'a>(&'a self, version: GdtfVersion) -> impl Iterator<Item = &'a Entry> {
        self.resolve_indices(self.by_version.get(&version))
    }

    /// Find entries by uploader type.
    pub fn by_uploader<'a>(&'a self, uploader: Uploader) -> impl Iterator<Item = &'a Entry> {
        let indices = match uploader {
            Uploader::User => &self.by_uploader[0],
            Uploader::Manufacturer => &self.by_uploader[1],
        };
        indices.iter().map(|&i| &self.entries[i])
    }

    /// Find entries with a rating equal to or greater than `min_rating`.
    pub fn by_min_rating<'a>(&'a self, min_rating: Rating) -> impl Iterator<Item = &'a Entry> {
        self.by_rating
            .range(min_rating..)
            .flat_map(|(_, indices)| indices.iter())
            .map(|&i| &self.entries[i])
    }

    /// Text search matching `query` across fixture model, manufacturer, and creator fields.
    pub fn search<'a>(&'a self, query: &str) -> impl Iterator<Item = &'a Entry> {
        let q = query.to_lowercase();
        self.entries.iter().filter(move |entry| {
            entry.fixture.to_lowercase().contains(&q)
                || entry.manufacturer.to_lowercase().contains(&q)
                || entry.creator.to_lowercase().contains(&q)
        })
    }

    #[inline]
    fn resolve_indices<'a>(
        &'a self,
        indices: Option<&'a Vec<usize>>,
    ) -> impl Iterator<Item = &'a Entry> {
        indices.into_iter().flatten().map(move |&i| &self.entries[i])
    }
}

// NOTE: `description` is missing, since it's always null.
/// A GDTF entry from the GDTF Share library.
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    /// Revision ID (RID).
    pub rid: u32,
    /// Name of the fixture model.
    pub fixture: String,
    /// Name of the manufacturer.
    pub manufacturer: String,
    /// Revision name.
    pub revision: String,
    /// Creation timestamp (Unix time).
    pub creation_date: u64,
    /// Last modified timestamp (Unix time).
    pub last_modified: u64,
    /// Username of the uploader.
    pub uploader: Uploader,
    /// Rating.
    pub rating: Rating,
    /// GDTF Version.
    pub version: GdtfVersion,
    /// Creator of the GDTF file.
    pub creator: String,
    /// The GDTF's UUID.
    pub uuid: Option<uuid::Uuid>,
    /// Size of the GDTF archive in bytes.
    pub filesize: u64,
    /// Modes supported by this fixture.
    pub modes: Vec<GdtfMode>,
}

impl Entry {
    /// Generates a sanitized filename for the GDTF entry.
    ///
    /// Has the format: `<manufacturer>@<fixture>@<revision>.gdtf`, with invalid characters replaced by underscores.
    pub fn file_name(&self) -> String {
        let sanitize = |s: &str| {
            let cleaned: String = s
                .chars()
                .map(|c| match c {
                    ' ' | '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0' => '_',
                    c if c.is_control() => '_',
                    c => c,
                })
                .collect();

            let trimmed = cleaned.trim_end_matches(['.', '_']);
            if trimmed.is_empty() { "unnamed".to_string() } else { trimmed.to_string() }
        };

        format!(
            "{}@{}@{}.gdtf",
            sanitize(&self.manufacturer),
            sanitize(&self.fixture),
            sanitize(&self.revision)
        )
    }
}

/// A DMX mode configuration within a GDTF entry.
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GdtfMode {
    /// Name of the DMX mode.
    pub name: String,
    /// DMX channel footprint.
    #[serde(rename = "dmxfootprint")]
    pub dmx_footprint: u32,
}

/// The uploader type for a GDTF entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Uploader {
    /// The GDTF was uploaded by a user.
    #[serde(rename = "User")]
    User,
    /// The GDTF was uploaded by a manufacturer.
    #[serde(rename = "Manuf.")]
    Manufacturer,
}

impl std::fmt::Display for Uploader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User => write!(f, "User"),
            Self::Manufacturer => write!(f, "Manufacturer"),
        }
    }
}

/// A GDTF file version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GdtfVersion {
    /// Major version number.
    pub major: u8,
    /// Minor version number.
    pub minor: u8,
}

impl std::fmt::Display for GdtfVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl std::str::FromStr for GdtfVersion {
    type Err = Error;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 2 {
            return Err(Error::InvalidRating);
        }
        let major = parts[0].parse::<u8>().map_err(|_| Error::InvalidRating)?;
        let minor = parts[1].parse::<u8>().map_err(|_| Error::InvalidRating)?;
        Ok(GdtfVersion { major, minor })
    }
}

impl serde::Serialize for GdtfVersion {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for GdtfVersion {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// A rating value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rating {
    /// No rating is available ("N/A").
    NotAvailable,
    /// Numerical rating value from 0 to 50.
    ///
    /// The rating is represented as an integer instead of a float, to make it `Ord` and allow for
    /// proper sorting.
    Value(u8),
}

impl Rating {
    /// Converts the rating to a float value, or returns `None` if the rating is not available.
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Rating::NotAvailable => None,
            Rating::Value(v) => Some(*v as f64),
        }
    }
}

impl std::fmt::Display for Rating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rating::NotAvailable => write!(f, "N/A"),
            Rating::Value(v) => write!(f, "{}", *v as f64 / 10.0),
        }
    }
}

impl std::str::FromStr for Rating {
    type Err = Error;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        if s == "N/A" {
            Ok(Rating::NotAvailable)
        } else {
            let f = s.parse::<f64>().map_err(|_| Error::InvalidRating)?;
            Ok(Rating::Value((f * 10.0).round() as u8))
        }
    }
}

impl<'de> serde::Deserialize<'de> for Rating {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

impl serde::Serialize for Rating {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
