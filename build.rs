use std::path::Path;
use std::process::Command;
use std::fs;
use std::io;

fn main() {
    println!("cargo:rerun-if-changed=../codex-launchpad/src");
    println!("cargo:rerun-if-changed=../codex-launchpad/package.json");
    println!("cargo:rerun-if-changed=../codex-launchpad/bun.lock");

    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    let frontend_dir = Path::new(cargo_manifest_dir).join("../codex-launchpad");
    let frontend_dist = frontend_dir.join("dist/client");

    if !frontend_dist.exists() {
        println!("cargo:warning=Frontend not built. Building...");
        let status = Command::new("npm")
                .args(["run", "build"])
                .current_dir(&frontend_dir)
                .status()
                .expect("Failed to run npm build");
        if !status.success() {
            println!("cargo:warning=Frontend build failed, using existing assets if available");
            }
        }

    let dist_dst = Path::new(cargo_manifest_dir).join("codex-launchpad/dist/client");
    let dist_src = &frontend_dist;

    if dist_src.exists() {
        copy_dir_all(dist_src, &dist_dst).expect("Failed to copy frontend assets");
        }
}

fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
            } else {
            fs::copy(&src_path, &dst_path)?;
            }
        }
    Ok(())
}
