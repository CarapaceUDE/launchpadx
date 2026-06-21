use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=web/src");
    println!("cargo:rerun-if-changed=web/package.json");
    println!("cargo:rerun-if-changed=web/vite.config.ts");

    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    let web_dir = Path::new(cargo_manifest_dir).join("web");
    let web_dist = web_dir.join("dist/index.html");

    if !web_dist.is_file() {
        println!("cargo:warning=Web UI not built. Run: cd web && npm run build");
        let status = Command::new("npm")
            .args(["run", "build"])
            .current_dir(&web_dir)
            .status();
        if let Ok(status) = status {
            if !status.success() {
                println!("cargo:warning=Web UI build failed during cargo build");
            }
        }
    }
}