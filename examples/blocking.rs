fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv()?;
    let user = dotenv::var("GDTF_SHARE_USER")?;
    let password = dotenv::var("GDTF_SHARE_PASSWORD")?;

    let mut client = gdtf_share::Client::new();
    client.login(&user, &password)?;

    let mut list = client.get_list()?;
    list.sort_by(|a, b| a.fixture.cmp(&b.fixture));

    for entry in &list {
        eprintln!("{}", entry.fixture);
    }

    let file_bytes = client.download(list[0].rid).unwrap();
    let download_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("downloaded-from-example.gdtf");
    std::fs::write(download_path, file_bytes).unwrap();

    Ok(())
}
