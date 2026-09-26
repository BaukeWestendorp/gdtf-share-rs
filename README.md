# gdtf-share - A rust library for interacting with the GDTF Share API

This crate allows for interaction with the [GDTF Share API](https://www.gdtf.eu/gdtf/share_api/share-api/) to fetch and search fixture entries, and download GDTF zip archives.

## Example

```rs
// Create a new GDTF Share client.
let mut client = gdtf_share::Client::new();

// Log in to the GDTF Share with a username and password. Make sure these are stored securely, and not inlined like this!
client.login("Username", "P455w0rd")?;

// Get all entries in the GDTF Share.
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

## Cargo Features

- `client`: Provides the HTTP API client for managing a GDTF Share session.
- `async`: Enables asynchronous API methods for logging in, fetching the fixture list, and downloading files.

## Contributing

Feel free to open an issue to discuss about missing features or found bugs, it's OSS after all!
