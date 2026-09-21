use gpui_kit::{assets::Assets, *};
use rust_embed::RustEmbed;
use std::borrow::Cow;

#[derive(RustEmbed)]
#[folder = "assets/fluent"]
struct FluentAssets;

#[derive(RustEmbed)]
#[folder = "assets/brand"]
struct BrandAssets;

#[derive(RustEmbed)]
#[folder = "assets/toolbar"]
struct ToolbarAssets;

pub struct AppAssets;
impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> anyhow::Result<Option<Cow<'static, [u8]>>> {
        if let Some(name) = path.strip_prefix("toolbar/") {
            return Ok(ToolbarAssets::get(name).map(|file| file.data));
        }
        if let Some(name) = path.strip_prefix("brand/") {
            return Ok(BrandAssets::get(name).map(|file| file.data));
        }
        if let Some(name) = path.strip_prefix("fluent/") {
            return Ok(FluentAssets::get(name).map(|file| file.data));
        }
        Assets.load(path)
    }
    fn list(&self, path: &str) -> anyhow::Result<Vec<SharedString>> {
        if path.starts_with("toolbar") {
            return Ok(ToolbarAssets::iter()
                .map(|name| format!("toolbar/{name}").into())
                .collect());
        }
        if path.starts_with("brand") {
            return Ok(BrandAssets::iter()
                .map(|name| format!("brand/{name}").into())
                .collect());
        }
        if path.starts_with("fluent") {
            return Ok(FluentAssets::iter()
                .map(|name| format!("fluent/{name}").into())
                .collect());
        }
        Assets.list(path)
    }
}
