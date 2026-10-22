#!/usr/bin/env python3
"""Build and collect the nine VST3 bundles; never modifies the system plugin folders."""
import argparse
import json
import platform
from pathlib import Path
import shutil
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parent
PLUGINS = ["synth", "sampler", "utility", "eq", "filter", "delay", "room", "comp", "limiter"]
NAMES = ["LimeSynth", "Lime Sampler", "Lime Utility", "Lime EQ", "Lime Filter", "Lime Delay", "Lime Room", "Lime Comp", "Lime Limiter"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--debug", action="store_true", help="Keep processing allocation assertions enabled")
    parser.add_argument("--target", help="Rust compilation target (must already be installed)")
    args = parser.parse_args()
    command = ["cargo", "xtask", "bundle"]
    for slug in PLUGINS:
        command += ["-p", f"lime-{slug}"]
    if not args.debug:
        command += ["--release"]
    if args.target:
        command += ["--target", args.target]
    subprocess.run(command, cwd=ROOT, check=True)
    profile = "debug" if args.debug else "release"
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--locked", "--format-version", "1"], cwd=ROOT
    ))
    # nice-plug's bundler writes to Cargo's target directory, independently of profile.
    source = Path(metadata["target_directory"]) / "bundled"
    dest = ROOT / "target" / "debug-bundled" if args.debug else ROOT / "dist"
    if args.target:
        dest /= args.target
    dest.mkdir(parents=True, exist_ok=True)
    for name in NAMES:
        bundle = source / f"{name}.vst3"
        if not bundle.exists():
            raise SystemExit(f"Bundle missing: {bundle}")
        output = dest / bundle.name
        if output.is_dir():
            shutil.rmtree(output)
        elif output.exists():
            output.unlink()
        if bundle.is_dir():
            shutil.copytree(bundle, output)
        else:
            shutil.copy2(bundle, output)
        print(output)
    for name in ["README.md", "THIRD_PARTY.md", "VALIDATION.md"]:
        if (ROOT / name).exists():
            shutil.copy2(ROOT / name, dest / name)
    license_dir = dest / "licenses"
    license_dir.mkdir(exist_ok=True)
    for name in ["LICENSE", "LICENSE-MIT", "LICENSE-APACHE", "ATTRIBUTION.md"]:
        shutil.copy2(ROOT.parent / name, license_dir / name)
    for path in (ROOT.parent / "assets/fonts").glob("OFL-*.txt"):
        shutil.copy2(path, license_dir / path.name)
    for package in metadata["packages"]:
        package_dir = Path(package["manifest_path"]).parent
        texts = [p for pattern in ["LICENSE*", "COPYING*", "NOTICE*"]
                 for p in package_dir.glob(pattern) if p.is_file()]
        if texts:
            package_license_dir = license_dir / f'{package["name"]}-{package["version"]}'
            package_license_dir.mkdir(exist_ok=True)
            for path in texts:
                shutil.copy2(path, package_license_dir / path.name)
    if not args.debug:
        os_name = {"Darwin": "macOS"}.get(platform.system(), platform.system())
        tag = args.target or f"{os_name}-{platform.machine()}"
        archive = dest / f"Lime-Native-{tag}.zip"
        with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED, compresslevel=6, strict_timestamps=False) as output:
            for path in sorted(dest.rglob("*")):
                if path.is_file() and path.suffix != ".zip":
                    output.write(path, path.relative_to(dest))
        print(archive)
    print(f"Built all nine {profile} VST3 plugins in {dest}")


if __name__ == "__main__":
    main()
