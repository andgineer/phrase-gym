use std::path::PathBuf;

use crate::authority::Authority;
use std::sync::Arc;

const LOOPBACK_HOSTS: [&str; 2] = ["127.0.0.1", "localhost"];

#[derive(Clone)]
pub struct ServerSettings {
    pub authority: Arc<Authority>,
    pub allowed_hosts: Vec<String>,
    pub allowed_origins: Vec<String>,
    pub client_dir: Option<PathBuf>,
}

impl ServerSettings {
    pub fn loopback() -> Self {
        Self {
            authority: Arc::new(Authority::default()),
            allowed_hosts: LOOPBACK_HOSTS.map(str::to_owned).to_vec(),
            allowed_origins: Vec::new(),
            client_dir: None,
        }
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
