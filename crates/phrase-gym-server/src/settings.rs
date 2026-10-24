use std::path::PathBuf;

const LOOPBACK_HOSTS: [&str; 2] = ["127.0.0.1", "localhost"];

#[derive(Clone)]
pub struct ServerSettings {
    pub allowed_hosts: Vec<String>,
    pub allowed_origins: Vec<String>,
    pub client_dir: Option<PathBuf>,
}

impl ServerSettings {
    pub fn loopback() -> Self {
        Self {
            allowed_hosts: LOOPBACK_HOSTS.map(str::to_owned).to_vec(),
            allowed_origins: Vec::new(),
            client_dir: None,
        }
    }

    pub fn with_tailscale_origin(mut self, origin: &str) -> Result<Self, String> {
        let authority = origin.strip_prefix("https://").unwrap_or_default();
        let (host, port) = authority
            .split_once(':')
            .map_or((authority, None), |(host, port)| (host, Some(port)));
        let valid_host = host.ends_with(".ts.net")
            && !host.starts_with('.')
            && host
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b".-".contains(&byte));
        // Browsers omit the default port from Origin, so an explicit :443 could never match.
        let valid_port = port.is_none_or(|port| {
            port != "443" && !port.starts_with('0') && port.parse::<u16>().is_ok()
        });
        if !valid_host || !valid_port {
            return Err(
                "Use the exact https://machine.tailnet.ts.net[:port] Tailscale Serve origin, without the default :443".into(),
            );
        }
        self.allowed_hosts.push(host.to_owned());
        self.allowed_origins.push(origin.to_owned());
        Ok(self)
    }

    #[must_use]
    pub fn with_hosts(mut self, hosts: impl IntoIterator<Item = String>) -> Self {
        for host in hosts {
            if !self.allowed_hosts.contains(&host) {
                self.allowed_hosts.push(host);
            }
        }
        self
    }
}
