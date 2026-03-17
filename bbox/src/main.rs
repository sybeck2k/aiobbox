use std::collections::HashMap;

use clap::Parser;
use serde::Serialize;

use bbox::models::{Host, Router, WanIpStats};
use bbox::BboxApi;

#[derive(Parser)]
#[command(about = "Bouygues Bbox router CLI")]
struct Args {
    /// Output the full result as JSON instead of the formatted summary.
    #[arg(long)]
    json: bool,

    /// Output a flat JSON payload designed for Home Assistant command_line sensors.
    /// Fields: state, uptime_s, wan_rx_kbps, wan_tx_kbps, wan_rx_pct, wan_tx_pct,
    /// wan_rx_max_kbps, wan_tx_max_kbps, active_devices, total_devices, active_macs.
    #[arg(long)]
    ha_json: bool,
}

// ---------------------------------------------------------------------------
// Vendor lookup
// ---------------------------------------------------------------------------

/// Look up the OUI vendor for a MAC address string (e.g. "AA:BB:CC:DD:EE:FF").
/// Results are cached by the first three octets so repeated lookups for hosts
/// on the same manufacturer hit the HashMap instead of the database.
fn lookup_vendor(
    cache: &mut HashMap<String, Option<String>>,
    oui_db: &Option<mac_oui::Oui>,
    mac: &str,
) -> Option<String> {
    // Key is the uppercase OUI prefix: "AA:BB:CC"
    let prefix = mac.get(..8)?.to_ascii_uppercase();
    if let Some(cached) = cache.get(&prefix) {
        return cached.clone();
    }
    let vendor = oui_db
        .as_ref()
        .and_then(|db| db.lookup_by_mac(mac).ok().flatten().map(|e| e.company_name.clone()));
    cache.insert(prefix, vendor.clone());
    vendor
}

// ---------------------------------------------------------------------------
// HA output type
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct HaOutput {
    /// Sensor state value — always "online" when the binary succeeds.
    state: &'static str,
    /// Router uptime in seconds.
    uptime_s: u64,
    /// WAN download bandwidth in kbps (current).
    wan_rx_kbps: u64,
    /// WAN upload bandwidth in kbps (current).
    wan_tx_kbps: u64,
    /// WAN download occupation percentage (0–100).
    wan_rx_pct: u64,
    /// WAN upload occupation percentage (0–100).
    wan_tx_pct: u64,
    /// Maximum WAN download bandwidth in kbps (line capacity).
    wan_rx_max_kbps: u64,
    /// Maximum WAN upload bandwidth in kbps (line capacity).
    wan_tx_max_kbps: u64,
    /// Number of currently active devices.
    active_devices: usize,
    /// Total number of known devices (active + inactive).
    total_devices: usize,
    /// MAC addresses of currently active devices (use for presence detection).
    active_macs: Vec<String>,
}

// ---------------------------------------------------------------------------
// JSON output types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct HostWithVendor<'a> {
    #[serde(flatten)]
    host: &'a Host,
    vendor: Option<String>,
}

#[derive(Serialize)]
struct JsonOutput<'a> {
    router: &'a Router,
    hosts: Vec<HostWithVendor<'a>>,
    wan_ip_stats: &'a WanIpStats,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let password = std::env::var("BBOX_PASSWORD").unwrap_or_else(|_| {
        eprintln!("Error: BBOX_PASSWORD environment variable is not set.");
        std::process::exit(1);
    });

    let base_url = std::env::var("BBOX_BASE_URL")
        .unwrap_or_else(|_| BboxApi::DEFAULT_BASE_URL.to_owned());

    let mut api = BboxApi::with_config(&password, &base_url, BboxApi::DEFAULT_TIMEOUT)?;
    api.authenticate().await?;

    let router = api.get_router_info().await?;
    let hosts = api.get_hosts().await?;
    let wan = api.get_wan_ip_stats().await?;

    // Initialise the embedded OUI database. With the `with-db` feature the
    // database is bundled in the binary, so this never touches the network.
    // If initialisation fails for any reason all vendor fields are null.
    let oui_db: Option<mac_oui::Oui> = mac_oui::Oui::default().ok();
    let mut vendor_cache: HashMap<String, Option<String>> = HashMap::new();

    if args.ha_json {
        let active: Vec<_> = hosts.iter().filter(|h| h.active).collect();
        let output = HaOutput {
            state: "online",
            uptime_s: router.uptime,
            wan_rx_kbps: wan.rx.bandwidth,
            wan_tx_kbps: wan.tx.bandwidth,
            wan_rx_pct: wan.rx.occupation,
            wan_tx_pct: wan.tx.occupation,
            wan_rx_max_kbps: wan.rx.max_bandwidth,
            wan_tx_max_kbps: wan.tx.max_bandwidth,
            active_devices: active.len(),
            total_devices: hosts.len(),
            active_macs: active.iter().map(|h| h.macaddress.clone()).collect(),
        };
        println!("{}", serde_json::to_string(&output)?);
    } else if args.json {
        let hosts_with_vendor: Vec<HostWithVendor<'_>> = hosts
            .iter()
            .map(|h| HostWithVendor {
                vendor: lookup_vendor(&mut vendor_cache, &oui_db, &h.macaddress),
                host: h,
            })
            .collect();

        let output = JsonOutput {
            router: &router,
            hosts: hosts_with_vendor,
            wan_ip_stats: &wan,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        let active: Vec<_> = hosts.iter().filter(|h| h.active).collect();

        println!("=== Router ===");
        println!("  Model:    {}", router.modelname);
        println!("  Serial:   {}", router.serialnumber);
        println!("  Uptime:   {}s", router.uptime);

        println!("\n=== Hosts ({} active / {} total) ===", active.len(), hosts.len());
        for host in &active {
            let vendor = lookup_vendor(&mut vendor_cache, &oui_db, &host.macaddress);
            println!(
                "  {:17}  {:15}  {:10}  {:25}  {}",
                host.macaddress,
                host.ipaddress,
                host.link,
                host.hostname.as_deref().unwrap_or("-"),
                vendor.as_deref().unwrap_or("-"),
            );
        }

        println!("\n=== WAN IP Stats ===");
        println!("  RX bandwidth:   {} kbps  ({}% used)", wan.rx.bandwidth, wan.rx.occupation);
        println!("  TX bandwidth:   {} kbps  ({}% used)", wan.tx.bandwidth, wan.tx.occupation);
    }

    Ok(())
}
