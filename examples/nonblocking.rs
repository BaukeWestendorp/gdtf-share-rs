fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv()?;
    let user = dotenv::var("GDTF_SHARE_USER")?;
    let password = dotenv::var("GDTF_SHARE_PASSWORD")?;

    smol::block_on(async {
        let mut client = gdtf_share::Client::new();
        client.login_async(&user, &password).await?;

        let resp = client.get_list_async().await?;
        eprintln!("{:?}", resp);

        Ok(())
    })
}
