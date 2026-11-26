//! HTTP client for interacting with the GDTF Share API.

use secrecy::ExposeSecret;

use super::*;

#[cfg(feature = "cache")]
const LIST_CACHE_FILE: &str = "get_list.json";

#[cfg(feature = "cache")]
const DOWNLOADS_CACHE_DIR: &str = "downloads";

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
    cache_enabled: bool,
    #[cfg(feature = "cache")]
    cache_override_path: Option<std::path::PathBuf>,
    #[cfg(feature = "cache")]
    default_cache_path: Option<std::path::PathBuf>,
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
            cache_enabled: true,
            #[cfg(feature = "cache")]
            cache_override_path: None,
            #[cfg(feature = "cache")]
            default_cache_path: default_cache_path(),
            #[cfg(feature = "cache")]
            cache_ttl: Some(std::time::Duration::from_secs(24 * 60 * 60)),

            username: secrecy::SecretString::from(username),
            password: secrecy::SecretString::from(password),
        }
    }

    /// Sets a custom cache directory path override and TTL.
    ///
    /// When searching for cached entries or downloads, the override cache directory will be checked
    /// first. If an item is not found in the override cache, the client falls back to the system
    /// default cache directory.
    ///
    /// The fixture list cache and downloaded GDTF archives will be stored inside this directory.
    #[cfg(feature = "cache")]
    pub fn with_cache(
        mut self,
        path: impl Into<std::path::PathBuf>,
        ttl: std::time::Duration,
    ) -> Self {
        self.cache_enabled = true;
        self.cache_override_path = Some(path.into());
        self.cache_ttl = Some(ttl);
        self
    }

    /// Sets a custom cache directory path override while keeping the default TTL.
    ///
    /// When searching for cached entries or downloads, the override cache directory will be checked
    /// first. If an item is not found in the override cache, the client falls back to the system
    /// default cache directory.
    ///
    /// The fixture list cache and downloaded GDTF archives will be stored inside this directory.
    #[cfg(feature = "cache")]
    pub fn with_cache_path(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.cache_enabled = true;
        self.cache_override_path = Some(path.into());
        self
    }

    /// Customizes the TTL duration while keeping the cache path settings.
    #[cfg(feature = "cache")]
    pub fn with_cache_ttl(mut self, ttl: std::time::Duration) -> Self {
        self.cache_ttl = Some(ttl);
        self
    }

    /// Disables filesystem caching for this client instance.
    #[cfg(feature = "cache")]
    pub fn without_cache(mut self) -> Self {
        self.cache_enabled = false;
        self
    }

    /// Authenticates with the GDTF Share API synchronously.
    ///
    /// This function is useful for when you want to prevent lazy-login on other API calls.
    pub fn login(&mut self) -> Result<()> {
        #[cfg(feature = "log")]
        log::debug!("Attempting synchronous login");

        let req = self.login_req(self.username.expose_secret(), self.password.expose_secret())?;
        let res = ehttp::fetch_blocking(&req).map_err(Error::ClientError)?;
        self.handle_login_res(res)?;
        Ok(())
    }

    /// Authenticates with the GDTF Share API asynchronously.
    ///
    /// This function is useful for when you want to prevent lazy-login on other API calls.
    #[cfg(feature = "async")]
    pub async fn login_async(&mut self) -> Result<()> {
        #[cfg(feature = "log")]
        log::debug!("Attempting asynchronous login");

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
                #[cfg(feature = "log")]
                log::debug!("Login successful");

                let cookie = res.headers.get("set-cookie").ok_or(Error::MissingCookie)?.to_string();
                self.session = Some(Session { cookie: secrecy::SecretString::from(cookie) });
                Ok(())
            }
            400 => Err(Error::BadRequest),
            401 => Err(Error::Unauthorized),
            code => Err(Error::UnexpectedStatusCode(code)),
        }
    }

    /// Tries to log in if not already authenticated, then fetches the fixture entry list synchronously.
    ///
    /// If the `cache` feature is enabled, this will attempt to read from the cache first,
    /// and write to the cache if the request is successful.
    pub fn get_list(&mut self) -> Result<Vec<Entry>> {
        #[cfg(feature = "log")]
        log::debug!("Fetching fixture entry list synchronously");

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

    /// Tries to log in if not already authenticated, then fetches the fixture entry list asynchronously.
    ///
    /// If the `cache` feature is enabled, this will attempt to read from the cache first,
    /// and write to the cache if the request is successful.
    #[cfg(feature = "async")]
    pub async fn get_list_async(&mut self) -> Result<Vec<Entry>> {
        #[cfg(feature = "log")]
        log::debug!("Fetching fixture entry list asynchronously");

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

        let parsed = serde_json::from_slice::<GetListResponse>(bytes).map_err(Error::Json)?;
        #[cfg(feature = "log")]
        log::debug!("Successfully parsed {} fixture entries", parsed.list.len());

        Ok(parsed.list)
    }

    fn session(&mut self) -> Result<&Session> {
        if self.session.is_none() {
            #[cfg(feature = "log")]
            log::debug!("No active session found; initiating login");
            self.login()?;
        }

        self.session.as_ref().ok_or(Error::Unauthorized)
    }

    /// Tries to log in if not already authenticated, then downloads a GDTF zip archive by Revision ID (RID) synchronously.
    ///
    /// If the `cache` feature is enabled, this will attempt to read from the download cache first,
    /// and write to the download cache if the request is successful.
    pub fn download(&mut self, rid: RevisionId) -> Result<Vec<u8>> {
        #[cfg(feature = "log")]
        log::debug!("Downloading GDTF archive for RID {rid} synchronously");

        #[cfg(feature = "cache")]
        if let Some(bytes) = self.read_download_cache(rid) {
            return Ok(bytes);
        }

        let cookie = self.session()?.cookie.clone();
        let req = self.download_req(rid, &cookie)?;
        let res = ehttp::fetch_blocking(&req).map_err(Error::ClientError)?;
        let bytes = self.handle_download_res(res)?;

        #[cfg(feature = "cache")]
        self.write_download_cache(rid, &bytes);

        Ok(bytes)
    }

    /// Tries to log in if not already authenticated, then downloads a GDTF zip archive by Revision ID (RID) asynchronously.
    ///
    /// If the `cache` feature is enabled, this will attempt to read from the download cache first,
    /// and write to the download cache if the request is successful.
    #[cfg(feature = "async")]
    pub async fn download_async(&mut self, rid: RevisionId) -> Result<Vec<u8>> {
        #[cfg(feature = "log")]
        log::debug!("Downloading GDTF archive for RID {rid} asynchronously");

        #[cfg(feature = "cache")]
        if let Some(bytes) = self.read_download_cache(rid) {
            return Ok(bytes);
        }

        let cookie = self.session_async().await?.cookie.clone();
        let req = self.download_req(rid, &cookie)?;
        let res = ehttp::fetch_async(req).await.map_err(Error::ClientError)?;
        let bytes = self.handle_download_res(res)?;

        #[cfg(feature = "cache")]
        self.write_download_cache(rid, &bytes);

        Ok(bytes)
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
            200 => {
                #[cfg(feature = "log")]
                log::debug!("Download successful ({} bytes)", res.bytes.len());
                Ok(res.bytes)
            }
            400 => Err(Error::BadRequest),
            401 => Err(Error::Unauthorized),
            404 => Err(Error::NotFound),
            code => Err(Error::UnexpectedStatusCode(code)),
        }
    }

    #[cfg(feature = "async")]
    async fn session_async(&mut self) -> Result<&Session> {
        if self.session.is_none() {
            #[cfg(feature = "log")]
            log::debug!("No active session found; initiating async login");
            self.login_async().await?;
        }

        self.session.as_ref().ok_or(Error::Unauthorized)
    }

    /// Caches a GDTF archive for a given Revision ID (RID) to the filesystem.
    #[cfg(feature = "cache")]
    pub fn cache_gdtf_revision(&mut self, rid: RevisionId, gdtf_bytes: &[u8]) -> Result<()> {
        self.write_download_cache(rid, gdtf_bytes);
        Ok(())
    }

    #[cfg(feature = "cache")]
    fn target_cache_dir(&self) -> Option<&std::path::Path> {
        if !self.cache_enabled {
            return None;
        }
        self.cache_override_path.as_deref().or(self.default_cache_path.as_deref())
    }

    #[cfg(feature = "cache")]
    fn list_cache_path(&self) -> Option<std::path::PathBuf> {
        Some(self.target_cache_dir()?.join(LIST_CACHE_FILE))
    }

    #[cfg(feature = "cache")]
    fn try_read_list_cache_at(&self, path: &std::path::Path) -> Option<Vec<Entry>> {
        let ttl = self.cache_ttl?;

        let metadata = std::fs::metadata(path).ok()?;
        let modified = metadata.modified().ok()?;

        if modified.elapsed().unwrap_or_default() > ttl {
            #[cfg(feature = "log")]
            log::debug!("Cache expired at path: {:?}", path);
            return None;
        }

        let bytes = std::fs::read(path).ok()?;
        let entries = self.handle_get_list_bytes(&bytes).ok()?;

        #[cfg(feature = "log")]
        log::debug!("Cache hit: loaded {} entries from {:?}", entries.len(), path);

        Some(entries)
    }

    #[cfg(feature = "cache")]
    fn read_cache(&self) -> Option<Vec<Entry>> {
        if !self.cache_enabled {
            return None;
        }

        if let Some(ref override_path) = self.cache_override_path {
            let path = override_path.join(LIST_CACHE_FILE);
            if let Some(entries) = self.try_read_list_cache_at(&path) {
                return Some(entries);
            }
        }

        if let Some(ref default_path) = self.default_cache_path {
            if self.cache_override_path.as_ref() != Some(default_path) {
                let path = default_path.join(LIST_CACHE_FILE);
                if let Some(entries) = self.try_read_list_cache_at(&path) {
                    return Some(entries);
                }
            }
        }

        None
    }

    #[cfg(feature = "cache")]
    fn write_cache(&self, bytes: &[u8]) {
        if let Some(path) = self.list_cache_path() {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if std::fs::write(&path, bytes).is_ok() {
                #[cfg(feature = "log")]
                log::debug!("Wrote {} bytes to cache at {:?}", bytes.len(), path);
            }
        }
    }

    #[cfg(feature = "cache")]
    fn download_cache_path(&self, rid: RevisionId) -> Option<std::path::PathBuf> {
        Some(self.target_cache_dir()?.join(DOWNLOADS_CACHE_DIR).join(format!("{rid}.gdtf")))
    }

    #[cfg(feature = "cache")]
    fn try_read_download_cache_at(
        &self,
        path: &std::path::Path,
        rid: RevisionId,
    ) -> Option<Vec<u8>> {
        let ttl = self.cache_ttl?;

        let metadata = std::fs::metadata(path).ok()?;
        let modified = metadata.modified().ok()?;

        if modified.elapsed().unwrap_or_default() > ttl {
            #[cfg(feature = "log")]
            log::debug!("Download cache expired at path: {:?}", path);
            return None;
        }

        let bytes = std::fs::read(path).ok()?;
        #[cfg(feature = "log")]
        log::debug!("Cache hit: loaded {} bytes for RID {} from {:?}", bytes.len(), rid, path);

        Some(bytes)
    }

    #[cfg(feature = "cache")]
    fn read_download_cache(&self, rid: RevisionId) -> Option<Vec<u8>> {
        if !self.cache_enabled {
            return None;
        }

        if let Some(ref override_path) = self.cache_override_path {
            let path = override_path.join(DOWNLOADS_CACHE_DIR).join(format!("{rid}.gdtf"));
            if let Some(bytes) = self.try_read_download_cache_at(&path, rid) {
                return Some(bytes);
            }
        }

        if let Some(ref default_path) = self.default_cache_path {
            if self.cache_override_path.as_ref() != Some(default_path) {
                let path = default_path.join(DOWNLOADS_CACHE_DIR).join(format!("{rid}.gdtf"));
                if let Some(bytes) = self.try_read_download_cache_at(&path, rid) {
                    return Some(bytes);
                }
            }
        }

        None
    }

    #[cfg(feature = "cache")]
    fn write_download_cache(&self, rid: RevisionId, bytes: &[u8]) {
        if let Some(path) = self.download_cache_path(rid) {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if std::fs::write(&path, bytes).is_ok() {
                #[cfg(feature = "log")]
                log::debug!("Wrote {} bytes to download cache at {:?}", bytes.len(), path);
            }
        }
    }
}

#[cfg(feature = "cache")]
fn default_cache_path() -> Option<std::path::PathBuf> {
    dirs::cache_dir().map(|p| p.join("gdtf-share"))
}
