use anyhow::{anyhow, Result};
use reqwest::blocking as reqwest;
use serde_json::Value;

pub fn download_article(language: &str, title: &str) -> Result<(String, String)> {
    let query = format!("https://{}.wikipedia.org/w/api.php?action=parse&page={}&prop=wikitext&formatversion=2&format=json", language, title);

    let content = reqwest::get(query)?;
    let content = content.text()?;
    let content: Value = serde_json::from_str(&content)?;

    let wikitext = content
        .pointer("/parse/wikitext")
        .and_then(|val| val.as_str())
        .ok_or(anyhow!("JSON response did not contain wikitext"))?;

    let page_title = content
        .pointer("/parse/title")
        .and_then(|val| val.as_str())
        .ok_or(anyhow!("JSON response did not contain page title"))?;

    Ok((String::from(page_title), String::from(wikitext)))
}
