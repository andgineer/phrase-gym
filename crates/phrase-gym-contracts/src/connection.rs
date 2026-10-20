pub const DEFAULT_SERVER_PORT: u16 = 8000;

pub fn default_server_origin() -> String {
    format!("http://127.0.0.1:{DEFAULT_SERVER_PORT}")
}
