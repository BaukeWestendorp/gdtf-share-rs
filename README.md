# gdtf-share - A rust library for interacting with the GDTF Share API

This crate allows for interaction with the [GDTF Share API](https://www.gdtf.eu/gdtf/share_api/share-api/) to fetch and search fixture entries, and download GDTF zip archives.

## Example

```rust
// Create a new GDTF Share client with your credentials.
// Make sure these are stored securely, and not hardcoded like this!
let mut client = gdtf_share::Client::new("Username", "P455w0rd");

// Optionally pre-authenticate synchronously to prevent lazy login on subsequent calls.
client.login()?;

// Get all entries in the GDTF Share (uses filesystem caching if `cache` feature is enabled).
let entries = client.get_list()?;

for entry in &entries {
    eprintln!("{}", entry.file_name());
}

// Download GDTF files by their Revision ID (cached locally if `cache` feature is enabled).
let file_bytes = client.download(entries[0].rid)?;
std::fs::write(entries[0].file_name(), file_bytes)?;

// Use the `Library` container containing indexes to search through the entries.
let library = gdtf_share::Library::new(entries);
eprintln!(
    "Found {} latest entries with the fixture name \"JDC-1\"",
    library.query().fixture("JDC-1").latest_only(true).execute().count()
);
```

## Caching

When the `cache` feature is enabled, `Client` automatically caches API responses and downloaded files locally.
Calls to `get_list()` and `download()` will read directly from disk if a valid cached item exists.
Cache behavior can be customized or disabled using builder methods on `Client`:

```rust
use std::time::Duration;

// Customize TTL while keeping the default system cache path
let client = gdtf_share::Client::new("user", "pass")
    .with_cache_ttl(Duration::from_secs(12 * 3600));

// Set a custom cache directory override and TTL.
// The override is checked first; if the item isn't found there,
// the client falls back to the system default cache directory.
let client = gdtf_share::Client::new("user", "pass")
    .with_cache("/path/to/cache/", Duration::from_secs(3600));

// Set a custom cache directory override while keeping the default TTL.
let client = gdtf_share::Client::new("user", "pass")
    .with_cache_path("/path/to/cache/");

// Disable caching
let client = gdtf_share::Client::new("user", "pass")
    .without_cache();
```

## Cargo Features

- `client`: Provides the HTTP API client for managing a GDTF Share session.
- `async`: Enables asynchronous API methods for logging in, fetching the fixture list, and downloading files.
- `cache`: Enables automatic local filesystem caching of the fixture entry list and downloaded GDTF files.
- `log`: Logs debug info for the `log` crate.

## Contributing

Feel free to open an issue to discuss about missing features or found bugs, it's OSS after all!

> **NOTE** > I will not accept any PRs that includes an AI agent as a contributer.
> I strongly believe in keeping the domain-knowledge of the library in human hands.
> That being said, a PR that _suggests_ a code change (without an actual agent contribution) is fine as long as I don't feel the obligation to include said code.
> (i.e. I want the ability to review, potentially personally modify and accept a PR without an AI agent being part of this repository.)
