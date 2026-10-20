use std::io::Read;
use std::time::Duration;

use reqwest::blocking::Client;
use reqwest::{Method, Url};
use serde::Serialize;
use serde::de::DeserializeOwned;

pub struct ServerClient {
    pub server: String,
    pub identity: uuid::Uuid,
    http: Client,
}

pub fn server_origin(value: &str) -> Result<String, String> {
    let url = Url::parse(value).map_err(|_| "Enter the local or Tailscale Server URL")?;
    if !((url.scheme() == "http"
        && matches!(url.host_str(), Some("127.0.0.1" | "[::1]" | "localhost")))
        || (url.scheme() == "https"
            && url.host_str().is_some_and(|host| host.ends_with(".ts.net"))))
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(
            "Use loopback HTTP or Tailscale HTTPS with no path, query or user information".into(),
        );
    }
    Ok(url.as_str().trim_end_matches('/').to_owned())
}

impl ServerClient {
    pub fn new(server: &str) -> Result<Self, String> {
        let server = server_origin(server)?;
        Ok(Self {
            identity: uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, server.as_bytes()),
            server,
            http: Client::builder()
                .no_proxy()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(Duration::from_secs(15))
                .build()
                .map_err(|_| "Cannot initialize the Server connection")?,
        })
    }

    pub fn post<T: Serialize, R: DeserializeOwned>(
        &self,
        path: &str,
        value: &T,
    ) -> Result<R, String> {
        self.request(Method::POST, path, Some(value))
    }

    pub fn get<R: DeserializeOwned>(&self, path: &str) -> Result<R, String> {
        self.request::<(), R>(Method::GET, path, None)
    }

    fn request<T: Serialize, R: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        value: Option<&T>,
    ) -> Result<R, String> {
        let mut request = self.http.request(method, format!("{}{path}", self.server));
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
                    .unwrap_or("check Tailscale connectivity, supplied content and Server status")
            ));
        }
        serde_json::from_slice(&bytes)
            .map_err(|_| "Invalid Server response; retry to reconcile".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_origin_defines_stable_nonsecret_connection_identity() {
        let first = ServerClient::new("http://127.0.0.1:8000").unwrap();
        let repeated = ServerClient::new("http://127.0.0.1:8000/").unwrap();
        let other = ServerClient::new("https://gym.example.ts.net").unwrap();
        assert_eq!(first.identity, repeated.identity);
        assert_ne!(first.identity, other.identity);
    }

    #[test]
    fn only_local_http_or_tailnet_https_origins_are_accepted() {
        for url in [
            "https://gym.example.ts.net",
            "http://127.0.0.1:8000",
            "http://localhost:8000",
        ] {
            assert!(server_origin(url).is_ok(), "{url}");
        }
        for url in [
            "http://gym.example.ts.net",
            "https://example.com",
            "http://192.168.1.2",
            "https://gym.example.ts.net/path",
            "https://user:secret@gym.example.ts.net",
            "https://gym.example.ts.net?token=x",
        ] {
            assert!(server_origin(url).is_err(), "{url}");
        }
    }
}
