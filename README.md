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

When the `cache` feature is enabled, `Client` automatically caches API responses and downloaded files locally
Calls to `get_list()` and `download()` will read directly from disk if a valid cached item exists.
Cache behavior can be customized or disabled using builder methods on Client:

```rust
use std::time::Duration;

// Customize TTL while keeping default system cache path
let client = gdtf_share::Client::new()
    .with_cache_ttl(Duration::from_secs(12 * 3600));

// Specify a custom cache path and TTL
let client = gdtf_share::Client::new()
    .with_cache("/path/to/cache/", Duration::from_secs(3600));

// Disable caching
let client = gdtf_share::Client::new()
    .without_cache();
```

## Cargo Features

- `client`: Provides the HTTP API client for managing a GDTF Share session.
- `async`: Enables asynchronous API methods for logging in, fetching the fixture list, and downloading files.
- `cache`: Enables automatic local filesystem caching of the fixture entry list and downloaded GDTF files.
- `log`: Logs debug info for the `log` crate.

## Contributing

Feel free to open an issue to discuss about missing features or found bugs, it's OSS after all!
