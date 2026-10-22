//! Use the nested plugin workspace rather than xtask's outermost-manifest heuristic.
fn main() -> nice_plug_xtask::Result<()> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    std::env::set_current_dir(root)?;
    let mut args = std::env::args().skip(1).peekable();
    let command = args.next().unwrap_or_default();
    if command != "bundle" {
        return Err(std::io::Error::other("usage: cargo xtask bundle -p lime-synth [-p …] [--release]").into());
    }
    let mut packages = Vec::new();
    while args.peek().is_some_and(|a| a == "-p") {
        args.next();
        packages.push(
            args.next()
                .ok_or_else(|| std::io::Error::other("missing package after -p"))?,
        );
    }
    if packages.is_empty() {
        return Err(std::io::Error::other("pass at least one -p package").into());
    }
    let other: Vec<_> = args.collect();
    let meta = cargo_metadata::MetadataCommand::new().no_deps().exec()?;
    nice_plug_xtask::build(&packages, &other)?;
    for package in packages {
        nice_plug_xtask::bundle(meta.target_directory.as_std_path(), &package, &other, false)?;
    }
    Ok(())
}
