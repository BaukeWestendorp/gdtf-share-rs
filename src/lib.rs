//! Library for interacting with the GDTF Share API.

#[cfg(feature = "client")]
mod client;
mod error;

#[cfg(feature = "client")]
pub use client::*;
pub use error::*;

/// Represents an active authenticated session.
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Session {
    /// The session cookie string returned by the server.
    pub cookie: String,
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
    pub uuid: Option<String>,
    /// Size of the GDTF archive in bytes.
    pub filesize: u64,
    /// Modes supported by this fixture.
    pub modes: Vec<GdtfMode>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Uploader {
    #[serde(rename = "User")]
    User,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GdtfVersion {
    pub major: u8,
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
