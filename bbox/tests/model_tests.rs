use bbox::models::{Host, Router, WanIpStats, WanStats};
use serde_json::json;

// ---------------------------------------------------------------------------
// Shared fixtures (mirroring conftest.py)
// ---------------------------------------------------------------------------

fn sample_device_data() -> serde_json::Value {
    json!({
        "device": {
            "now": "2025-11-07T13:40:00+0100",
            "status": 1,
            "numberofboots": 42,
            "modelname": "TestRouter3000",
            "modelclass": "TR3000",
            "optimisation": 1,
            "user_configured": 1,
            "serialnumber": "123456789012345",
            "display": {
                "luminosity": 2,
                "luminosity_extender": 100,
                "state": "."
            },
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
    })
}

fn sample_inactive_host() -> serde_json::Value {
    json!({
        "id": 1,
        "active": 0,
        "hostname": "",
        "ipaddress": "192.168.1.100",
        "macaddress": "aa:bb:cc:dd:ee:01",
        "type": "Static",
        "link": "Wifi 5",
        "lease": 0,
        "firstseen": "2024-06-15T10:00:00+0200",
        "lastseen": 3131260,
        "devicetype": "Device",
        "duid": "",
        "guest": 0,
        "serialNumber": "",
        "ip6address": [],
        "ethernet": { "physicalport": 0, "logicalport": 0, "speed": 0, "mode": "" },
        "wireless": {
            "wexindex": 0, "static": 0, "band": "", "txUsage": 0, "rxUsage": 0,
            "estimatedRate": 0, "rssi0": 0, "mcs": 0, "rate": 0
        },
        "wirelessByBand": [],
        "plc": { "rxphyrate": "", "txphyrate": "", "associateddevice": 0, "interface": 0, "ethernetspeed": 0 },
        "informations": {
            "type": "GÃ©nÃ©rique",
            "manufacturer": "GenericCorp",
            "model": "Device",
            "icon": "generic",
            "operatingSystem": "Unknown OS",
            "version": ""
        },
        "parentalcontrol": { "enable": 0, "status": "Allowed", "statusRemaining": 0, "statusUntil": "" },
        "ping": { "average": 0 },
        "scan": { "services": [] }
    })
}

fn sample_active_dhcp_host() -> serde_json::Value {
    json!({
        "id": 5,
        "active": 1,
        "hostname": "office-printer",
        "ipaddress": "192.168.1.110",
        "macaddress": "aa:bb:cc:dd:ee:05",
        "type": "DHCP",
        "link": "Wifi 2.4",
        "lease": 67189,
        "firstseen": "2024-06-01T12:30:00+0200",
        "lastseen": 0,
        "devicetype": "Device",
        "duid": "02:00:00:00:00:03:00:01:aa:bb:cc:dd:ee:05",
        "guest": 0,
        "serialNumber": "",
        "ip6address": [
            { "ipaddress": "fe80::aabb:ccff:fedd:ee05", "status": "Preferred",
              "lastseen": "2025-11-07T07:17:15+0100", "lastscan": "2025-10-31T10:16:40+0100" },
            { "ipaddress": "2001:db8:85a3::8a2e:370:7334", "status": "Preferred",
              "lastseen": "2025-11-01T20:13:42+0100", "lastscan": "1970-01-16T07:15:14+0100" }
        ],
        "ethernet": { "physicalport": 10, "logicalport": 11, "speed": 0, "mode": "" },
        "wireless": {
            "wexindex": 0, "static": 0, "band": "2.4", "txUsage": 0, "rxUsage": 0,
            "estimatedRate": 30, "rssi0": "-52", "mcs": 7, "rate": 72
        },
        "wirelessByBand": [
            { "band": "2.4", "txUsage": 0, "rxUsage": 0, "estimatedRate": 30,
              "rssi0": "-52", "mcs": 7, "rate": 72 }
        ],
        "plc": { "rxphyrate": "", "txphyrate": "", "associateddevice": 0, "interface": 0, "ethernetspeed": 0 },
        "informations": {
            "type": "Printer", "manufacturer": "PrinterVendor", "model": "OfficeNet 5000",
            "icon": "printer", "operatingSystem": "Embedded OS", "version": ""
        },
        "parentalcontrol": { "enable": 0, "status": "Allowed", "statusRemaining": 0, "statusUntil": "" },
        "ping": { "average": 0 },
        "scan": { "services": [] }
    })
}

fn sample_ethernet_host() -> serde_json::Value {
    json!({
        "id": 10,
        "me": 1,
        "active": 1,
        "hostname": "home-server",
        "ipaddress": "192.168.1.50",
        "macaddress": "aa:bb:cc:dd:ee:10",
        "type": "DHCP",
        "link": "Ethernet",
        "lease": 61613,
        "firstseen": "2024-08-10T14:20:00+0200",
        "lastseen": 0,
        "devicetype": "Device",
        "duid": "e3:a1:b5:37:00:02:00:00:ab:11:6d:a8:bb:63:72:7d:94:e4",
        "guest": 0,
        "serialNumber": "",
        "ip6address": [
            { "ipaddress": "fe80::aabb:ccff:fedd:ee10", "status": "Preferred",
              "lastseen": "2025-11-07T13:39:38+0100", "lastscan": "2025-09-25T14:38:44+0200" },
            { "ipaddress": "2001:db8:85a3::8a2e:370:7335", "status": "Preferred",
              "lastseen": "2025-10-24T23:14:32+0200", "lastscan": "1970-01-16T07:15:14+0100" }
        ],
        "ethernet": { "physicalport": 2, "logicalport": 3, "speed": 2500, "mode": "Full" },
        "wireless": {
            "wexindex": 0, "static": 0, "band": "", "txUsage": 0, "rxUsage": 0,
            "estimatedRate": 0, "rssi0": 0, "mcs": 0, "rate": 0
        },
        "wirelessByBand": [],
        "plc": { "rxphyrate": "", "txphyrate": "", "associateddevice": 0, "interface": 0, "ethernetspeed": 0 },
        "informations": {
            "type": "Generic Device", "manufacturer": "ServerVendor", "model": "EdgeNode Pro",
            "icon": "generic", "operatingSystem": "Linux", "version": ""
        },
        "parentalcontrol": { "enable": 0, "status": "Allowed", "statusRemaining": 0, "statusUntil": "" },
        "ping": { "average": 0 },
        "scan": { "services": [] }
    })
}

fn sample_wan_stats_data() -> serde_json::Value {
    json!({
        "wan": { "ip": { "stats": {
            "tx": {
                "packets": 400000, "bytes": 2400000, "packetserrors": 5,
                "packetsdiscards": 2, "occupation": 25, "bandwidth": 250,
                "maxBandwidth": 1000, "contractualBandwidth": 500
            },
            "rx": {
                "packets": 500000, "bytes": 2500000, "packetserrors": 5,
                "packetsdiscards": 3, "occupation": 20, "bandwidth": 200,
                "maxBandwidth": 1000, "contractualBandwidth": 500
            }
        }}}
    })
}

// ---------------------------------------------------------------------------
// Router model tests
// ---------------------------------------------------------------------------

#[test]
fn test_router_creation() {
    let data = sample_device_data();
    let router: Router = serde_json::from_value(data["device"].clone()).unwrap();
    assert_eq!(router.modelname, "TestRouter3000");
    assert_eq!(router.serialnumber, "123456789012345");
    assert_eq!(router.numberofboots, 42);
    assert_eq!(router.uptime, 4043471);
    assert!(router.using.ftth);
    assert!(!router.using.adsl);
}

#[test]
fn test_router_display() {
    let data = sample_device_data();
    let router: Router = serde_json::from_value(data["device"].clone()).unwrap();
    assert_eq!(router.display.luminosity, 2);
    assert_eq!(router.display.luminosity_extender, 100);
    assert_eq!(router.display.state, ".");
}

#[test]
fn test_router_versions() {
    let data = sample_device_data();
    let router: Router = serde_json::from_value(data["device"].clone()).unwrap();
    assert_eq!(router.main.version.as_deref(), Some("1.0.0"));
    assert_eq!(router.running.version.as_deref(), Some("1.0.0"));
    // Empty string → None
    assert!(router.spl.version.is_none());
}

#[test]
fn test_router_empty_string_to_none() {
    let data = json!({
        "now": "2025-11-07T13:40:00+0100",
        "status": 1,
        "numberofboots": 10,
        "modelname": "Test",
        "modelclass": "Test",
        "optimisation": 1,
        "user_configured": 1,
        "serialnumber": "123",
        "display": { "luminosity": 0, "luminosity_extender": 0, "state": "." },
        "main": { "version": "" },
        "reco": { "version": "" },
        "running": { "version": "" },
        "spl": { "version": "" },
        "tpl": { "version": "" },
        "ldr1": { "version": "" },
        "ldr2": { "version": "" },
        "firstusedate": "2024-06-01T08:00:00Z",
        "uptime": 0,
        "lastFactoryReset": 0,
        "using": { "ipv4": 1, "ipv6": 1, "ftth": 1, "adsl": 0, "vdsl": 0 }
    });
    let router: Router = serde_json::from_value(data).unwrap();
    assert!(router.main.version.is_none());
}

#[test]
fn test_router_missing_required_field() {
    let data = json!({
        "now": "2025-11-07T13:40:00+0100",
        "status": 1,
        "numberofboots": 10,
        "modelname": "Test",
        "modelclass": "Test",
        "optimisation": 1,
        "user_configured": 1,
        "serialnumber": "123",
        "display": { "luminosity": 0, "luminosity_extender": 0, "state": "." },
        // Missing: main, reco, running, spl, tpl, ldr1, ldr2
        "uptime": 0,
        "lastFactoryReset": 0,
        "using": { "ipv4": 1, "ipv6": 1, "ftth": 1, "adsl": 0, "vdsl": 0 }
    });
    assert!(serde_json::from_value::<Router>(data).is_err());
}

// ---------------------------------------------------------------------------
// Host model tests
// ---------------------------------------------------------------------------

#[test]
fn test_inactive_host() {
    let host: Host = serde_json::from_value(sample_inactive_host()).unwrap();
    assert_eq!(host.id, 1);
    assert!(!host.active);
    assert_eq!(host.link, "Wifi 5");
    assert!(host.hostname.is_none()); // empty string → None
}

#[test]
fn test_active_dhcp_host() {
    let host: Host = serde_json::from_value(sample_active_dhcp_host()).unwrap();
    assert_eq!(host.id, 5);
    assert!(host.active);
    assert_eq!(host.link, "Wifi 2.4");
    assert_eq!(host.hostname.as_deref(), Some("office-printer"));
}

#[test]
fn test_ethernet_host() {
    let host: Host = serde_json::from_value(sample_ethernet_host()).unwrap();
    assert_eq!(host.id, 10);
    assert!(host.active);
    assert_eq!(host.link, "Ethernet");
    assert_eq!(host.hostname.as_deref(), Some("home-server"));
    assert!(host.me); // me: 1 → true
}

#[test]
fn test_host_ipv6_addresses() {
    let host: Host = serde_json::from_value(sample_active_dhcp_host()).unwrap();
    assert_eq!(host.link_type, "DHCP");
    assert_eq!(host.ip6address.len(), 2);
    assert_eq!(host.ip6address[0].ipaddress, "fe80::aabb:ccff:fedd:ee05");
}

#[test]
fn test_host_wireless_band_and_rssi() {
    let host: Host = serde_json::from_value(sample_active_dhcp_host()).unwrap();
    let wireless = host.wireless.as_ref().unwrap();
    assert_eq!(wireless.band, Some(2.4));
    assert_eq!(wireless.rssi0, -52); // "-52" string → i64
}

#[test]
fn test_host_ethernet_speed() {
    let host: Host = serde_json::from_value(sample_ethernet_host()).unwrap();
    let eth = host.ethernet.as_ref().unwrap();
    assert_eq!(eth.speed, 2500);
    assert_eq!(eth.mode.as_deref(), Some("Full"));
}

#[test]
fn test_host_wireless_by_band() {
    let host: Host = serde_json::from_value(sample_active_dhcp_host()).unwrap();
    assert_eq!(host.wireless_by_band.len(), 1);
    assert_eq!(host.wireless_by_band[0].band, 2.4);
    assert_eq!(host.wireless_by_band[0].rate, 72);
}

#[test]
fn test_host_mojibake_fix() {
    // "GÃ©nÃ©rique" is "Générique" mis-decoded as Latin-1 when it should be UTF-8.
    let host: Host = serde_json::from_value(sample_inactive_host()).unwrap();
    let info = host.informations.as_ref().unwrap();
    assert_eq!(info.device_type, "Générique");
}

#[test]
fn test_host_defaults() {
    let host: Host = serde_json::from_value(sample_inactive_host()).unwrap();
    assert!(!host.me); // absent → default false
    assert!(!host.guest);
}

// ---------------------------------------------------------------------------
// WAN stats model tests
// ---------------------------------------------------------------------------

#[test]
fn test_wan_stats_creation() {
    let data = sample_wan_stats_data();
    let stats: WanIpStats =
        serde_json::from_value(data["wan"]["ip"]["stats"].clone()).unwrap();
    assert_eq!(stats.tx.packets, 400000);
    assert_eq!(stats.rx.packets, 500000);
}

#[test]
fn test_wan_stats_fields() {
    let data = json!({
        "packets": 1000, "bytes": 500000, "packetserrors": 0,
        "packetsdiscards": 0, "occupation": 50, "bandwidth": 100,
        "maxBandwidth": 1000, "contractualBandwidth": 500
    });
    let stats: WanStats = serde_json::from_value(data).unwrap();
    assert_eq!(stats.packets, 1000);
    assert_eq!(stats.occupation, 50);
}

#[test]
fn test_wan_stats_missing_field() {
    // Missing most fields – should fail.
    let data = json!({ "packets": 1000, "occupation": 50 });
    assert!(serde_json::from_value::<WanStats>(data).is_err());
}
