//! An indexed collection of [`Entry`] values.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::ops::{Bound, RangeBounds};

use uuid::Uuid;

use super::*;

/// A unique identifier for a GDTF revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct RevisionId(u32);

impl RevisionId {
    /// Creates a new `RevisionId` from a `u32`.
    pub fn new(v: u32) -> Self {
        RevisionId(v)
    }

    /// Returns the `RevisionId` as a `u32`.
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl From<u32> for RevisionId {
    fn from(value: u32) -> Self {
        RevisionId(value)
    }
}

impl From<RevisionId> for u32 {
    fn from(value: RevisionId) -> Self {
        value.0
    }
}

impl std::fmt::Display for RevisionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// An indexed collection of GDTF [`Entry`] values.
#[derive(Debug, Clone, Default)]
pub struct Library {
    entries: BTreeMap<RevisionId, Entry>,
    latest_revisions: BTreeMap<Uuid, RevisionId>,

    by_uuid: BTreeMap<Uuid, BTreeSet<RevisionId>>,
    by_manufacturer: BTreeMap<String, BTreeSet<RevisionId>>,
    by_fixture: BTreeMap<String, BTreeSet<RevisionId>>,
    by_revision: BTreeMap<String, BTreeSet<RevisionId>>,
    by_creator: BTreeMap<String, BTreeSet<RevisionId>>,
    by_uploader: BTreeMap<Uploader, BTreeSet<RevisionId>>,
    by_rating: BTreeMap<Rating, BTreeSet<RevisionId>>,
    by_version: BTreeMap<GdtfVersion, BTreeSet<RevisionId>>,
    by_creation_date: BTreeMap<u64, BTreeSet<RevisionId>>,
    by_last_modified: BTreeMap<u64, BTreeSet<RevisionId>>,
    by_filesize: BTreeMap<u64, BTreeSet<RevisionId>>,
}

impl Library {
    /// Creates a library from a list of entries.
    pub fn new(entries: impl IntoIterator<Item = Entry>) -> Self {
        let mut library = Self::default();
        for entry in entries {
            library.insert(entry);
        }
        library
    }

    /// Number of entries in the library.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the library has no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterates over all entries, ordered by `rid`.
    pub fn iter_all_revisions(&self) -> impl Iterator<Item = &Entry> {
        self.entries.values()
    }

    /// Iterates over the latest revision of each fixture (by UUID).
    pub fn iter_latest_revisions(&self) -> impl Iterator<Item = &Entry> {
        self.latest_revisions.values().filter_map(|rid| self.entries.get(rid))
    }

    /// Looks up a single entry by its `rid`.
    pub fn get(&self, rid: RevisionId) -> Option<&Entry> {
        self.entries.get(&rid)
    }

    /// Returns a [`Query`] for this [`Library`].
    pub fn query(&self) -> Query<'_> {
        Query::new(self)
    }

    /// Searches for entries whose manufacturer and fixture name contain all words in the query string (case-insensitive).
    pub fn search_latest(&self, query: &str) -> impl Iterator<Item = &Entry> {
        let query_words =
            query.to_lowercase().split_whitespace().map(ToString::to_string).collect::<Vec<_>>();
        self.iter_latest_revisions().filter(move |e| {
            let haystack = format!("{} {}", e.manufacturer, e.fixture).to_lowercase();
            query_words.iter().all(|word| haystack.contains(word))
        })
    }

    fn insert(&mut self, entry: Entry) {
        let rid = entry.rid;

        index_push(&mut self.by_manufacturer, entry.manufacturer.clone(), rid);
        index_push(&mut self.by_fixture, entry.fixture.clone(), rid);
        index_push(&mut self.by_revision, entry.revision.clone(), rid);
        index_push(&mut self.by_creator, entry.creator.clone(), rid);
        index_push(&mut self.by_uploader, entry.uploader.clone(), rid);
        index_push(&mut self.by_rating, entry.rating, rid);
        index_push(&mut self.by_version, entry.version, rid);
        index_push(&mut self.by_creation_date, entry.creation_date, rid);
        index_push(&mut self.by_last_modified, entry.last_modified, rid);
        index_push(&mut self.by_filesize, entry.filesize, rid);

        if let Some(uuid) = entry.uuid {
            index_push(&mut self.by_uuid, uuid, rid);
        }

        self.entries.insert(rid, entry);

        if let Some(uuid) = self.entries[&rid].uuid {
            let is_latest = match self.latest_revisions.get(&uuid) {
                Some(&latest_rid) => rid > latest_rid,
                None => true,
            };

            if is_latest {
                self.latest_revisions.insert(uuid, rid);
            }
        }
    }
}

/// A builder for querying entries in a [`Library`].
#[derive(Clone)]
pub struct Query<'a> {
    library: &'a Library,
    uuid: Option<Uuid>,
    manufacturer: Option<&'a str>,
    fixture: Option<&'a str>,
    revision: Option<&'a str>,
    creator: Option<&'a str>,
    uploader: Option<&'a Uploader>,
    rating: Option<Rating>,
    rating_range: Option<(Bound<Rating>, Bound<Rating>)>,
    version: Option<GdtfVersion>,
    version_range: Option<(Bound<GdtfVersion>, Bound<GdtfVersion>)>,
    creation_date: Option<u64>,
    creation_date_range: Option<(Bound<u64>, Bound<u64>)>,
    last_modified: Option<u64>,
    last_modified_range: Option<(Bound<u64>, Bound<u64>)>,
    filesize: Option<u64>,
    filesize_range: Option<(Bound<u64>, Bound<u64>)>,
    latest_only: bool,
}

impl<'a> Query<'a> {
    /// Creates a new query for the given library.
    pub fn new(library: &'a Library) -> Self {
        Self {
            library,
            uuid: None,
            manufacturer: None,
            fixture: None,
            revision: None,
            creator: None,
            uploader: None,
            rating: None,
            rating_range: None,
            version: None,
            version_range: None,
            creation_date: None,
            creation_date_range: None,
            last_modified: None,
            last_modified_range: None,
            filesize: None,
            filesize_range: None,
            latest_only: false,
        }
    }

    /// Filters the query to only include entries with the given UUID.
    pub fn uuid(mut self, uuid: Uuid) -> Self {
        self.uuid = Some(uuid);
        self
    }

    /// Filters the query to only include entries with the given manufacturer.
    pub fn manufacturer(mut self, manufacturer: &'a str) -> Self {
        self.manufacturer = Some(manufacturer);
        self
    }

    /// Filters the query to only include entries with the given fixture name.
    pub fn fixture(mut self, fixture: &'a str) -> Self {
        self.fixture = Some(fixture);
        self
    }

    /// Filters the query to only include entries with the given revision name.
    pub fn revision(mut self, revision: &'a str) -> Self {
        self.revision = Some(revision);
        self
    }

    /// Filters the query to only include entries with the given creator.
    pub fn creator(mut self, creator: &'a str) -> Self {
        self.creator = Some(creator);
        self
    }

    /// Filters the query to only include entries with the given uploader.
    pub fn uploader(mut self, uploader: &'a Uploader) -> Self {
        self.uploader = Some(uploader);
        self
    }

    /// Filters the query to only include entries with the given rating.
    pub fn rating(mut self, rating: Rating) -> Self {
        self.rating = Some(rating);
        self
    }

    /// Filters the query to only include entries whose rating falls within the given range.
    pub fn rating_range(mut self, range: impl RangeBounds<Rating>) -> Self {
        self.rating_range = Some((range.start_bound().cloned(), range.end_bound().cloned()));
        self
    }

    /// Filters the query to only include entries with the given GDTF version.
    pub fn version(mut self, version: GdtfVersion) -> Self {
        self.version = Some(version);
        self
    }

    /// Filters the query to only include entries whose GDTF version falls within the given range.
    pub fn version_range(mut self, range: impl RangeBounds<GdtfVersion>) -> Self {
        self.version_range = Some((range.start_bound().cloned(), range.end_bound().cloned()));
        self
    }

    /// Filters the query to only include entries with the given creation date (Unix timestamp).
    pub fn creation_date(mut self, creation_date: u64) -> Self {
        self.creation_date = Some(creation_date);
        self
    }

    /// Filters the query to only include entries whose creation date falls within the given range (Unix timestamps).
    pub fn creation_date_range(mut self, range: impl RangeBounds<u64>) -> Self {
        self.creation_date_range = Some((range.start_bound().cloned(), range.end_bound().cloned()));
        self
    }

    /// Filters the query to only include entries with the given last modified date (Unix timestamp).
    pub fn last_modified(mut self, last_modified: u64) -> Self {
        self.last_modified = Some(last_modified);
        self
    }

    /// Filters the query to only include entries whose last modified date falls within the given range (Unix timestamps).
    pub fn last_modified_range(mut self, range: impl RangeBounds<u64>) -> Self {
        self.last_modified_range = Some((range.start_bound().cloned(), range.end_bound().cloned()));
        self
    }

    /// Filters the query to only include entries with the given filesize (in bytes).
    pub fn filesize(mut self, filesize: u64) -> Self {
        self.filesize = Some(filesize);
        self
    }

    /// Filters the query to only include entries whose filesize falls within the given range (in bytes).
    pub fn filesize_range(mut self, range: impl RangeBounds<u64>) -> Self {
        self.filesize_range = Some((range.start_bound().cloned(), range.end_bound().cloned()));
        self
    }

    /// Filters the query to only include the latest revision of each fixture (by UUID).
    pub fn latest_only(mut self, latest_only: bool) -> Self {
        self.latest_only = latest_only;
        self
    }

    /// Executes the query and returns an iterator over the matching entries.
    pub fn execute(self) -> Box<dyn Iterator<Item = &'a Entry> + 'a> {
        let mut matching_sets: Vec<Cow<'a, BTreeSet<RevisionId>>> = Vec::new();

        if let Some(uuid) = self.uuid {
            match self.library.by_uuid.get(&uuid) {
                Some(set) => matching_sets.push(Cow::Borrowed(set)),
                None => return Box::new(std::iter::empty()),
            }
        }
        if let Some(manufacturer) = self.manufacturer {
            match self.library.by_manufacturer.get(manufacturer) {
                Some(set) => matching_sets.push(Cow::Borrowed(set)),
                None => return Box::new(std::iter::empty()),
            }
        }
        if let Some(fixture) = self.fixture {
            match self.library.by_fixture.get(fixture) {
                Some(set) => matching_sets.push(Cow::Borrowed(set)),
                None => return Box::new(std::iter::empty()),
            }
        }
        if let Some(revision) = self.revision {
            match self.library.by_revision.get(revision) {
                Some(set) => matching_sets.push(Cow::Borrowed(set)),
                None => return Box::new(std::iter::empty()),
            }
        }
        if let Some(creator) = self.creator {
            match self.library.by_creator.get(creator) {
                Some(set) => matching_sets.push(Cow::Borrowed(set)),
                None => return Box::new(std::iter::empty()),
            }
        }
        if let Some(uploader) = self.uploader {
            match self.library.by_uploader.get(uploader) {
                Some(set) => matching_sets.push(Cow::Borrowed(set)),
                None => return Box::new(std::iter::empty()),
            }
        }
        if let Some(rating) = self.rating {
            match self.library.by_rating.get(&rating) {
                Some(set) => matching_sets.push(Cow::Borrowed(set)),
                None => return Box::new(std::iter::empty()),
            }
        }
        if let Some((start, end)) = self.rating_range {
            let mut merged = BTreeSet::new();
            for (_key, set) in self.library.by_rating.range((start, end)) {
                merged.extend(set);
            }
            if merged.is_empty() {
                return Box::new(std::iter::empty());
            }
            matching_sets.push(Cow::Owned(merged));
        }
        if let Some(version) = self.version {
            match self.library.by_version.get(&version) {
                Some(set) => matching_sets.push(Cow::Borrowed(set)),
                None => return Box::new(std::iter::empty()),
            }
        }
        if let Some((start, end)) = self.version_range {
            let mut merged = BTreeSet::new();
            for (_key, set) in self.library.by_version.range((start, end)) {
                merged.extend(set);
            }
            if merged.is_empty() {
                return Box::new(std::iter::empty());
            }
            matching_sets.push(Cow::Owned(merged));
        }
        if let Some(creation_date) = self.creation_date {
            match self.library.by_creation_date.get(&creation_date) {
                Some(set) => matching_sets.push(Cow::Borrowed(set)),
                None => return Box::new(std::iter::empty()),
            }
        }
        if let Some((start, end)) = self.creation_date_range {
            let mut merged = BTreeSet::new();
            for (_key, set) in self.library.by_creation_date.range((start, end)) {
                merged.extend(set);
            }
            if merged.is_empty() {
                return Box::new(std::iter::empty());
            }
            matching_sets.push(Cow::Owned(merged));
        }
        if let Some(last_modified) = self.last_modified {
            match self.library.by_last_modified.get(&last_modified) {
                Some(set) => matching_sets.push(Cow::Borrowed(set)),
                None => return Box::new(std::iter::empty()),
            }
        }
        if let Some((start, end)) = self.last_modified_range {
            let mut merged = BTreeSet::new();
            for (_key, set) in self.library.by_last_modified.range((start, end)) {
                merged.extend(set);
            }
            if merged.is_empty() {
                return Box::new(std::iter::empty());
            }
            matching_sets.push(Cow::Owned(merged));
        }
        if let Some(filesize) = self.filesize {
            match self.library.by_filesize.get(&filesize) {
                Some(set) => matching_sets.push(Cow::Borrowed(set)),
                None => return Box::new(std::iter::empty()),
            }
        }
        if let Some((start, end)) = self.filesize_range {
            let mut merged = BTreeSet::new();
            for (_key, set) in self.library.by_filesize.range((start, end)) {
                merged.extend(set);
            }
            if merged.is_empty() {
                return Box::new(std::iter::empty());
            }
            matching_sets.push(Cow::Owned(merged));
        }

        let latest_only = self.latest_only;
        let lib = self.library;

        if matching_sets.is_empty() {
            if latest_only {
                Box::new(lib.iter_latest_revisions())
            } else {
                Box::new(lib.iter_all_revisions())
            }
        } else {
            matching_sets.sort_by_key(|s| s.len());

            let mut candidates: BTreeSet<RevisionId> = matching_sets.remove(0).into_owned();
            for set in &matching_sets {
                candidates.retain(|rid| set.contains(rid));
            }

            Box::new(candidates.into_iter().filter_map(move |rid| lib.get(rid)).filter(
                move |entry| {
                    if !latest_only {
                        return true;
                    }
                    match entry.uuid {
                        Some(uuid) => lib.latest_revisions.get(&uuid) == Some(&entry.rid),
                        None => true,
                    }
                },
            ))
        }
    }
}

impl<'a> IntoIterator for Query<'a> {
    type Item = &'a Entry;
    type IntoIter = Box<dyn Iterator<Item = &'a Entry> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.execute()
    }
}

impl From<Vec<Entry>> for Library {
    fn from(entries: Vec<Entry>) -> Self {
        Self::new(entries)
    }
}

impl FromIterator<Entry> for Library {
    fn from_iter<T: IntoIterator<Item = Entry>>(iter: T) -> Self {
        Self::new(iter)
    }
}

fn index_push<K: Ord>(index: &mut BTreeMap<K, BTreeSet<RevisionId>>, key: K, rid: RevisionId) {
    index.entry(key).or_default().insert(rid);
}
