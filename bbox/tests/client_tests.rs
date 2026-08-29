use bbox::{BboxApi, BboxError};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn mock_server_with(
    http_method: &str,
    uri_path: &str,
    response: ResponseTemplate,
) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method(http_method))
        .and(path(uri_path))
        .respond_with(response)
        .mount(&server)
        .await;
    server
}

fn device_response() -> serde_json::Value {
    json!([{
        "device": {
            "now": "2025-11-07T13:40:00+0100",
            "status": 1,
            "numberofboots": 42,
            "modelname": "TestRouter3000",
            "modelclass": "TR3000",
            "optimisation": 1,
            "user_configured": 1,
            "serialnumber": "123456789012345",
            "display": { "luminosity": 2, "luminosity_extender": 100, "state": "." },
            "main": { "version": "1.0.0", "date": "2025-01-15T10:00:00Z" },
            "reco": { "version": "1.0.0", "date": "2025-01-15T09:30:00Z" },
            "running": { "version": "1.0.0", "date": "2025-01-15T09:45:00Z" },
            "spl": { "version": "" },
            "tpl": { "version": "" },
            "ldr1": { "version": "2.1.0" },
            "ldr2": { "version": "2.1.0" },
            "firstusedate": "2024-06-01T08:00:00Z",
            "uptime": 4043471,
            "lastFactoryReset": 0,
            "using": { "ipv4": 1, "ipv6": 1, "ftth": 1, "adsl": 0, "vdsl": 0 },
            "isCertified": 1
        }
    }])
}

fn hosts_response() -> serde_json::Value {
    json!([{
        "hosts": { "list": [
            {
                "id": 10, "me": 1, "active": 1, "hostname": "home-server",
                "ipaddress": "192.168.1.50", "macaddress": "aa:bb:cc:dd:ee:10",
                "type": "DHCP", "link": "Ethernet", "lease": 61613,
                "firstseen": "2024-08-10T14:20:00+0200", "lastseen": 0,
                "devicetype": "Device", "duid": "", "guest": 0, "serialNumber": "",
                "ip6address": [], "ethernet": { "physicalport": 2, "logicalport": 3, "speed": 2500, "mode": "Full" },
                "wireless": { "wexindex": 0, "static": 0, "band": "", "txUsage": 0, "rxUsage": 0, "estimatedRate": 0, "rssi0": 0, "mcs": 0, "rate": 0 },
                "wirelessByBand": [],
                "plc": { "rxphyrate": "", "txphyrate": "", "associateddevice": 0, "interface": 0, "ethernetspeed": 0 },
                "informations": { "type": "Server", "manufacturer": "Acme", "model": "Box", "icon": "generic", "operatingSystem": "Linux", "version": "" },
                "parentalcontrol": { "enable": 0, "status": "Allowed", "statusRemaining": 0, "statusUntil": "" },
                "ping": { "average": 0 }, "scan": { "services": [] }
            }
        ]}
    }])
}

fn wan_stats_response() -> serde_json::Value {
    json!([{
        "wan": { "ip": { "stats": {
            "tx": { "packets": 400000, "bytes": 2400000, "packetserrors": 5, "packetsdiscards": 2, "occupation": 25, "bandwidth": 250, "maxBandwidth": 1000, "contractualBandwidth": 500 },
            "rx": { "packets": 500000, "bytes": 2500000, "packetserrors": 5, "packetsdiscards": 3, "occupation": 20, "bandwidth": 200, "maxBandwidth": 1000, "contractualBandwidth": 500 }
        }}}
    }])
}

async fn authenticated_client(server: &MockServer) -> BboxApi {
    Mock::given(method("POST"))
        .and(path("/api/v1/login"))
        .respond_with(ResponseTemplate::new(200))
        .mount(server)
        .await;

    let mut api = BboxApi::with_config(
        "test_password",
        format!("{}/api/v1/", server.uri()),
        std::time::Duration::from_secs(5),
    )
    .unwrap();
    api.authenticate().await.unwrap();
    api
}

// ---------------------------------------------------------------------------
// Initialisation tests
// ---------------------------------------------------------------------------

#[test]
fn test_new_empty_password_fails() {
    assert!(BboxApi::new("").is_err());
}

#[test]
fn test_new_valid_password() {
    assert!(BboxApi::new("secret").is_ok());
}

// ---------------------------------------------------------------------------
// Authentication tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_successful_authentication() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/login"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let mut api = BboxApi::with_config(
        "test_password",
        format!("{}/api/v1/", server.uri()),
        std::time::Duration::from_secs(5),
    )
    .unwrap();

    api.authenticate().await.unwrap();
    // Second call should be a no-op (already authenticated)
    api.authenticate().await.unwrap();
}

#[tokio::test]
async fn test_authentication_invalid_credentials() {
    let server = mock_server_with(
        "POST",
        "/api/v1/login",
        ResponseTemplate::new(401).set_body_json(json!({
            "exception": { "errors": [{ "reason": "wrong password" }] }
        })),
    )
    .await;

    let mut api = BboxApi::with_config(
        "wrong",
        format!("{}/api/v1/", server.uri()),
        std::time::Duration::from_secs(5),
    )
    .unwrap();

    let err = api.authenticate().await.unwrap_err();
    assert!(matches!(err, BboxError::InvalidCredentials(_)));
    assert!(err.to_string().contains("Invalid password"));
}

#[tokio::test]
async fn test_authentication_rate_limit() {
    let server = mock_server_with(
        "POST",
        "/api/v1/login",
        ResponseTemplate::new(429).set_body_json(json!({
            "exception": { "errors": [{ "reason": "too many attempts" }] }
        })),
    )
    .await;

    let mut api = BboxApi::with_config(
        "test",
        format!("{}/api/v1/", server.uri()),
        std::time::Duration::from_secs(5),
    )
    .unwrap();

    let err = api.authenticate().await.unwrap_err();
    assert!(matches!(err, BboxError::RateLimit(_)));
}

#[tokio::test]
async fn test_authentication_server_error() {
    let server = mock_server_with(
        "POST",
        "/api/v1/login",
        ResponseTemplate::new(500),
    )
    .await;

    let mut api = BboxApi::with_config(
        "test",
        format!("{}/api/v1/", server.uri()),
        std::time::Duration::from_secs(5),
    )
    .unwrap();

    let err = api.authenticate().await.unwrap_err();
    assert!(matches!(err, BboxError::Api { .. }));
}

// ---------------------------------------------------------------------------
// Unauthenticated request test
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_request_unauthenticated() {
    let api = BboxApi::new("test").unwrap();
    // Can't call get_router_info on unauthenticated client
    // We need a server but the request should fail before hitting it
    let server = MockServer::start().await;
    let mut api = BboxApi::with_config(
        "test",
        format!("{}/api/v1/", server.uri()),
        std::time::Duration::from_secs(5),
    )
    .unwrap();
    let err = api.get_router_info().await.unwrap_err();
    assert!(matches!(err, BboxError::Unauthenticated(_)));
    assert!(err.to_string().contains("Not authenticated"));
}

// ---------------------------------------------------------------------------
// API method tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_router_info() {
    let server = MockServer::start().await;
    let mut api = authenticated_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/device"))
        .respond_with(ResponseTemplate::new(200).set_body_json(device_response()))
        .mount(&server)
        .await;

    let router = api.get_router_info().await.unwrap();
    assert_eq!(router.modelname, "TestRouter3000");
    assert_eq!(router.serialnumber, "123456789012345");
    assert_eq!(router.numberofboots, 42);
}

#[tokio::test]
async fn test_get_hosts() {
    let server = MockServer::start().await;
    let mut api = authenticated_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/hosts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(hosts_response()))
        .mount(&server)
        .await;

    let hosts = api.get_hosts().await.unwrap();
    assert_eq!(hosts.len(), 1);
    assert_eq!(hosts[0].hostname.as_deref(), Some("home-server"));
    assert!(hosts[0].me);
}

#[tokio::test]
async fn test_get_wan_ip_stats() {
    let server = MockServer::start().await;
    let mut api = authenticated_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/wan/ip/stats"))
        .respond_with(ResponseTemplate::new(200).set_body_json(wan_stats_response()))
        .mount(&server)
        .await;

    let stats = api.get_wan_ip_stats().await.unwrap();
    assert_eq!(stats.tx.bytes, 2400000);
    assert_eq!(stats.rx.bytes, 2500000);
}

#[tokio::test]
async fn test_get_device_power() {
    use bbox::models::PowerPeriod;

    let server = MockServer::start().await;
    let mut api = authenticated_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/graphs/device/power/week"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "type": 0,
            "rate": 900,
            "data": [[900, 14679], [900, 14760], [900, 14863]],
            "last": 5400
        })))
        .mount(&server)
        .await;

    let graph = api.get_device_power(PowerPeriod::Week).await.unwrap();
    assert_eq!(graph.rate, 900);
    assert_eq!(graph.last, 5400);
    assert_eq!(graph.samples.len(), 3);
    assert_eq!(graph.samples[0].interval_s, 900);
    assert_eq!(graph.latest().unwrap().value, 14863);
}

#[tokio::test]
async fn test_session_expired() {
    let server = MockServer::start().await;
    let mut api = authenticated_client(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/device"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;

    let err = api.get_router_info().await.unwrap_err();
    assert!(matches!(err, BboxError::SessionExpired(_)));
}
