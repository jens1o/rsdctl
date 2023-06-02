use anyhow::{anyhow, Result};
use ::reqwest::redirect::Policy;
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

pub fn random_english_article() -> Result<String> {
    let query = "https://randomincategory.toolforge.org/?category=All_Wikipedia_level-4_vital_articles&server=en.wikipedia.org&cmnamespace=&cmtype=&returntype=subject&debug=0";

    let client = reqwest::Client::builder()
        .user_agent("curl/8.1.2")
        .redirect(Policy::none())
        .build()?;

    let response = client.get(query).send()?;
    let redir_header = response.headers().get("location").ok_or(anyhow!("Could not find location header"))?;
    let redir_location = redir_header.to_str()?;

    let redir_prefix = "https://en.wikipedia.org/wiki/";

    if !redir_location.starts_with(redir_prefix) {
        return Err(anyhow!("Redirect had unexpected format: {}", redir_location));
    }

    Ok(String::from(&redir_location[redir_prefix.len()..]))
}
