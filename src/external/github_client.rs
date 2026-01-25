use anyhow::{Context as _, Result};
use mockall::automock;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubReleaseResponse {
    pub tag_name: String,
    pub name: String,
    pub html_url: String,
    pub published_at: String,
}

#[automock]
pub trait GitHubClient: Send + Sync {
    fn fetch_latest_release(&self) -> Result<Option<GitHubReleaseResponse>>;
}

pub struct GitHubClientImpl {
    client: reqwest::blocking::Client,
}

impl Default for GitHubClientImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl GitHubClientImpl {
    pub fn new() -> GitHubClientImpl {
        GitHubClientImpl {
            client: reqwest::blocking::Client::new(),
        }
    }
}

impl GitHubClient for GitHubClientImpl {
    fn fetch_latest_release(&self) -> Result<Option<GitHubReleaseResponse>> {
        let mut request = self
            .client
            .get("https://api.github.com/repos/fenv-org/fenv/releases/latest")
            .header("User-Agent", "fenv");

        // Add authorization header if GH_TOKEN or GITHUB_TOKEN is set
        if let Some(token) = std::env::var("GH_TOKEN")
            .ok()
            .or_else(|| std::env::var("GITHUB_TOKEN").ok())
        {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request
            .send()
            .context("Failed to fetch latest release from GitHub")?;

        if response.status() == 404 {
            return Ok(None);
        }

        let text = response
            .error_for_status()
            .context("GitHub API returned an error")?
            .text()
            .context("Failed to read response body")?;

        let release = serde_json::from_str::<GitHubReleaseResponse>(&text)
            .context("Failed to parse GitHub release response")?;

        Ok(Some(release))
    }
}
