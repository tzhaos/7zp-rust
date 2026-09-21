fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=packaging/app.rc");
    println!("cargo:rerun-if-changed=assets/brand/app.ico");
    embed_resource::compile("packaging/app.rc", embed_resource::NONE).manifest_required()?;
    Ok(())
}
