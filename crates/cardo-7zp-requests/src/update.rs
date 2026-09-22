use anyhow::{Context, Result, bail};
use cardo_7zp_core::i18n::tr;
use reqwest::blocking::Client;
use semver::Version;
use serde::Deserialize;
use std::time::Duration;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const REPOSITORY: Option<&str> = option_env!("SEVENZIP_RELEASE_REPOSITORY");

pub enum Status {
    Checking,
    Unconfigured,
    Current,
    Available {
        version: String,
        download: String,
        release: String,
    },
    Failed(String),
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

pub fn check() -> Result<Status> {
    let Some(repository) = REPOSITORY.filter(|value| !value.is_empty()) else {
        return Ok(Status::Unconfigured);
    };
    let client = Client::builder()
        .user_agent(concat!("7zplus/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(25))
        .build()?;
    let response = client
        .get(format!(
            "https://api.github.com/repos/{repository}/releases/latest"
        ))
        .header("Accept", "application/vnd.github+json")
        .send()?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        bail!(tr("update-no-release"));
    }
    let release: Release = response.error_for_status()?.json()?;
    let version = Version::parse(release.tag_name.trim_start_matches('v'))
        .context(tr("update-version-invalid"))?;
    if release.draft || release.prerelease || !version.pre.is_empty() {
        bail!(tr("update-no-release"));
    }
    if version <= Version::parse(VERSION)? {
        return Ok(Status::Current);
    }
    let asset = release
        .assets
        .into_iter()
        .find(|asset| asset.name == "7zplus-amd64-installer.exe")
        .context(tr("update-asset-missing"))?;
    let base = format!("https://github.com/{repository}/releases/");
    if !asset
        .browser_download_url
        .starts_with(&format!("{base}download/"))
    {
        bail!(tr("update-url-invalid"));
    }
    let mut page = reqwest::Url::parse(&format!("{base}tag/"))?;
    page.path_segments_mut()
        .map_err(|_| anyhow::anyhow!(tr("update-url-invalid")))?
        .pop_if_empty()
        .push(&release.tag_name);
    Ok(Status::Available {
        version: version.to_string(),
        download: asset.browser_download_url,
        release: page.into(),
    })
}
