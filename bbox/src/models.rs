use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use serde_json::Value;

// ---------------------------------------------------------------------------
// Serde helpers
// ---------------------------------------------------------------------------

mod de {
    use chrono::{DateTime, FixedOffset};
    use serde::{Deserialize, Deserializer};
    use serde_json::Value;

    /// Re-encode a string from Latin-1 bytes interpreted as UTF-8, fixing
    /// mojibake that the router API sometimes produces.
    pub fn fix_mojibake(s: &str) -> String {
        // Latin-1 maps every char to its byte value (U+0000–U+00FF).
        // If any char exceeds U+00FF the string cannot be Latin-1-encoded.
        if s.chars().any(|c| c as u32 > 0xFF) {
            return s.to_owned();
        }
        let bytes: Vec<u8> = s.chars().map(|c| c as u8).collect();
        String::from_utf8(bytes).unwrap_or_else(|_| s.to_owned())
    }

    /// Deserialize a bool that may arrive as JSON `true`/`false` or `0`/`1`.
    pub fn bool_from_int_or_bool<'de, D: Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
        match Value::deserialize(d)? {
            Value::Bool(b) => Ok(b),
            Value::Number(n) => Ok(n.as_i64().unwrap_or(0) != 0),
            other => Err(serde::de::Error::custom(format!(
                "expected bool or integer, got {other:?}"
            ))),
        }
    }

    /// Like `bool_from_int_or_bool` but with a hard-coded `false` default,
    /// used with `#[serde(default = "de::default_false")]`.
    pub fn default_false() -> bool {
        false
    }

    /// Deserialize a string, converting empty strings to `None`.
    pub fn opt_string_empty_as_none<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Option<String>, D::Error> {
        let opt = Option::<String>::deserialize(d)?;
        Ok(opt
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty()))
    }

    /// Deserialize a string, fix mojibake, and convert empty strings to `None`.
    pub fn opt_clean_string_empty_as_none<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Option<String>, D::Error> {
        let opt = Option::<String>::deserialize(d)?;
        Ok(opt
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .map(|s| fix_mojibake(&s)))
    }

    /// Deserialize a string and apply the mojibake fix.
    pub fn clean_string<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
        let s = String::deserialize(d)?;
        Ok(fix_mojibake(s.trim()))
    }

    /// Deserialize an f64 that may come as a JSON number or a numeric string
    /// (e.g., `"2.4"`, `5`, `6.0`).
    pub fn f64_from_str_or_num<'de, D: Deserializer<'de>>(d: D) -> Result<f64, D::Error> {
        match Value::deserialize(d)? {
            Value::String(s) => s
                .trim()
                .parse::<f64>()
                .map_err(serde::de::Error::custom),
            Value::Number(n) => n
                .as_f64()
                .ok_or_else(|| serde::de::Error::custom("invalid number for f64")),
            other => Err(serde::de::Error::custom(format!(
                "expected number or string for band, got {other:?}"
            ))),
        }
    }

    /// Deserialize an optional f64 from a number, numeric string, empty string,
    /// or null (e.g., the `band` field in `WirelessInfo`).
    pub fn opt_f64_from_str_or_num_empty_as_none<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Option<f64>, D::Error> {
        match Value::deserialize(d)? {
            Value::Null => Ok(None),
            Value::String(s) if s.trim().is_empty() => Ok(None),
            Value::String(s) => s
                .trim()
                .parse::<f64>()
                .map(Some)
                .map_err(serde::de::Error::custom),
            Value::Number(n) => Ok(n.as_f64()),
            other => Err(serde::de::Error::custom(format!(
                "expected number, string, or null for band, got {other:?}"
            ))),
        }
    }

    /// Deserialize an i64 from a JSON integer or numeric string (e.g., `rssi0`
    /// which can be `0` or `"-52"`).
    pub fn i64_from_str_or_num<'de, D: Deserializer<'de>>(d: D) -> Result<i64, D::Error> {
        match Value::deserialize(d)? {
            Value::String(s) => s
                .trim()
                .parse::<i64>()
                .map_err(serde::de::Error::custom),
            Value::Number(n) => n
                .as_i64()
                .ok_or_else(|| serde::de::Error::custom("invalid number for i64")),
            other => Err(serde::de::Error::custom(format!(
                "expected integer or string for rssi, got {other:?}"
            ))),
        }
    }

    /// Deserialize a `DateTime<FixedOffset>` from an ISO 8601 string.
    /// Handles both RFC 3339 (`+01:00`) and the compact offset format (`+0100`)
    /// that the Bbox API emits.
    pub fn datetime<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<DateTime<FixedOffset>, D::Error> {
        let s = String::deserialize(d)?;
        // RFC 3339 / ISO 8601 with colon in offset (e.g., "2025-11-07T13:40:00+01:00" or "Z")
        if let Ok(dt) = DateTime::parse_from_rfc3339(&s) {
            return Ok(dt);
        }
        // Compact offset without colon (e.g., "+0100")
        DateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%z")
            .map_err(serde::de::Error::custom)
    }

    /// Deserialize an optional `DateTime<FixedOffset>`, treating null and empty
    /// strings as `None`.
    pub fn opt_datetime_empty_as_none<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Option<DateTime<FixedOffset>>, D::Error> {
        let opt = Option::<String>::deserialize(d)?;
        match opt.as_deref() {
            None | Some("") => Ok(None),
            Some(s) => {
                if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
                    return Ok(Some(dt));
                }
                DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%z")
                    .map(Some)
                    .map_err(serde::de::Error::custom)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// /device
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct RouterDisplay {
    pub luminosity: u32,
    pub luminosity_extender: u32,
    pub state: String,
}

#[derive(Debug, Deserialize)]
pub struct RouterVersion {
    #[serde(default, deserialize_with = "de::opt_string_empty_as_none")]
    pub version: Option<String>,
    #[serde(default, deserialize_with = "de::opt_datetime_empty_as_none")]
    pub date: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Deserialize)]
pub struct RouterUsing {
    #[serde(deserialize_with = "de::bool_from_int_or_bool")]
    pub ipv4: bool,
    #[serde(deserialize_with = "de::bool_from_int_or_bool")]
    pub ipv6: bool,
    #[serde(deserialize_with = "de::bool_from_int_or_bool")]
    pub ftth: bool,
    #[serde(deserialize_with = "de::bool_from_int_or_bool")]
    pub adsl: bool,
    #[serde(deserialize_with = "de::bool_from_int_or_bool")]
    pub vdsl: bool,
}

#[derive(Debug, Deserialize)]
pub struct Router {
    #[serde(deserialize_with = "de::datetime")]
    pub now: DateTime<FixedOffset>,
    pub status: i64,
    pub numberofboots: u64,
    #[serde(deserialize_with = "de::clean_string")]
    pub modelname: String,
    pub modelclass: String,
    #[serde(deserialize_with = "de::bool_from_int_or_bool")]
    pub optimisation: bool,
    #[serde(deserialize_with = "de::bool_from_int_or_bool")]
    pub user_configured: bool,
    pub serialnumber: String,
    pub display: RouterDisplay,
    pub main: RouterVersion,
    pub reco: RouterVersion,
    pub running: RouterVersion,
    pub spl: RouterVersion,
    pub tpl: RouterVersion,
    pub ldr1: RouterVersion,
    pub ldr2: RouterVersion,
    #[serde(deserialize_with = "de::datetime")]
    pub firstusedate: DateTime<FixedOffset>,
    pub uptime: u64,
    #[serde(rename = "lastFactoryReset")]
    pub last_factory_reset: i64,
    pub using: RouterUsing,
}

// ---------------------------------------------------------------------------
// /hosts
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct IPv6Address {
    pub ipaddress: String,
    pub status: String,
    #[serde(deserialize_with = "de::datetime")]
    pub lastseen: DateTime<FixedOffset>,
    #[serde(deserialize_with = "de::datetime")]
    pub lastscan: DateTime<FixedOffset>,
}

#[derive(Debug, Deserialize)]
pub struct EthernetInfo {
    pub physicalport: i64,
    pub logicalport: i64,
    pub speed: u64,
    #[serde(deserialize_with = "de::opt_string_empty_as_none")]
    pub mode: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WirelessInfo {
    pub wexindex: i64,
    #[serde(rename = "static", deserialize_with = "de::bool_from_int_or_bool")]
    pub static_: bool,
    /// Frequency band in GHz (2.4, 5, 6). `None` when not applicable.
    #[serde(
        rename = "band",
        deserialize_with = "de::opt_f64_from_str_or_num_empty_as_none"
    )]
    pub band: Option<f64>,
    #[serde(rename = "txUsage")]
    pub tx_usage: u64,
    #[serde(rename = "rxUsage")]
    pub rx_usage: u64,
    #[serde(rename = "estimatedRate")]
    pub estimated_rate: u64,
    /// RSSI signal strength. Arrives as integer or numeric string (e.g. `"-52"`).
    #[serde(deserialize_with = "de::i64_from_str_or_num")]
    pub rssi0: i64,
    pub mcs: u64,
    pub rate: u64,
}

#[derive(Debug, Deserialize)]
pub struct WirelessByBand {
    /// Frequency band in GHz. Arrives as number or numeric string (e.g. `"2.4"`).
    #[serde(deserialize_with = "de::f64_from_str_or_num")]
    pub band: f64,
    #[serde(rename = "txUsage")]
    pub tx_usage: u64,
    #[serde(rename = "rxUsage")]
    pub rx_usage: u64,
    #[serde(rename = "estimatedRate")]
    pub estimated_rate: u64,
    #[serde(deserialize_with = "de::i64_from_str_or_num")]
    pub rssi0: i64,
    pub mcs: u64,
    pub rate: u64,
}

#[derive(Debug, Deserialize)]
pub struct PlcInfo {
    #[serde(deserialize_with = "de::opt_string_empty_as_none")]
    pub rxphyrate: Option<String>,
    #[serde(deserialize_with = "de::opt_string_empty_as_none")]
    pub txphyrate: Option<String>,
    pub associateddevice: i64,
    pub interface: i64,
    pub ethernetspeed: u64,
}

#[derive(Debug, Deserialize)]
pub struct DeviceInformation {
    #[serde(rename = "type", deserialize_with = "de::clean_string")]
    pub device_type: String,
    #[serde(deserialize_with = "de::opt_clean_string_empty_as_none")]
    pub manufacturer: Option<String>,
    #[serde(deserialize_with = "de::opt_clean_string_empty_as_none")]
    pub model: Option<String>,
    pub icon: String,
    #[serde(rename = "operatingSystem", deserialize_with = "de::opt_clean_string_empty_as_none")]
    pub operating_system: Option<String>,
    #[serde(deserialize_with = "de::opt_clean_string_empty_as_none")]
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ParentalControl {
    #[serde(deserialize_with = "de::bool_from_int_or_bool")]
    pub enable: bool,
    pub status: String,
    #[serde(rename = "statusRemaining")]
    pub status_remaining: u64,
    #[serde(rename = "statusUntil", deserialize_with = "de::opt_datetime_empty_as_none")]
    pub status_until: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Deserialize)]
pub struct PingInfo {
    /// Average ping time in ms. Arrives as integer or float.
    pub average: f64,
}

#[derive(Debug, Deserialize)]
pub struct ScanInfo {
    #[serde(default)]
    pub services: Vec<Value>,
}

#[derive(Debug, Deserialize)]
pub struct Host {
    pub id: i64,
    #[serde(deserialize_with = "de::bool_from_int_or_bool")]
    pub active: bool,
    #[serde(deserialize_with = "de::opt_clean_string_empty_as_none")]
    pub hostname: Option<String>,
    pub ipaddress: String,
    pub macaddress: String,
    #[serde(rename = "type")]
    pub link_type: String,
    pub link: String,
    pub lease: i64,
    #[serde(deserialize_with = "de::datetime")]
    pub firstseen: DateTime<FixedOffset>,
    pub lastseen: i64,
    #[serde(deserialize_with = "de::opt_string_empty_as_none")]
    pub devicetype: Option<String>,
    #[serde(deserialize_with = "de::opt_string_empty_as_none")]
    pub duid: Option<String>,
    #[serde(default = "de::default_false", deserialize_with = "de::bool_from_int_or_bool")]
    pub guest: bool,
    #[serde(rename = "serialNumber", deserialize_with = "de::opt_string_empty_as_none")]
    pub serial_number: Option<String>,
    #[serde(default, rename = "ip6address")]
    pub ip6address: Vec<IPv6Address>,
    #[serde(default)]
    pub ethernet: Option<EthernetInfo>,
    #[serde(default)]
    pub wireless: Option<WirelessInfo>,
    #[serde(default, rename = "wirelessByBand")]
    pub wireless_by_band: Vec<WirelessByBand>,
    #[serde(default)]
    pub plc: Option<PlcInfo>,
    #[serde(default)]
    pub informations: Option<DeviceInformation>,
    #[serde(default)]
    pub parentalcontrol: Option<ParentalControl>,
    #[serde(default)]
    pub ping: Option<PingInfo>,
    #[serde(default)]
    pub scan: Option<ScanInfo>,
    #[serde(default = "de::default_false", deserialize_with = "de::bool_from_int_or_bool")]
    pub me: bool,
}

// ---------------------------------------------------------------------------
// /wan/ip/stats
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct WanStats {
    pub packets: u64,
    pub bytes: u64,
    pub packetserrors: u64,
    pub packetsdiscards: u64,
    /// Bandwidth occupation percentage (0–100).
    pub occupation: u8,
    pub bandwidth: u64,
    #[serde(rename = "maxBandwidth")]
    pub max_bandwidth: u64,
    #[serde(rename = "contractualBandwidth")]
    pub contractual_bandwidth: u64,
}

#[derive(Debug, Deserialize)]
pub struct WanIpStats {
    pub rx: WanStats,
    pub tx: WanStats,
}
