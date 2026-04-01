use reqwest::Response;
use serde_json::Value;
use toml_edit::Document;

pub async fn fetch_json(resp: Response) -> Result<Value, reqwest::Error> {
    let json: Value = resp.json::<Value>().await?;
    Ok(json)
}

pub fn parse_toml(content: &str) -> Result<toml_edit::ImDocument<String>, toml_edit::TomlError> {
    let doc = content.parse::<toml_edit::ImDocument<String>>()?;
    Ok(doc)
}
