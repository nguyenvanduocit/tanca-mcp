use std::collections::BTreeMap;
use std::fmt;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use reqwest::Method;
use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::sync::OnceCell;

use super::types::Response;
use crate::log::go_duration;
use crate::logf;

/// Default Tanca API base URL.
pub const DEFAULT_BASE_URL: &str = "https://api.tanca.io/api/v4";

/// Tanca API configuration.
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// e.g. `https://api.tanca.io/api/v4`.
    pub base_url: String,
    /// Full `authorization` header value, including the `Bearer ` prefix.
    pub token: String,
    /// MongoDB ObjectId; fetched from the API when empty.
    pub shop_id: String,
    /// MongoDB ObjectId.
    pub branch_id: String,
    /// Base64-encoded device info sent as the `device` header.
    pub device_header: String,
    /// IANA timezone, default `Asia/Saigon`.
    pub timezone: String,
    /// Language code, default `vi`.
    pub lang: String,
    /// `1` for admin, `0` for regular.
    pub is_admin: String,
}

/// Query string parameters. Keys are sorted on encoding, like Go's
/// `url.Values.Encode`.
pub type Query = BTreeMap<String, String>;

/// A non-2xx response from the Tanca API.
#[derive(Debug, Clone)]
pub struct ApiError {
    pub status_code: u16,
    pub body: String,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "tanca API error {}: {}",
            self.status_code,
            truncate(&self.body, 200)
        )
    }
}

impl std::error::Error for ApiError {}

/// Returns true for 4xx errors, which are not retried.
fn is_client_error(err: &anyhow::Error) -> bool {
    err.downcast_ref::<ApiError>()
        .is_some_and(|e| (400..500).contains(&e.status_code))
}

/// Limits a string to `max_len` bytes (on a char boundary) for logging.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }
    let mut end = max_len;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}...", &s[..end])
}

/// Encodes a query like Go's `url.Values.Encode`: sorted keys, spaces as
/// `+`, and only `A-Za-z0-9-_.~` left unescaped.
pub fn encode_query(query: &Query) -> String {
    fn escape(s: &str, out: &mut String) {
        for b in s.bytes() {
            match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    out.push(b as char)
                }
                b' ' => out.push('+'),
                _ => out.push_str(&format!("%{b:02X}")),
            }
        }
    }
    let mut out = String::new();
    for (k, v) in query {
        if !out.is_empty() {
            out.push('&');
        }
        escape(k, &mut out);
        out.push('=');
        escape(v, &mut out);
    }
    out
}

enum Body<'a> {
    None,
    Json(&'a (dyn erased::JsonBody + Sync)),
    Form(&'a [(&'a str, String)]),
}

mod erased {
    /// Object-safe wrapper so request bodies can be passed by reference.
    pub trait JsonBody {
        fn to_json(&self) -> serde_json::Result<Vec<u8>>;
    }
    impl<T: serde::Serialize> JsonBody for T {
        fn to_json(&self) -> serde_json::Result<Vec<u8>> {
            serde_json::to_vec(self)
        }
    }
}

/// Tanca API client.
pub struct Client {
    cfg: Config,
    http: reqwest::Client,
    max_retries: u32,
    base_delay: Duration,
    shop_id: OnceCell<String>,
}

impl Client {
    /// Creates a client, filling unset config fields with defaults.
    pub fn new(mut cfg: Config) -> Self {
        if cfg.base_url.is_empty() {
            cfg.base_url = DEFAULT_BASE_URL.into();
        }
        if cfg.timezone.is_empty() {
            cfg.timezone = "Asia/Saigon".into();
        }
        if cfg.lang.is_empty() {
            cfg.lang = "vi".into();
        }
        if cfg.is_admin.is_empty() {
            cfg.is_admin = "1".into();
        }
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to build HTTP client");
        Self {
            cfg,
            http,
            max_retries: 3,
            base_delay: Duration::from_secs(1),
            shop_id: OnceCell::new(),
        }
    }

    /// Overrides the retry policy (used by tests).
    pub fn with_retry(mut self, max_retries: u32, base_delay: Duration) -> Self {
        self.max_retries = max_retries;
        self.base_delay = base_delay;
        self
    }

    /// Executes a request, retrying network errors and 5xx responses with
    /// exponential backoff. 4xx responses are returned immediately.
    async fn do_request(
        &self,
        method: Method,
        path: &str,
        query: Option<&Query>,
        body: Body<'_>,
    ) -> Result<Vec<u8>> {
        let label = if matches!(body, Body::Form(_)) {
            "form request"
        } else {
            "request"
        };
        let mut last_err = None;
        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                let delay = self.calculate_backoff(attempt);
                logf!(
                    "[tanca] retry {}/{} after {}",
                    attempt,
                    self.max_retries,
                    go_duration(delay)
                );
                tokio::time::sleep(delay).await;
            }

            match self
                .execute_request(method.clone(), path, query, &body)
                .await
            {
                Ok(resp) => return Ok(resp),
                Err(err) => {
                    if is_client_error(&err) {
                        return Err(err);
                    }
                    logf!("[tanca] {label} failed (attempt {}): {err:#}", attempt + 1);
                    last_err = Some(err);
                }
            }
        }
        Err(last_err
            .unwrap_or_else(|| anyhow!("no attempts made"))
            .context("max retries exceeded"))
    }

    /// Performs a single HTTP request.
    async fn execute_request(
        &self,
        method: Method,
        path: &str,
        query: Option<&Query>,
        body: &Body<'_>,
    ) -> Result<Vec<u8>> {
        let mut url = format!("{}{}", self.cfg.base_url, path);
        if let Some(q) = query.filter(|q| !q.is_empty()) {
            url.push('?');
            url.push_str(&encode_query(q));
        }

        let mut req = self
            .http
            .request(method, &url)
            .headers(self.headers().context("create request")?);
        match body {
            Body::None => {}
            Body::Json(b) => {
                let data = b.to_json().context("marshal body")?;
                req = req.header(CONTENT_TYPE, "application/json").body(data);
            }
            Body::Form(fields) => {
                let mut form = reqwest::multipart::Form::new();
                for (k, v) in fields.iter() {
                    form = form.text(k.to_string(), v.clone());
                }
                req = req.multipart(form);
            }
        }

        let resp = req.send().await.context("execute request")?;
        let status = resp.status();
        let bytes = resp.bytes().await.context("read response")?;
        if status.as_u16() >= 400 {
            return Err(ApiError {
                status_code: status.as_u16(),
                body: String::from_utf8_lossy(&bytes).into_owned(),
            }
            .into());
        }
        Ok(bytes.to_vec())
    }

    /// Required Tanca headers.
    fn headers(&self) -> Result<HeaderMap> {
        let mut h = HeaderMap::new();
        let mut set = |name: &'static str, value: &str| -> Result<()> {
            let v = HeaderValue::from_str(value)
                .with_context(|| format!("invalid value for header {name:?}"))?;
            h.insert(HeaderName::from_static(name), v);
            Ok(())
        };
        set("authorization", &self.cfg.token)?;
        set("device", &self.cfg.device_header)?;
        set("is-admin", &self.cfg.is_admin)?;
        set("lang", &self.cfg.lang)?;
        set("timezone", &self.cfg.timezone)?;
        h.insert(ACCEPT, HeaderValue::from_static("application/json"));
        Ok(h)
    }

    /// Exponential backoff (1s, 2s, 4s, ...) capped at one minute, plus
    /// 0-25% jitter.
    fn calculate_backoff(&self, attempt: u32) -> Duration {
        let factor = 1u32.checked_shl(attempt - 1).unwrap_or(u32::MAX);
        let delay = self
            .base_delay
            .saturating_mul(factor)
            .min(Duration::from_secs(60));
        let max_jitter = (delay / 4).as_nanos() as u64;
        let jitter = if max_jitter == 0 {
            0
        } else {
            fastrand::u64(0..max_jitter)
        };
        delay + Duration::from_nanos(jitter)
    }

    /// Performs a GET request.
    pub async fn get(&self, path: &str, query: Option<&Query>) -> Result<Vec<u8>> {
        self.do_request(Method::GET, path, query, Body::None).await
    }

    /// Performs a POST request with a JSON body.
    pub async fn post<B: Serialize + Sync>(&self, path: &str, body: &B) -> Result<Vec<u8>> {
        self.do_request(Method::POST, path, None, Body::Json(body))
            .await
    }

    /// Performs a POST request with a `multipart/form-data` body.
    pub async fn post_form(&self, path: &str, fields: &[(&str, String)]) -> Result<Vec<u8>> {
        self.do_request(Method::POST, path, None, Body::Form(fields))
            .await
    }

    /// Returns the shop ID, fetching it from the API on first use when it
    /// is not configured. Only a successful lookup is cached.
    pub async fn get_shop_id(&self) -> Result<String> {
        if !self.cfg.shop_id.is_empty() {
            return Ok(self.cfg.shop_id.clone());
        }
        self.shop_id
            .get_or_try_init(|| async {
                logf!("[tanca] fetching shop ID from API...");
                let shop = self.get_shop_detail().await.context("auto-fetch shop ID")?;
                logf!("[tanca] shop ID: {} ({})", shop.id, shop.name);
                Ok::<_, anyhow::Error>(shop.id)
            })
            .await
            .cloned()
    }

    /// The configured branch ID.
    pub fn branch_id(&self) -> &str {
        &self.cfg.branch_id
    }
}

/// Decodes a Tanca response envelope and returns its `data`, failing on a
/// non-zero `error_code`.
pub(crate) fn decode<T: DeserializeOwned + Default>(body: &[u8]) -> Result<T> {
    let resp: Response<T> = crate::json::from_slice(body).context("unmarshal response")?;
    if resp.error_code != 0 {
        return Err(anyhow!(
            "API error {}: [{}]",
            resp.error_code,
            resp.message.join(" ")
        ));
    }
    Ok(resp.data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_encoding_matches_go() {
        let mut q = Query::new();
        q.insert("department_ids[0]".into(), "a b".into());
        q.insert("a".into(), "x*~".into());
        assert_eq!(encode_query(&q), "a=x%2A~&department_ids%5B0%5D=a+b");
    }

    #[test]
    fn truncate_respects_char_boundaries() {
        assert_eq!(truncate("abc", 5), "abc");
        assert_eq!(truncate("ăăă", 3), "ă...");
    }

    #[test]
    fn backoff_grows_and_has_bounded_jitter() {
        let c = Client::new(Config::default());
        for attempt in 1..=3 {
            let base = Duration::from_secs(1 << (attempt - 1));
            let d = c.calculate_backoff(attempt);
            assert!(d >= base && d < base + base / 4, "{d:?}");
        }
    }
}
