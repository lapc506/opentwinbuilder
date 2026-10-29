//! Thingport adapter — self-hosted 3D model library as an on-demand asset source.
//!
//! Implements the `ModelSource` port with a blocking HTTP client (ureq). The
//! token is sent as a Bearer header. Expected Thingport REST shape (subject to the
//! upstream API): `GET /api/models?search=` -> `[{id,name,...}]`, and
//! `GET /api/models/{id}` -> model JSON.

use crate::ports::model_source::{ModelSource, ModelSummary};
use serde_json::Value;

/// Thin client to a self-hosted Thingport instance.
#[derive(Debug, Clone)]
pub struct ThingportClient {
    base_url: String,
    api_token: String,
}

impl ThingportClient {
    /// `base_url` e.g. `http://localhost:80` (Thingport default port).
    pub fn new(base_url: impl Into<String>, api_token: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_token: api_token.into(),
        }
    }

    fn get(&self, path: &str) -> Result<Value, String> {
        let url = format!("{}{}", self.base_url, path);
        let resp = ureq::get(&url)
            .set("Authorization", &format!("Bearer {}", self.api_token))
            .call()
            .map_err(|e| format!("thingport GET {url}: {e}"))?;
        let text = resp
            .into_string()
            .map_err(|e| format!("thingport read {url}: {e}"))?;
        serde_json::from_str(&text).map_err(|e| format!("thingport parse {url}: {e}"))
    }

    /// True when a Thingport instance URL has been configured.
    pub fn is_configured(&self) -> bool {
        !self.base_url.is_empty()
    }
}

impl ModelSource for ThingportClient {
    fn search(&self, query: &str) -> Result<Vec<ModelSummary>, String> {
        let encoded = percent_encode(query);
        let value = self.get(&format!("/api/models?search={encoded}"))?;
        let arr = value.as_array().ok_or_else(|| "thingport.search: expected array".to_string())?;
        let mut out = Vec::with_capacity(arr.len());
        for item in arr {
            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            out.push(ModelSummary { id, name, source: "thingport" });
        }
        Ok(out)
    }

    fn fetch(&self, id: &str) -> Result<Value, String> {
        let encoded = percent_encode(id);
        self.get(&format!("/api/models/{encoded}"))
    }
}

/// Minimal RFC-3986 percent-encoding enough for query params in a POC.
fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for b in input.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}
