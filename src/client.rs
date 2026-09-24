use super::*;

#[derive(Default)]
pub struct Client {
    session: Option<Session>,
}

impl Client {
    pub fn new() -> Self {
        Self { session: None }
    }

    pub fn login(&mut self, user: &str, password: &str) -> Result<()> {
        let req = self.login_req(user, password)?;
        let res = ehttp::fetch_blocking(&req).map_err(|e| Error::ClientError(e.to_string()))?;
        self.handle_login_res(res)?;
        Ok(())
    }

    #[cfg(feature = "async")]
    pub async fn login_async(&mut self, user: &str, password: &str) -> Result<()> {
        let req = self.login_req(user, password)?;
        let res = ehttp::fetch_async(req).await.map_err(|e| Error::ClientError(e.to_string()))?;
        self.handle_login_res(res)?;
        Ok(())
    }

    fn login_req(&self, user: &str, password: &str) -> Result<ehttp::Request> {
        const URL_LOGIN: &str = "https://gdtf-share.com/apis/public/login.php";

        let body = serde_json::to_vec(&serde_json::json! {{
            "user": user,
            "password": password
        }})
        .map_err(crate::Error::Json)?;

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

    pub fn get_list(&mut self) -> Result<Vec<crate::Entry>> {
        let req = self.get_list_req()?;
        let res = ehttp::fetch_blocking(&req).map_err(|e| Error::ClientError(e.to_string()))?;
        let list = self.handle_get_list_res(res)?;
        Ok(list)
    }

    #[cfg(feature = "async")]
    pub async fn get_list_async(&mut self) -> Result<Vec<crate::Entry>> {
        let req = self.get_list_req()?;
        let res = ehttp::fetch_async(req).await.map_err(|e| Error::ClientError(e.to_string()))?;
        let list = self.handle_get_list_res(res)?;
        Ok(list)
    }

    fn get_list_req(&self) -> Result<ehttp::Request> {
        const URL_GET_LIST: &str = "https://gdtf-share.com/apis/public/getList.php";

        let Some(session) = &self.session else { return Err(crate::Error::NoSession) };

        let req = ehttp::Request::get(URL_GET_LIST)
            .with_headers(ehttp::Headers::new(&[("Cookie", &session.cookie)]));

        Ok(req)
    }

    fn handle_get_list_res(&self, res: ehttp::Response) -> Result<Vec<crate::Entry>> {
        #[derive(serde::Deserialize)]
        struct GetListResponse {
            // NOTE: I'm ignoring `result` as it should always be true.
            // NOTE: I'm ignoring `timestamp` as it seems to be missing from the actual response...
            pub list: Vec<crate::Entry>,
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

    pub fn download(&self, rid: i64) -> Result<Vec<u8>> {
        let req = self.download_req(rid)?;
        let res = ehttp::fetch_blocking(&req).map_err(|e| Error::ClientError(e.to_string()))?;
        let list = self.handle_download_res(res)?;
        Ok(list)
    }

    #[cfg(feature = "async")]
    pub async fn download_async(&self, rid: i64) -> Result<Vec<u8>> {
        let req = self.download_req(rid)?;
        let res = ehttp::fetch_async(req).await.map_err(|e| Error::ClientError(e.to_string()))?;
        let list = self.handle_download_res(res)?;
        Ok(list)
    }

    fn download_req(&self, rid: i64) -> Result<ehttp::Request> {
        const URL_DOWNLOAD: &str = "https://gdtf-share.com/apis/public/downloadFile.php";
        let Some(session) = &self.session else { return Err(crate::Error::NoSession) };
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
