//! HTTP client for interacting with the GDTF Share API.

use secrecy::ExposeSecret;

use super::*;

/// Represents an active authenticated session.
#[derive(Clone)]
pub struct Session {
    /// The session cookie string returned by the server.
    cookie: secrecy::SecretString,
}

/// API client for managing a GDTF Share session.
#[derive(Clone)]
pub struct Client {
    session: Option<Session>,

    #[cfg(feature = "cache")]
    cache_path: Option<std::path::PathBuf>,
    #[cfg(feature = "cache")]
    cache_ttl: Option<std::time::Duration>,

    username: secrecy::SecretString,
    password: secrecy::SecretString,
}

impl Client {
    /// Creates a new unauthenticated GDTF Share API client.
    pub fn new(username: &str, password: &str) -> Self {
        Self {
            session: None,
            #[cfg(feature = "cache")]
            cache_path: default_cache_path(),
            #[cfg(feature = "cache")]
            cache_ttl: Some(std::time::Duration::from_secs(24 * 60 * 60)),

            username: secrecy::SecretString::from(username),
            password: secrecy::SecretString::from(password),
        }
    }

    /// Sets a custom cache file path and TTL, overriding the system default cache location.
    #[cfg(feature = "cache")]
    pub fn with_cache(
        mut self,
        path: impl Into<std::path::PathBuf>,
        ttl: std::time::Duration,
    ) -> Self {
        self.cache_path = Some(path.into());
        self.cache_ttl = Some(ttl);
        self
    }

    /// Customizes the TTL duration while keeping the system default cache path.
    #[cfg(feature = "cache")]
    pub fn with_cache_ttl(mut self, ttl: std::time::Duration) -> Self {
        self.cache_ttl = Some(ttl);
        self
    }

    /// Disables filesystem caching for this client instance.
    #[cfg(feature = "cache")]
    pub fn without_cache(mut self) -> Self {
        self.cache_path = None;
        self
    }

    /// Authenticates with the GDTF Share API synchronously.
    ///
    /// This function is usefull for when you want to prevent lazy-login on other API calls.
    pub fn login(&mut self) -> Result<()> {
        let req = self.login_req(self.username.expose_secret(), self.password.expose_secret())?;
        let res = ehttp::fetch_blocking(&req).map_err(Error::ClientError)?;
        self.handle_login_res(res)?;
        Ok(())
    }

    /// Authenticates with the GDTF Share API asynchronously.
    ///
    /// This function is usefull for when you want to prevent lazy-login on other API calls.
    #[cfg(feature = "async")]
    pub async fn login_async(&mut self) -> Result<()> {
        let req = self.login_req(self.username.expose_secret(), self.password.expose_secret())?;
        let res = ehttp::fetch_async(req).await.map_err(Error::ClientError)?;
        self.handle_login_res(res)?;
        Ok(())
    }

    fn login_req(&self, user: &str, password: &str) -> Result<ehttp::Request> {
        const URL_LOGIN: &str = "https://gdtf-share.com/apis/public/login.php";
        let body = serde_json::to_vec(&serde_json::json! {{
            "user": user,
            "password": password
        }})
        .map_err(Error::Json)?;
        let req = ehttp::Request::post(URL_LOGIN, body)
            .with_headers(ehttp::Headers::new(&[("Content-Type", "application/json")]));
        Ok(req)
    }

    fn handle_login_res(&mut self, res: ehttp::Response) -> Result<()> {
        match res.status {
            200 => {
                let cookie = res.headers.get("set-cookie").ok_or(Error::MissingCookie)?.to_string();
                self.session = Some(Session { cookie: secrecy::SecretString::from(cookie) });
                Ok(())
            }
            400 => Err(Error::BadRequest),
            401 => Err(Error::Unauthorized),
            code => Err(Error::UnexpectedStatusCode(code)),
        }
    }

    /// Tries to log in if it's not, then fetches the fixture entry list synchronously
    ///
    /// If the `cache` feature is enabled, this will attempt to read from the cache first,
    /// and write to the cache if the request is successful.
    pub fn get_list(&mut self) -> Result<Vec<Entry>> {
        #[cfg(feature = "cache")]
        if let Some(entries) = self.read_cache() {
            return Ok(entries);
        }

        let cookie = self.session()?.cookie.clone();
        let req = self.get_list_req(&cookie)?;
        let res = ehttp::fetch_blocking(&req).map_err(Error::ClientError)?;

        #[cfg(feature = "cache")]
        if res.status == 200 {
            self.write_cache(&res.bytes);
        }

        let list = self.handle_get_list_res(&res)?;
        Ok(list)
    }

    /// Tries to log in if it's not, then fetches the fixture entry list asynchronously.
    ///
    /// If the `cache` feature is enabled, this will attempt to read from the cache first,
    /// and write to the cache if the request is successful.
    #[cfg(feature = "async")]
    pub async fn get_list_async(&mut self) -> Result<Vec<Entry>> {
        #[cfg(feature = "cache")]
        if let Some(entries) = self.read_cache() {
            return Ok(entries);
        }

        let cookie = self.session_async().await?.cookie.clone();
        let req = self.get_list_req(&cookie)?;
        let res = ehttp::fetch_async(req).await.map_err(Error::ClientError)?;

        #[cfg(feature = "cache")]
        if res.status == 200 {
            self.write_cache(&res.bytes);
        }

        let list = self.handle_get_list_res(&res)?;
        Ok(list)
    }

    fn get_list_req(&self, cookie: &secrecy::SecretString) -> Result<ehttp::Request> {
        const URL_GET_LIST: &str = "https://gdtf-share.com/apis/public/getList.php";
        let req = ehttp::Request::get(URL_GET_LIST)
            .with_headers(ehttp::Headers::new(&[("Cookie", cookie.expose_secret())]));
        Ok(req)
    }

    fn handle_get_list_res(&self, res: &ehttp::Response) -> Result<Vec<Entry>> {
        match res.status {
            200 => self.handle_get_list_bytes(&res.bytes),
            400 => Err(Error::BadRequest),
            401 => Err(Error::Unauthorized),
            code => Err(Error::UnexpectedStatusCode(code)),
        }
    }

    fn handle_get_list_bytes(&self, bytes: &[u8]) -> Result<Vec<Entry>> {
        #[derive(serde::Deserialize)]
        struct GetListResponse {
            // NOTE: I'm ignoring `result` as it should always be true.
            // NOTE: I'm ignoring `timestamp` as it seems to be missing from the actual response...
            pub list: Vec<Entry>,
        }

        serde_json::from_slice::<GetListResponse>(bytes).map_err(Error::Json).map(|r| r.list)
    }

    fn session(&mut self) -> Result<&Session> {
        if self.session.is_none() {
            self.login()?;
        }

        self.session.as_ref().ok_or(Error::Unauthorized)
    }

    /// Tries to log in if it's not, then downloads a GDTF zip archive by Revision ID (RID) synchronously.
    pub fn download(&mut self, rid: RevisionId) -> Result<Vec<u8>> {
        let cookie = self.session()?.cookie.clone();
        let req = self.download_req(rid, &cookie)?;
        let res = ehttp::fetch_blocking(&req).map_err(Error::ClientError)?;
        let list = self.handle_download_res(res)?;
        Ok(list)
    }

    /// Tries to log in if it's not, then downloads a GDTF zip archive by Revision ID (RID) asynchronously.
    #[cfg(feature = "async")]
    pub async fn download_async(&mut self, rid: RevisionId) -> Result<Vec<u8>> {
        let cookie = self.session_async().await?.cookie.clone();
        let req = self.download_req(rid, &cookie)?;
        let res = ehttp::fetch_async(req).await.map_err(Error::ClientError)?;
        let list = self.handle_download_res(res)?;
        Ok(list)
    }

    fn download_req(
        &self,
        rid: RevisionId,
        cookie: &secrecy::SecretString,
    ) -> Result<ehttp::Request> {
        const URL_DOWNLOAD: &str = "https://gdtf-share.com/apis/public/downloadFile.php";
        let req = ehttp::Request::get(format!("{URL_DOWNLOAD}?rid={rid}"))
            .with_headers(ehttp::Headers::new(&[("Cookie", cookie.expose_secret())]));
        Ok(req)
    }

    fn handle_download_res(&self, res: ehttp::Response) -> Result<Vec<u8>> {
        match res.status {
            200 => Ok(res.bytes),
            400 => Err(Error::BadRequest),
            401 => Err(Error::Unauthorized),
            404 => Err(Error::NotFound),
            code => Err(Error::UnexpectedStatusCode(code)),
        }
    }

    #[cfg(feature = "async")]
    async fn session_async(&mut self) -> Result<&Session> {
        if self.session.is_none() {
            self.login_async().await?;
        }

        self.session.as_ref().ok_or(Error::Unauthorized)
    }

    #[cfg(feature = "cache")]
    fn read_cache(&self) -> Option<Vec<Entry>> {
        let path = self.cache_path.as_ref()?;
        let ttl = self.cache_ttl?;

        let metadata = std::fs::metadata(path).ok()?;
        let modified = metadata.modified().ok()?;

        if modified.elapsed().unwrap_or_default() > ttl {
            return None;
        }

        let bytes = std::fs::read(path).ok()?;
        self.handle_get_list_bytes(&bytes).ok()
    }

    #[cfg(feature = "cache")]
    fn write_cache(&self, bytes: &[u8]) {
        if let Some(path) = &self.cache_path {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(path, bytes);
        }
    }
}

#[cfg(feature = "cache")]
fn default_cache_path() -> Option<std::path::PathBuf> {
    dirs::cache_dir().map(|p| p.join("gdtf-share").join("get_list.json"))
}
