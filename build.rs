use std::path::Path;
use std::path::PathBuf;
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
    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=web/src/assets/icon.png");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        compile_windows_resources(Path::new(env!("CARGO_MANIFEST_DIR")));
    }

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

fn compile_windows_resources(project_root: &Path) {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR is set"));
    let rc_path = out_dir.join("launchpadx.rc");
    let res_path = out_dir.join("launchpadx.res");
    let icon_path = project_root
        .join("assets/icon.ico")
        .display()
        .to_string()
        .replace('\\', "/");
    let version = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string());
    let mut parts = version
        .split('.')
        .map(|part| part.parse::<u16>().unwrap_or(0));
    let major = parts.next().unwrap_or(0);
    let minor = parts.next().unwrap_or(0);
    let patch = parts.next().unwrap_or(0);
    let resource = format!(
        r#"1 ICON "{icon_path}"
1 VERSIONINFO
FILEVERSION {major},{minor},{patch},0
PRODUCTVERSION {major},{minor},{patch},0
FILEOS 0x40004
FILETYPE 0x1
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "040904B0"
    BEGIN
      VALUE "CompanyName", "Carapace LLC\0"
      VALUE "FileDescription", "LaunchPadX desktop endpoint launcher\0"
      VALUE "FileVersion", "{version}\0"
      VALUE "OriginalFilename", "launchpadx.exe\0"
      VALUE "ProductName", "LaunchPadX\0"
      VALUE "ProductVersion", "{version}\0"
      VALUE "LegalCopyright", "Copyright (c) 2026 Carapace LLC\0"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x0409, 1200
  END
END
"#
    );
    std::fs::write(&rc_path, resource).expect("write Windows resource script");

    let rc_exe = find_windows_resource_compiler().unwrap_or_else(|| PathBuf::from("rc.exe"));
    let status = Command::new(&rc_exe)
        .arg("/nologo")
        .arg(format!("/fo{}", res_path.display()))
        .arg(&rc_path)
        .status()
        .unwrap_or_else(|err| panic!("Could not start {}: {err}", rc_exe.display()));
    if !status.success() {
        panic!("Windows resource compiler failed with {status}");
    }
    println!("cargo:rustc-link-arg-bin=launchpadx={}", res_path.display());
}

fn find_windows_resource_compiler() -> Option<PathBuf> {
    if let (Some(sdk_dir), Some(sdk_version)) = (
        std::env::var_os("WindowsSdkDir"),
        std::env::var_os("WindowsSDKVersion"),
    ) {
        let candidate = PathBuf::from(sdk_dir)
            .join("bin")
            .join(sdk_version)
            .join("x64")
            .join("rc.exe");
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    let kits_bin =
        PathBuf::from(std::env::var_os("ProgramFiles(x86)")?).join("Windows Kits/10/bin");
    let mut versions = std::fs::read_dir(kits_bin)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    versions.sort();
    versions.reverse();
    versions
        .into_iter()
        .map(|version| version.join("x64/rc.exe"))
        .find(|candidate| candidate.is_file())
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
