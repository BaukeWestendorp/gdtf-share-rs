fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv()?;
    let user = dotenv::var("GDTF_SHARE_USER")?;
    let password = dotenv::var("GDTF_SHARE_PASSWORD")?;

    let mut client = gdtf_share::Client::new();
    client.login(&user, &password)?;

    let resp = client.get_list()?;
    eprintln!("{:?}", resp);

    Ok(())
}
