use std::time::Duration;

use reqwest::{Client, ClientBuilder, StatusCode};
use serde_json::Value;
use tracing::{debug, error, info, warn};

use crate::error::{BboxError, Result};
use crate::models::{Host, Router, WanIpStats};

/// Async Rust API client for Bouygues Telecom Bbox routers.
///
/// # Example
/// ```no_run
/// use bbox::BboxApi;
///
/// #[tokio::main]
/// async fn main() -> Result<(), bbox::BboxError> {
///     let mut api = BboxApi::connect("your_password").await?;
///     let router = api.get_router_info().await?;
///     println!("Model: {}", router.modelname);
///     Ok(())
/// }
/// ```
pub struct BboxApi {
    password: String,
    base_url: String,
    timeout: Duration,
    client: Client,
    authenticated: bool,
}

impl BboxApi {
    pub const DEFAULT_BASE_URL: &'static str = "https://mabbox.bytel.fr/api/v1/";
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

    /// Create a new (unauthenticated) client with default settings.
    ///
    /// Call [`authenticate`](BboxApi::authenticate) or use
    /// [`connect`](BboxApi::connect) to authenticate before making requests.
    pub fn new(password: impl Into<String>) -> Result<Self> {
        Self::with_config(password, Self::DEFAULT_BASE_URL, Self::DEFAULT_TIMEOUT)
    }

    /// Create a new client with custom base URL and timeout.
    pub fn with_config(
        password: impl Into<String>,
        base_url: impl Into<String>,
        timeout: Duration,
    ) -> Result<Self> {
        let password = password.into();
        if password.is_empty() {
            return Err(BboxError::Api {
                message: "Password cannot be empty".to_owned(),
                status_code: None,
            });
        }

        let mut base_url = base_url.into();
        if !base_url.ends_with('/') {
            base_url.push('/');
        }

        let client = ClientBuilder::new()
            .cookie_store(true)
            .build()
            .map_err(BboxError::Http)?;

        debug!(%base_url, "Initialized BboxApi client");

        Ok(Self {
            password,
            base_url,
            timeout,
            client,
            authenticated: false,
        })
    }

    /// Create a client and immediately authenticate.
    pub async fn connect(password: impl Into<String>) -> Result<Self> {
        let mut api = Self::new(password)?;
        api.authenticate().await?;
        Ok(api)
    }

    /// Authenticate against the Bbox login endpoint.
    ///
    /// Idempotent: if already authenticated this is a no-op.
    pub async fn authenticate(&mut self) -> Result<()> {
        if self.authenticated {
            debug!("Already authenticated, skipping");
            return Ok(());
        }

        let url = format!("{}login", self.base_url);
        let origin = self.base_url.trim_end_matches('/').to_owned();

        let response = self
            .client
            .post(&url)
            .header("Referer", &url)
            .header("Origin", &origin)
            .form(&[("password", &self.password), ("remember", &"1".to_owned())])
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    error!("Authentication timeout");
                    BboxError::Timeout("Authentication timeout".to_owned())
                } else {
                    BboxError::Http(e)
                }
            })?;

        let status = response.status();
        if status == StatusCode::OK {
            self.authenticated = true;
            info!("Successfully authenticated to Bbox");
            return Ok(());
        }

        // Parse optional error reason from the response body.
        let reason = response
            .json::<Value>()
            .await
            .ok()
            .and_then(|v| {
                v["exception"]["errors"][0]["reason"]
                    .as_str()
                    .map(str::to_owned)
            });

        match status.as_u16() {
            401 => {
                error!("Invalid credentials");
                Err(BboxError::InvalidCredentials("Invalid password".to_owned()))
            }
            429 => {
                let msg = format!(
                    "Rate limit exceeded: {}",
                    reason.as_deref().unwrap_or("too many login attempts")
                );
                error!("{msg}");
                Err(BboxError::RateLimit(msg))
            }
            code => {
                let mut msg = format!("Authentication failed with status {code}");
                if let Some(r) = reason {
                    msg.push_str(": ");
                    msg.push_str(&r);
                }
                error!("{msg}");
                Err(BboxError::Api {
                    message: msg,
                    status_code: Some(code),
                })
            }
        }
    }

    /// Make an authenticated GET request and return the parsed JSON value.
    ///
    /// Single-element arrays are automatically unwrapped (the real Bbox API
    /// wraps most responses in `[{...}]`).
    async fn request(&mut self, endpoint: &str) -> Result<Value> {
        if !self.authenticated {
            return Err(BboxError::Unauthenticated(
                "Not authenticated. Call authenticate() first.".to_owned(),
            ));
        }

        let url = format!("{}{}", self.base_url, endpoint);
        let origin = self.base_url.trim_end_matches('/').to_owned();

        let response = self
            .client
            .get(&url)
            .header("Referer", &self.base_url)
            .header("Origin", &origin)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    error!(%endpoint, "Request timeout");
                    BboxError::Timeout(format!("Request timeout for {endpoint}"))
                } else {
                    BboxError::Http(e)
                }
            })?;

        let status = response.status();

        if status == StatusCode::UNAUTHORIZED {
            self.authenticated = false;
            warn!(%endpoint, "Session expired");
            return Err(BboxError::SessionExpired(
                "Session expired, re-authenticate".to_owned(),
            ));
        }

        if !status.is_success() {
            error!(%endpoint, code = status.as_u16(), "HTTP error");
            return Err(BboxError::Api {
                message: format!("Request failed with HTTP {}", status.as_u16()),
                status_code: Some(status.as_u16()),
            });
        }

        let mut value: Value = response.json().await.map_err(BboxError::Http)?;

        // Unwrap single-element arrays – the Bbox API wraps most responses.
        if let Value::Array(ref arr) = value.clone() {
            if arr.len() == 1 {
                value = arr[0].clone();
            }
        }

        Ok(value)
    }

    /// Retrieve router device information (`GET /device`).
    pub async fn get_router_info(&mut self) -> Result<Router> {
        debug!("Fetching router info");
        let data = self.request("device").await?;
        serde_json::from_value(data["device"].clone()).map_err(BboxError::Json)
    }

    /// Retrieve the list of connected hosts/devices (`GET /hosts`).
    pub async fn get_hosts(&mut self) -> Result<Vec<Host>> {
        debug!("Fetching hosts list");
        let data = self.request("hosts").await?;
        let list = data["hosts"]["list"].clone();
        serde_json::from_value(list).map_err(BboxError::Json)
    }

    /// Retrieve WAN IP statistics (`GET /wan/ip/stats`).
    pub async fn get_wan_ip_stats(&mut self) -> Result<WanIpStats> {
        debug!("Fetching WAN IP stats");
        let data = self.request("wan/ip/stats").await?;
        serde_json::from_value(data["wan"]["ip"]["stats"].clone()).map_err(BboxError::Json)
    }
}
