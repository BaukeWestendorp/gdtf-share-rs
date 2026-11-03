# gdtf-share - A rust library for interacting with the GDTF Share API

This crate allows for interaction with the [GDTF Share API](https://www.gdtf.eu/gdtf/share_api/share-api/) to fetch and search fixture entries, and download GDTF zip archives.

## Example

```rust
// Create a new GDTF Share client.
let mut client = gdtf_share::Client::new();

// Log in to the GDTF Share with a username and password. Make sure these are stored securely, and not inlined like this!
client.login("Username", "P455w0rd")?;

// Get all entries in the GDTF Share (uses filesystem caching if `cache` feature is enabled).
let entries = client.get_list()?;

for entry in &entries {
    eprintln!("{}", entry.file_name());
}

// Download GDTF files by their Revision ID.
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

When the cache feature is enabled, Client::new() automatically caches the get_list API response locally in the system's default cache directory (dirs::cache_dir()). Calls to get_list() or get_list_async() will read directly from the disk cache if the cached response is valid.
Cache behavior can be customized or disabled using builder methods on Client:

```rust
use std::time::Duration;

// Customize TTL while keeping default system cache path
let client = gdtf_share::Client::new()
    .with_cache_ttl(Duration::from_secs(12 * 3600));

// Specify a custom cache path and TTL
let client = gdtf_share::Client::new()
    .with_cache("/path/to/cache.json", Duration::from_secs(3600));

// Disable caching explicitly
let client = gdtf_share::Client::new()
    .without_cache();
```

## Cargo Features

- `client`: Provides the HTTP API client for managing a GDTF Share session.
- `async`: Enables asynchronous API methods for logging in, fetching the fixture list, and downloading files.
- `cache`: Enables automatic local filesystem caching of the fixture entry list using the dirs crate.

## Contributing

Feel free to open an issue to discuss about missing features or found bugs, it's OSS after all!
