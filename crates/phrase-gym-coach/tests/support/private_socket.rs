use super::*;

#[test]
fn production_listener_accepts_tokenless_loopback_and_refuses_other_local_addresses() {
    let (_directory, binary, data) = setup();
    let (_server, url) = start(&binary, &data, "0");
    let response = reqwest::blocking::Client::builder()
        .no_proxy()
        .build()
        .unwrap()
        .get(format!("{url}/api/v1/settings"))
        .send()
        .unwrap();
    assert!(response.status().is_success());
    assert!(!response.headers().contains_key("set-cookie"));
    let port = url.rsplit(':').next().unwrap();
    let other = format!("127.0.0.2:{port}").parse().unwrap();
    assert!(
        std::net::TcpStream::connect_timeout(&other, std::time::Duration::from_secs(1)).is_err()
    );
}
