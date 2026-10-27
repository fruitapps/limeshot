fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../../packaging/windows/LimeShot.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        // Target-specific links keep resources out of the library, tests and examples.
        let mut resource = winresource::WindowsResource::new();
        resource
            .set_icon(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../packaging/windows/LimeShot.ico"
            ))
            .set("ProductName", "LimeShot")
            .set("FileDescription", "LimeShot");
        resource.compile_for("limeshot")?;
        resource.set("FileDescription", "LimeShot command-line tool");
        resource.compile_for("limeshot-cli")?;
    }
    Ok(())
}
