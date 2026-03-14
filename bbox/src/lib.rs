//! Async Rust API client for Bouygues Telecom Bbox routers.
//!
//! Tested with model F@st5696b, firmware version 25.5.28.
//!
//! # Quick start
//! ```no_run
//! #[tokio::main]
//! async fn main() -> Result<(), bbox::BboxError> {
//!     let mut api = bbox::BboxApi::connect("your_password").await?;
//!
//!     let router = api.get_router_info().await?;
//!     println!("Router model: {}", router.modelname);
//!
//!     let hosts = api.get_hosts().await?;
//!     println!("Connected hosts: {}", hosts.len());
//!
//!     let wan = api.get_wan_ip_stats().await?;
//!     println!("Download: {} Mbps", wan.rx.bandwidth);
//!     println!("Upload: {} Mbps", wan.tx.bandwidth);
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod error;
pub mod models;

pub use client::BboxApi;
pub use error::{BboxError, Result};
pub use models::{
    DeviceInformation, EthernetInfo, Host, IPv6Address, ParentalControl, PingInfo, PlcInfo,
    Router, RouterDisplay, RouterUsing, RouterVersion, ScanInfo, WanIpStats, WanStats,
    WirelessByBand, WirelessInfo,
};
