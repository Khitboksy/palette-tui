use std::fs;
use std::process;

fn main() {
    // Block the entire script if DEV_OPTIONS is 0
    if std::env::var("DEV_OPTIONS").unwrap_or_default() != "1" {
        eprintln!("bump: dev-only tool (set DEV_OPTIONS=1 or use `nix develop`)");
        process::exit(1);
    }

    let args: Vec<String> = std::env::args().collect();
    let bump = match args.get(1).map(|s| s.as_str()) {
        Some("-p") | Some("--patch") => "patch",
        Some("-m") | Some("--minor") => "minor",
        Some("-b") | Some("--major") => "major",
        _ => {
            eprintln!("Usage: {} <--patch|--minor|--major|-p|-m|-b>", args[0]);
            process::exit(1);
        }
    };

    // Read and parse Cargo.toml
    let cargo_raw = fs::read_to_string("Cargo.toml").expect("failed to read Cargo.toml");
    let cargo: toml::Value = cargo_raw.parse().expect("failed to parse Cargo.toml");

    let current = cargo
        .get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .expect("no version found in [package]");

    let parts: Vec<u32> = current
        .split('.')
        .map(|p| p.parse().expect("invalid version segment"))
        .collect();
    if parts.len() != 3 {
        eprintln!("expected semver (x.y.z), got: {current}");
        process::exit(1);
    }
    let (mut major, mut minor, mut patch) = (parts[0], parts[1], parts[2]);

    match bump {
        "major" => {
            major += 1;
            minor = 0;
            patch = 0;
        }
        "minor" => {
            minor += 1;
            patch = 0;
        }
        "patch" => {
            patch += 1;
        }
        _ => unreachable!(),
    }
    let new_version = format!("{major}.{minor}.{patch}");

    // Update Cargo.toml -- string-replace only the [package] version line
    let new_cargo = cargo_raw.replacen(
        &format!("version = \"{current}\""),
        &format!("version = \"{new_version}\""),
        1,
    );
    fs::write("Cargo.toml", &new_cargo).expect("failed to write Cargo.toml");

    // Update nix/package.nix if it exists
    let nix_path = "nix/package.nix";
    if let Ok(nix_raw) = fs::read_to_string(nix_path) {
        let new_nix = nix_raw.replacen(
            &format!("version = \"{current}\""),
            &format!("version = \"{new_version}\""),
            1,
        );
        fs::write(nix_path, &new_nix).expect("failed to write nix/package.nix");
    }

    println!("bumped {current} -> {new_version} ({bump})");
}
