//! HTTP client for interacting with the GDTF Share API.

use super::*;

/// API client for managing a GDTF Share session.
#[derive(Default, Debug)]
pub struct Client {
    session: Option<Session>,
}

impl Client {
    /// Creates a new unauthenticated GDTF Share API client.
    pub fn new() -> Self {
        Self { session: None }
    }

    /// Authenticates with the GDTF Share API synchronously.
    ///
    /// # Errors
    ///
    /// * [`Error::Unauthorized`] - Invalid username or password.
    /// * [`Error::BadRequest`] - The request payload was malformed.
    /// * [`Error::MissingCookie`] - The response missing the `Set-Cookie` header.
    /// * [`Error::ClientError`] - Transport or network failure.
    pub fn login(&mut self, user: &str, password: &str) -> Result<()> {
        let req = self.login_req(user, password)?;
        let res = ehttp::fetch_blocking(&req).map_err(Error::ClientError)?;
        self.handle_login_res(res)?;
        Ok(())
    }

    /// Authenticates with the GDTF Share API asynchronously.
    ///
    /// # Errors
    ///
    /// * [`Error::Unauthorized`] - Invalid username or password.
    /// * [`Error::BadRequest`] - The request payload was malformed.
    /// * [`Error::MissingCookie`] - The response missing the `Set-Cookie` header.
    /// * [`Error::ClientError`] - Transport or network failure.
    #[cfg(feature = "async")]
    pub async fn login_async(&mut self, user: &str, password: &str) -> Result<()> {
        let req = self.login_req(user, password)?;
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
                self.session = Some(Session { cookie });
                Ok(())
            }
            400 => Err(Error::BadRequest),
            401 => Err(Error::Unauthorized),
            code => Err(Error::UnexpectedStatusCode(code)),
        }
    }

    /// Fetches the fixture entry list synchronously.
    ///
    /// # Errors
    ///
    /// * [`Error::NoSession`] - Client is unauthenticated.
    /// * [`Error::Unauthorized`] - Session cookie has expired or is invalid.
    /// * [`Error::Json`] - Failed to deserialize response JSON.
    /// * [`Error::ClientError`] - Transport or network failure.
    pub fn get_list(&self) -> Result<Vec<Entry>> {
        let req = self.get_list_req()?;
        let res = ehttp::fetch_blocking(&req).map_err(Error::ClientError)?;
        let list = self.handle_get_list_res(res)?;
        Ok(list)
    }

    /// Fetches the fixture entry list asynchronously.
    ///
    /// # Errors
    ///
    /// * [`Error::NoSession`] - Client is unauthenticated.
    /// * [`Error::Unauthorized`] - Session cookie has expired or is invalid.
    /// * [`Error::Json`] - Failed to deserialize response JSON.
    /// * [`Error::ClientError`] - Transport or network failure.
    #[cfg(feature = "async")]
    pub async fn get_list_async(&self) -> Result<Vec<Entry>> {
        let req = self.get_list_req()?;
        let res = ehttp::fetch_async(req).await.map_err(Error::ClientError)?;
        let list = self.handle_get_list_res(res)?;
        Ok(list)
    }

    fn get_list_req(&self) -> Result<ehttp::Request> {
        const URL_GET_LIST: &str = "https://gdtf-share.com/apis/public/getList.php";
        let Some(session) = &self.session else { return Err(Error::NoSession) };
        let req = ehttp::Request::get(URL_GET_LIST)
            .with_headers(ehttp::Headers::new(&[("Cookie", &session.cookie)]));
        Ok(req)
    }

    fn handle_get_list_res(&self, res: ehttp::Response) -> Result<Vec<Entry>> {
        #[derive(serde::Deserialize)]
        struct GetListResponse {
            // NOTE: I'm ignoring `result` as it should always be true.
            // NOTE: I'm ignoring `timestamp` as it seems to be missing from the actual response...
            pub list: Vec<Entry>,
        }

        match res.status {
            200 => serde_json::from_slice::<GetListResponse>(&res.bytes)
                .map_err(Error::Json)
                .map(|r| r.list),
            400 => Err(Error::BadRequest),
            401 => Err(Error::Unauthorized),
            code => Err(Error::UnexpectedStatusCode(code)),
        }
    }

    /// Downloads a GDTF zip archive by Revision ID (RID) synchronously.
    ///
    /// # Errors
    ///
    /// * [`Error::NoSession`] - Client is unauthenticated.
    /// * [`Error::NotFound`] - Revision ID (`rid`) does not exist.
    /// * [`Error::Unauthorized`] - Session cookie has expired or is invalid.
    /// * [`Error::ClientError`] - Transport or network failure.
    pub fn download(&self, rid: u32) -> Result<Vec<u8>> {
        let req = self.download_req(rid)?;
        let res = ehttp::fetch_blocking(&req).map_err(Error::ClientError)?;
        let list = self.handle_download_res(res)?;
        Ok(list)
    }

    /// Downloads a GDTF zip archive by Revision ID (RID) asynchronously.
    ///
    /// # Errors
    ///
    /// * [`Error::NoSession`] - Client is unauthenticated.
    /// * [`Error::NotFound`] - Revision ID (`rid`) does not exist.
    /// * [`Error::Unauthorized`] - Session cookie has expired or is invalid.
    /// * [`Error::ClientError`] - Transport or network failure.
    #[cfg(feature = "async")]
    pub async fn download_async(&self, rid: u32) -> Result<Vec<u8>> {
        let req = self.download_req(rid)?;
        let res = ehttp::fetch_async(req).await.map_err(Error::ClientError)?;
        let list = self.handle_download_res(res)?;
        Ok(list)
    }

    fn download_req(&self, rid: u32) -> Result<ehttp::Request> {
        const URL_DOWNLOAD: &str = "https://gdtf-share.com/apis/public/downloadFile.php";
        let Some(session) = &self.session else { return Err(Error::NoSession) };
        let req = ehttp::Request::get(format!("{URL_DOWNLOAD}?rid={rid}"))
            .with_headers(ehttp::Headers::new(&[("Cookie", &session.cookie)]));
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
}
