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

#[test]
fn direct_connection_enforces_host_and_csrf_without_login_or_cookies() {
    let (_directory, binary, data) = setup();
    let (_server, url) = start(&binary, &data, "0");
    let http = reqwest::blocking::Client::builder()
        .no_proxy()
        .build()
        .unwrap();
    for (host, origin, fetch_site, expected) in [
        ("remote.example", None, None, 400),
        ("127.0.0.1", Some("https://remote.example"), None, 403),
        ("127.0.0.1", Some("null"), None, 403),
        ("127.0.0.1", None, Some("cross-site"), 403),
    ] {
        let mut request = http
            .post(format!("{url}/api/v1/preparation/claims"))
            .header("host", host)
            .json(&ClaimRequest {
                claim_id: Uuid::new_v4(),
            });
        if let Some(origin) = origin {
            request = request.header("origin", origin);
        }
        if let Some(site) = fetch_site {
            request = request.header("sec-fetch-site", site);
        }
        let response = request.send().unwrap();
        assert_eq!(response.status().as_u16(), expected);
        assert!(!response.headers().contains_key("set-cookie"));
        assert!(!response.headers().contains_key("www-authenticate"));
    }
    let client = ServerClient::new(&url).unwrap();
    let claim: Option<JobClaim> = client
        .post(
            "/api/v1/preparation/claims",
            &ClaimRequest {
                claim_id: Uuid::new_v4(),
            },
        )
        .unwrap();
    assert!(claim.is_none());
}
