use std::path::Path;
use std::process::Command;

#[cfg(target_os = "windows")]
const NPM_COMMAND: &str = "npm.cmd";
#[cfg(not(target_os = "windows"))]
const NPM_COMMAND: &str = "npm";

fn main() {
    println!("cargo:rerun-if-changed=web/src");
    println!("cargo:rerun-if-changed=web/package.json");
    println!("cargo:rerun-if-changed=web/package-lock.json");
    println!("cargo:rerun-if-changed=web/vite.config.ts");
    println!("cargo:rerun-if-changed=assets/icon.png");

    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    let web_dir = Path::new(cargo_manifest_dir).join("web");
    let web_dist = web_dir.join("dist/index.html");

    if !web_dist.is_file() {
        let status = Command::new(NPM_COMMAND)
            .args(["run", "build"])
            .current_dir(&web_dir)
            .status();
        match status {
            Ok(status) if status.success() => {}
            Ok(_) => println!("cargo:warning=Web UI build failed during cargo build"),
            Err(err) => println!("cargo:warning=Could not start npm to build web UI: {err}"),
        }
    }
}
