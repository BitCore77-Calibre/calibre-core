use crate::error::LightError;
use serde_json::json;

#[derive(Clone)]
pub struct RpcClient {
    url: String,
    http: reqwest::Client,
}

impl RpcClient {
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into(), http: reqwest::Client::new() }
    }

    pub async fn call(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, LightError> {
        let body = json!({
            "jsonrpc": "2.0", "id": 1,
            "method": method, "params": params
        });
        let resp: serde_json::Value = self
            .http
            .post(&self.url)
            .json(&body)
            .send()
            .await
            .map_err(|e| LightError::Transport(e.to_string()))?
            .json()
            .await
            .map_err(|e| LightError::Transport(e.to_string()))?;
        if let Some(err) = resp.get("error") {
            let code = err.get("code").and_then(|v| v.as_i64()).unwrap_or(-1);
            let message = err
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            return Err(LightError::Rpc { code, message });
        }
        resp.get("result")
            .cloned()
            .ok_or_else(|| LightError::Parse("no result".into()))
    }
}
