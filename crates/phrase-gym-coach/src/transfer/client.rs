use std::io::Read;
use std::time::Duration;

use reqwest::blocking::Client;
use reqwest::{Method, Url};
use serde::Serialize;
use serde::de::DeserializeOwned;

pub struct ServerClient {
    pub server: String,
    http: Client,
    submit: String,
    worker: String,
}

pub fn loopback(value: &str) -> Result<String, String> {
    let url = Url::parse(value).map_err(|_| "Enter a loopback Server URL")?;
    if url.scheme() != "http"
        || !matches!(url.host_str(), Some("127.0.0.1" | "[::1]" | "localhost"))
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err("This attended preview requires http://127.0.0.1:PORT with no path or credentials in the URL".into());
    }
    Ok(url.as_str().trim_end_matches('/').to_owned())
}

impl ServerClient {
    pub fn new(server: &str, submit: String, worker: String) -> Result<Self, String> {
        let server = loopback(server)?;
        if submit.is_empty() || worker.is_empty() {
            return Err("Configure separate material-submit and worker credentials".into());
        }
        Ok(Self {
            server,
            http: Client::builder()
                .no_proxy()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(Duration::from_secs(15))
                .build()
                .map_err(|_| "Cannot initialize the Server connection")?,
            submit,
            worker,
        })
    }

    pub fn post<T: Serialize, R: DeserializeOwned>(
        &self,
        path: &str,
        value: &T,
        worker: bool,
    ) -> Result<R, String> {
        self.request(Method::POST, path, Some(value), worker)
    }

    pub fn get<R: DeserializeOwned>(&self, path: &str) -> Result<R, String> {
        self.request::<(), R>(Method::GET, path, None, false)
    }

    fn request<T: Serialize, R: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        value: Option<&T>,
        worker: bool,
    ) -> Result<R, String> {
        let mut request = self
            .http
            .request(method, format!("{}{path}", self.server))
            .bearer_auth(if worker { &self.worker } else { &self.submit });
        if let Some(value) = value {
            request = request.json(value);
        }
        let response = request.send().map_err(
            |_| "Server unavailable or response lost; Retry preserves the same submission",
        )?;
        let status = response.status();
        let mut bytes = Vec::new();
        response
            .take(100_001)
            .read_to_end(&mut bytes)
            .map_err(|_| "Server response interrupted; retry to reconcile")?;
        if bytes.len() > 100_000 {
            return Err("Server response exceeds the preview limit".into());
        }
        if !status.is_success() {
            let reason = serde_json::from_slice::<serde_json::Value>(&bytes)
                .ok()
                .and_then(|v| v.get("detail").and_then(|s| s.as_str()).map(str::to_owned));
            return Err(format!(
                "Server {status}: {}",
                reason
                    .as_deref()
                    .unwrap_or("check scoped credentials, supplied content and Server status")
            ));
        }
        serde_json::from_slice(&bytes)
            .map_err(|_| "Invalid Server response; retry to reconcile".into())
    }
}
