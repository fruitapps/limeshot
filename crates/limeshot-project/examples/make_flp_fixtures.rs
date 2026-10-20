//! Write the `.flp` test files to `crates/limeshot-project/tests/fixtures/flp`.
//!
//! `cargo run -p limeshot-project --example make_flp_fixtures`

fn main() -> std::io::Result<()> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/flp");
    std::fs::create_dir_all(&dir)?;
    for (name, bytes) in limeshot_project::flp::fixtures() {
        std::fs::write(dir.join(name), &bytes)?;
        println!("wrote {name} ({} bytes)", bytes.len());
    }
    Ok(())
}
