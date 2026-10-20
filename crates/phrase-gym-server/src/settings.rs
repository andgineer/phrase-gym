use std::path::PathBuf;

const LOOPBACK_HOSTS: [&str; 2] = ["127.0.0.1", "localhost"];

#[derive(Clone)]
pub struct ServerSettings {
    pub tailscale_origin: Option<String>,
    pub allowed_hosts: Vec<String>,
    pub allowed_origins: Vec<String>,
    pub client_dir: Option<PathBuf>,
}

impl ServerSettings {
    pub fn loopback() -> Self {
        Self {
            tailscale_origin: None,
            allowed_hosts: LOOPBACK_HOSTS.map(str::to_owned).to_vec(),
            allowed_origins: Vec::new(),
            client_dir: None,
        }
    }

    pub fn with_tailscale_origin(mut self, origin: &str) -> Result<Self, String> {
        let host = origin
            .strip_prefix("https://")
            .filter(|host| host.ends_with(".ts.net") && !host.starts_with('.'))
            .filter(|host| {
                host.bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b".-".contains(&byte))
            })
            .ok_or("Use the exact https://machine.tailnet.ts.net Tailscale Serve origin")?;
        self.allowed_hosts.push(host.to_owned());
        self.tailscale_origin = Some(origin.to_owned());
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
