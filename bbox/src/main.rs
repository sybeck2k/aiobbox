use std::time::Duration;

use bbox::{BboxApi, BboxError};
use tokio::time::sleep;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

const DEFAULT_INTERVAL_SECS: u64 = 30;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(
            "bbox=info".parse().expect("valid directive"),
        ))
        .init();

    let password = std::env::var("BBOX_PASSWORD").unwrap_or_else(|_| {
        eprintln!("Error: BBOX_PASSWORD environment variable is not set.");
        std::process::exit(1);
    });

    let base_url = std::env::var("BBOX_URL")
        .unwrap_or_else(|_| BboxApi::DEFAULT_BASE_URL.to_owned());

    let interval_secs: u64 = std::env::var("BBOX_INTERVAL")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_INTERVAL_SECS);

    info!(%base_url, interval_secs, "Starting bbox service");

    loop {
        match run_once(&password, &base_url).await {
            Ok(()) => {}
            Err(BboxError::InvalidCredentials(_)) => {
                error!("Invalid password – check BBOX_PASSWORD. Exiting.");
                std::process::exit(1);
            }
            Err(e) => {
                warn!("Poll failed: {e}. Retrying in {interval_secs}s.");
            }
        }
        sleep(Duration::from_secs(interval_secs)).await;
    }
}

async fn run_once(password: &str, base_url: &str) -> Result<(), BboxError> {
    let mut api = BboxApi::with_config(
        password,
        base_url,
        Duration::from_secs(10),
    )?;
    api.authenticate().await?;

    let router = api.get_router_info().await?;
    info!(
        model = %router.modelname,
        uptime_secs = router.uptime,
        serial = %router.serialnumber,
        "Router info"
    );

    let hosts = api.get_hosts().await?;
    let active: Vec<_> = hosts.iter().filter(|h| h.active).collect();
    info!(
        total = hosts.len(),
        active = active.len(),
        "Connected hosts"
    );
    for host in &active {
        info!(
            id = host.id,
            ip = %host.ipaddress,
            mac = %host.macaddress,
            link = %host.link,
            hostname = ?host.hostname,
            "  Host"
        );
    }

    let wan = api.get_wan_ip_stats().await?;
    info!(
        rx_bandwidth  = wan.rx.bandwidth,
        tx_bandwidth  = wan.tx.bandwidth,
        rx_occupation = wan.rx.occupation,
        tx_occupation = wan.tx.occupation,
        "WAN IP stats"
    );

    Ok(())
}
