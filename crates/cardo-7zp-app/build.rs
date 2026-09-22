fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    let workspace = manifest.join("../..");
    let icon = workspace.join("assets/brand/app.ico");
    let source = workspace.join("packaging/app.rc");
    println!("cargo:rerun-if-changed={}", icon.display());
    println!("cargo:rerun-if-changed={}", source.display());
    let rc = std::path::PathBuf::from(std::env::var("OUT_DIR")?).join("app.rc");
    let icon = icon.canonicalize()?.to_string_lossy().replace('\\', "/");
    std::fs::write(&rc, format!("1 ICON \"{icon}\"\n"))?;
    embed_resource::compile(&rc, embed_resource::NONE).manifest_required()?;
    Ok(())
}
