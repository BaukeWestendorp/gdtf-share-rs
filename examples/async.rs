fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv()?;
    let user = dotenv::var("GDTF_SHARE_USER")?;
    let password = dotenv::var("GDTF_SHARE_PASSWORD")?;

    smol::block_on(async {
        let mut client = gdtf_share::Client::new();
        client.login(&user, &password)?;

        let mut list = client.get_list()?;
        list.sort_by(|a, b| a.fixture.cmp(&b.fixture));

        for entry in list {
            eprintln!("{}", entry.fixture);
        }

        Ok(())
    })
}
