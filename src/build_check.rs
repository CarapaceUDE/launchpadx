use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
#[cfg(target_os = "windows")]
const NPM_COMMAND: &str = "npm.cmd";
#[cfg(not(target_os = "windows"))]
const NPM_COMMAND: &str = "npm";

pub fn run(project_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let binary_name = if cfg!(windows) {
        "launchpadx.exe"
    } else {
        "launchpadx"
    };

    let bin_path = project_root.join("target/release").join(binary_name);
    let src_dir = project_root.join("src");
    let web_src = project_root.join("web/src");
    let web_dist_bundle = project_root.join("web/dist/assets/index.js");

    if !bin_path.is_file() {
        println!("Release binary not found, building...");
        run_cargo_build(project_root)?;
        println!("Rust binary built successfully.");
    } else if tree_has_file_newer_than(&src_dir, &bin_path) {
        println!("Rust source is newer than binary, rebuilding...");
        run_cargo_build(project_root)?;
        println!("Rust binary built successfully.");
    } else {
        println!("Rust binary is up to date.");
    }

    if !web_dist_bundle.is_file() {
        build_web_ui(project_root)?;
        println!("Web UI built successfully.");
    } else if tree_has_file_newer_than(&web_src, &web_dist_bundle) {
        println!("Web source is newer than bundle, rebuilding...");
        build_web_ui(project_root)?;
        println!("Web UI built successfully.");
    } else {
        println!("Web UI is up to date.");
    }

    stage_artifacts(project_root)?;
    Ok(())
}

fn run_cargo_build(project_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("cargo")
        .args(["build", "--release", "--bin", "launchpadx"])
        .current_dir(project_root)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err("Rust build failed".into())
    }
}

fn build_web_ui(project_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let web_dir = project_root.join("web");
    println!("Building web UI...");
    sync_web_icon(project_root)?;
    ensure_web_dependencies(&web_dir)?;

    let status = Command::new(NPM_COMMAND)
        .args(["run", "build"])
        .current_dir(&web_dir)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err("Web build failed".into())
    }
}

fn sync_web_icon(project_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let src = project_root.join("assets/icon.png");
    let dst_dir = project_root.join("web/src/assets");
    let dst = dst_dir.join("icon.png");

    if !src.is_file() {
        return Err(format!("Missing app icon at {}", src.display()).into());
    }

    fs::create_dir_all(&dst_dir)?;
    fs::copy(&src, &dst)?;
    Ok(())
}

fn ensure_web_dependencies(web_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let node_modules = web_dir.join("node_modules");
    if node_modules.is_dir() {
        return Ok(());
    }

    println!("Installing web UI dependencies (first-time setup)...");
    let package_lock = web_dir.join("package-lock.json");
    let install_args = if package_lock.is_file() {
        ["ci"]
    } else {
        ["install"]
    };

    let status = Command::new(NPM_COMMAND)
        .args(install_args)
        .current_dir(web_dir)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err("Web dependency install failed. Run `cd web && npm ci` manually.".into())
    }
}

fn stage_artifacts(project_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let release_dir = project_root.join("target/release");
    let dist_dir = project_root.join("web/dist");
    let stage_dist = release_dir.join("web/dist");

    if dist_dir.is_dir() {
        if stage_dist.exists() {
            fs::remove_dir_all(&stage_dist)?;
        }
        if let Some(parent) = stage_dist.parent() {
            fs::create_dir_all(parent)?;
        }
        copy_dir_recursive(&dist_dir, &stage_dist)?;
        println!("Staged web UI to {}", stage_dist.display());
    }

    let assets_dir = project_root.join("assets");
    let stage_assets = release_dir.join("assets");
    if assets_dir.is_dir() {
        if stage_assets.exists() {
            fs::remove_dir_all(&stage_assets)?;
        }
        copy_dir_recursive(&assets_dir, &stage_assets)?;
        println!("Staged assets to {}", stage_assets.display());
    }

    let config_src = project_root.join("config.json");
    let config_dst = release_dir.join("config.json");
    if config_src.is_file() && !config_dst.exists() {
        fs::copy(&config_src, &config_dst)?;
        println!("Staged config.json next to release binary");
    }

    Ok(())
}

fn tree_has_file_newer_than(root: &Path, marker: &Path) -> bool {
    if !marker.exists() {
        return true;
    }

    let Ok(marker_modified) = marker.metadata().and_then(|meta| meta.modified()) else {
        return true;
    };

    for path in walk_files(root) {
        if path
            .metadata()
            .ok()
            .and_then(|meta| meta.modified().ok())
            .is_some_and(|modified| modified > marker_modified)
        {
            return true;
        }
    }
    false
}

fn walk_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_files(root, &mut files);
    files
}

fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, files);
        } else if path.is_file() {
            files.push(path);
        }
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn tree_has_file_newer_than_detects_recent_source() {
        let temp =
            std::env::temp_dir().join(format!("launchpadx-build-check-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(temp.join("src")).unwrap();
        fs::write(temp.join("src/lib.rs"), "old").unwrap();
        let marker = temp.join("marker.txt");
        fs::write(&marker, "marker").unwrap();

        thread::sleep(Duration::from_millis(50));
        fs::write(temp.join("src/lib.rs"), "new").unwrap();

        assert!(tree_has_file_newer_than(&temp.join("src"), &marker));
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn tree_has_file_newer_than_is_false_when_up_to_date() {
        let temp =
            std::env::temp_dir().join(format!("launchpadx-build-check-ok-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(temp.join("src")).unwrap();
        fs::write(temp.join("src/lib.rs"), "stable").unwrap();
        let marker = temp.join("marker.txt");
        fs::write(&marker, "marker").unwrap();
        thread::sleep(Duration::from_millis(50));
        fs::write(&marker, "marker-updated").unwrap();

        assert!(!tree_has_file_newer_than(&temp.join("src"), &marker));
        let _ = fs::remove_dir_all(&temp);
    }
}
