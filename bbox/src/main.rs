use bbox::BboxApi;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let password = std::env::var("BBOX_PASSWORD").unwrap_or_else(|_| {
        eprintln!("Error: BBOX_PASSWORD environment variable is not set.");
        std::process::exit(1);
    });

    let mut api = BboxApi::new(&password)?;
    api.authenticate().await?;

    let router = api.get_router_info().await?;
    println!("=== Router ===");
    println!("  Model:    {}", router.modelname);
    println!("  Serial:   {}", router.serialnumber);
    println!("  Uptime:   {}s", router.uptime);

    let hosts = api.get_hosts().await?;
    let active: Vec<_> = hosts.iter().filter(|h| h.active).collect();
    println!("\n=== Hosts ({} active / {} total) ===", active.len(), hosts.len());
    for host in &active {
        println!(
            "  {:17}  {:15}  {}  {}",
            host.macaddress,
            host.ipaddress,
            host.link,
            host.hostname.as_deref().unwrap_or("-"),
        );
    }

    let wan = api.get_wan_ip_stats().await?;
    println!("\n=== WAN IP Stats ===");
    println!("  RX bandwidth:   {} kbps  ({}% used)", wan.rx.bandwidth, wan.rx.occupation);
    println!("  TX bandwidth:   {} kbps  ({}% used)", wan.tx.bandwidth, wan.tx.occupation);

    Ok(())
}
