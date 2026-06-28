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
    println!("cargo:rerun-if-changed=web/src/assets/icon.png");

    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    let web_dir = Path::new(cargo_manifest_dir).join("web");
    let web_dist = web_dir.join("dist/index.html");

    if !web_dist.is_file() {
        if let Err(err) = ensure_web_dependencies(&web_dir) {
            println!("cargo:warning={err}");
            return;
        }
        if let Err(err) = sync_web_icon(Path::new(cargo_manifest_dir)) {
            println!("cargo:warning={err}");
            return;
        }

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

fn sync_web_icon(project_root: &Path) -> Result<(), String> {
    let src = project_root.join("assets/icon.png");
    let dst_dir = project_root.join("web/src/assets");
    let dst = dst_dir.join("icon.png");

    if !src.is_file() {
        return Err(format!(
            "Missing app icon at {}; cannot bundle UI branding",
            src.display()
        ));
    }

    std::fs::create_dir_all(&dst_dir)
        .map_err(|err| format!("Could not create {}: {err}", dst_dir.display()))?;
    std::fs::copy(&src, &dst)
        .map_err(|err| format!("Could not copy app icon into web UI assets: {err}"))?;
    Ok(())
}

fn ensure_web_dependencies(web_dir: &Path) -> Result<(), String> {
    let node_modules = web_dir.join("node_modules");
    let package_lock = web_dir.join("package-lock.json");

    if node_modules.is_dir() {
        return Ok(());
    }

    let install_args = if package_lock.is_file() {
        ["ci"]
    } else {
        ["install"]
    };

    let status = Command::new(NPM_COMMAND)
        .args(install_args)
        .current_dir(web_dir)
        .status()
        .map_err(|err| format!("Could not start npm to install web UI dependencies: {err}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(
            "Web UI dependency install failed. Run `cd web && npm ci` manually, then rebuild."
                .to_string(),
        )
    }
}
