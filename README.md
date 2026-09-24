# gdtf-share - A rust library for interacting with the GDTF Share API

This crate allows for interaction with the [GDTF Share API](https://www.gdtf.eu/gdtf/share_api/share-api/) to fetch fixture entry lists, and download GDTF zip archives.

## Example

```rs
let mut client = gdtf_share::Client::new();
client.login("Username", "P455w0rd")?;

let mut list = client.get_list()?;

for entry in &list {
    eprintln!("{}", entry.file_name());
}

let file_bytes = client.download(list[0].rid)?;
std::fs::write("downloaded.gdtf", file_bytes).unwrap();
```

## Cargo Features

- `client`: Provides the HTTP API client for managing a GDTF Share session.
- `async`: Enables asynchronous API methods for logging in, fetching the fixture list, and downloading files.

## Contributing

Feel free to open an issue to discuss about missing features or found bugs, it's OSS after all!
