fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv()?;
    let user = dotenv::var("GDTF_SHARE_USER")?;
    let password = dotenv::var("GDTF_SHARE_PASSWORD")?;

    smol::block_on(async {
        // Create a new GDTF Share client.
        let mut client = gdtf_share::Client::new();

        // Log in to the GDTF Share with a username and password. Make sure these are stored securely!
        client.login_async(&user, &password).await?;

        // Get all entries in the GDTF Share.
        let mut entries = client.get_list_async().await?;
        entries.sort_by(|a, b| a.fixture.cmp(&b.fixture));

        for entry in &entries {
            eprintln!("{}", entry.file_name());
        }

        // Download GDTF files by their Revision ID.
        let file_bytes = client.download_async(entries[0].rid).await?;
        let download_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("downloaded-from-example.gdtf");
        std::fs::write(download_path, file_bytes)?;

        // Use the `Library` container containing indexes to search through the entries.
        let library = gdtf_share::Library::new(entries);
        eprintln!(
            "Found {} latest entries with the fixture name \"JDC-1\"",
            library.query().fixture("JDC-1").latest_only(true).execute().count()
        );

        Ok(())
    })
}
