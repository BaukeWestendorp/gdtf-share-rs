fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv()?;
    let user = dotenv::var("GDTF_SHARE_USER")?;
    let password = dotenv::var("GDTF_SHARE_PASSWORD")?;

    // Create a new GDTF Share client.
    let mut client = gdtf_share::Client::new();

    // Log in to the GDTF Share with a username and password. Make sure these are stored securely!
    client.login(&user, &password)?;

    // Get all entries in the GDTF Share.
    let mut entries = client.get_list()?;
    entries.sort_by(|a, b| a.fixture.cmp(&b.fixture));

    for entry in &entries {
        eprintln!("{}", entry.file_name());
    }

    // Download GDTF files by their Revision ID.
    let file_bytes = client.download(entries[0].rid)?;
    let download_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("downloaded-from-example.gdtf");
    std::fs::write(download_path, file_bytes)?;

    // Use the `Catalog` container containing indexes to performantly search through the entries.
    let catalog = gdtf_share::Catalog::new(entries);
    eprintln!("Found {} entries containing \"JDC-1\"", catalog.search("JDC-1").count());

    Ok(())
}
