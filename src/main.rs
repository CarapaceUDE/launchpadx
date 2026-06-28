use std::path::{Path, PathBuf};

fn main() {
    if let Err(err) = dispatch() {
        eprintln!("codex-launchpad: {err}");
        std::process::exit(1);
    }
}

fn dispatch() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.serve_only {
        let root = codex_launchpad::web_backend::resolve_gui_root();
        let config_path = args
            .config_path
            .unwrap_or_else(|| default_config_path(&root));
        codex_launchpad::web_backend::serve_web_ui(root, config_path, args.port)?;
        return Ok(());
    }

    if args.gui {
        let root = codex_launchpad::web_backend::resolve_gui_root();
        let config_path = args.config_path.unwrap_or_else(|| root.join("config.json"));
        codex_launchpad::web_backend::launch_web_gui(root, config_path)?;
        return Ok(());
    }

    if std::env::args_os().any(|a| a == "--help" || a == "-h") {
        print_help();
        return Ok(());
    }

    if args.diagnose {
        let root = std::env::current_dir()?;
        let config_path = args
            .config_path
            .unwrap_or_else(|| default_config_path(&root));
        codex_launchpad::diagnose::run(&config_path, &root)?;
        return Ok(());
    }

    if args.build_check {
        let root = std::env::current_dir()?;
        codex_launchpad::build_check::run(&root)?;
        return Ok(());
    }

    if args.needs_async() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        return runtime.block_on(run_cli_async(args));
    }

    run_cli_sync(args)
}

fn run_cli_sync(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.to_path_buf()))
        .unwrap_or(std::env::current_dir()?);

    if !args.has_explicit_action() {
        print_help();
        return Ok(());
    }

    if !args.unknown_args.is_empty() {
        return Err(format!(
            "unrecognized argument(s): {}; pass --help to see available commands",
            args.unknown_args.join(", ")
        )
        .into());
    }

    let config_path = args
        .config_path
        .unwrap_or_else(|| default_config_path(&root));

    let config = codex_launchpad::config::LauncherConfig::read(&config_path)?;
    let pid_file = codex_launchpad::app_logic::codex_pid_file(&config_path);

    if args.restore {
        println!("{}", codex_launchpad::app_logic::restore(&config)?);
        return Ok(());
    }

    if args.refresh_models {
        let cache = codex_launchpad::app_logic::refresh_models(&config)?;
        print_models(&cache);
        println!(
            "Model cache: {}",
            codex_launchpad::ollama::model_cache_path()?.display()
        );
        return Ok(());
    }

    if args.list_models {
        let cache = codex_launchpad::app_logic::list_models(&config)?;
        print_models(&cache);
        println!(
            "Model cache: {}",
            codex_launchpad::ollama::model_cache_path()?.display()
        );
        return Ok(());
    }

    if args.write_config_only {
        println!("{}", codex_launchpad::app_logic::write_config(&config)?);
        return Ok(());
    }

    if args.launch {
        if let Some(message) = codex_launchpad::app_logic::write_config_for_launch(&config)? {
            println!("{message}");
        }
        println!(
            "{}",
            codex_launchpad::app_logic::launch(&config, &root, &pid_file)?
        );
        return Ok(());
    }

    if args.kill {
        match codex_launchpad::app_logic::kill_codex_by_pid(&pid_file) {
            Ok(msg) => {
                println!("{}", msg);
            }
            Err(e) => {
                eprintln!("Failed to kill Codex: {e}");
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    print_help();
    Ok(())
}

async fn run_cli_async(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.to_path_buf()))
        .unwrap_or(std::env::current_dir()?);

    let config_path = args
        .config_path
        .unwrap_or_else(|| default_config_path(&root));

    let config = codex_launchpad::config::LauncherConfig::read(&config_path)?;
    let pid_file = codex_launchpad::app_logic::codex_pid_file(&config_path);

    if args.launch_wait {
        if let Some(message) = codex_launchpad::app_logic::write_config_for_launch(&config)? {
            println!("{message}");
        }
        let process =
            codex_launchpad::app_logic::launch_and_wait(&config, &root, &pid_file).await?;
        let state = process.health_check(5).await?;
        if state.api_ready {
            println!("Codex launched and API is ready!");
        } else {
            println!("Codex launched (API may need more time)");
        }
        return Ok(());
    }

    if args.health {
        let state = codex_launchpad::app_logic::health_check(&config).await?;
        println!(
            "Codex API: {}",
            if state.api_ready {
                "ready"
            } else {
                "not ready yet"
            }
        );
        return Ok(());
    }

    if let Some(_session_id) = args.session_create {
        let session = codex_launchpad::app_logic::start_session(&config).await?;
        println!("Session created: {}", session.session_id);
        return Ok(());
    }

    if let Some((session_id, msg)) = &args.session_send {
        let response = codex_launchpad::app_logic::send_message(&config, session_id, msg).await?;
        println!("{}", response.content);
        return Ok(());
    }

    if let Some(session_id) = &args.session_response {
        let response = codex_launchpad::app_logic::get_response(&config, session_id).await?;
        println!("{}", response.content);
        return Ok(());
    }

    if let Some(session_id) = &args.session_close {
        println!(
            "{}",
            codex_launchpad::app_logic::close_session(&config, session_id).await?
        );
        return Ok(());
    }

    if args.session_list {
        println!(
            "{}",
            codex_launchpad::app_logic::list_sessions(&config).await?
        );
        return Ok(());
    }

    Ok(())
}

fn default_config_path(root: &Path) -> PathBuf {
    let packaged_config = root.join("config.json");
    if packaged_config.exists() {
        packaged_config
    } else {
        PathBuf::from("config.json")
    }
}

fn print_help() {
    println!("{}", codex_launchpad::branding::APP_NAME);
    println!();
    println!("USAGE:");
    println!("    codex-launchpad [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("         --gui                      Open the GUI");
    println!("         --serve-only               Serve web UI over HTTP (no native window)");
    println!("         --port <port>              Port for --serve-only (default: random)");
    println!("         --config <path>           Path to config.json (default: auto-detect)");
    println!("         --write-config-only       Write Codex config but do not launch");
    println!("         --restore                 Restore previous Codex settings");
    println!("         --refresh-models          Refresh the Ollama model cache");
    println!("         --list-models             List cached or fetched Ollama models");
    println!("         --launch                  Launch Codex");
    println!("         --launch-wait             Launch Codex and wait for API readiness");
    println!("         --kill                    Kill the running Codex process");
    println!("         --health                  Check if Codex API is ready");
    println!("         --diagnose                Run setup and connectivity checks");
    println!("         --build-check             Rebuild stale Rust/web artifacts and stage release output");
    println!("         --session-create          Create a new ACP session");
    println!("         --session-send <id> <msg> Send a message to a session");
    println!("         --session-response <id>   Read response from a session");
    println!("         --session-close <id>      Close a session");
    println!("         --session-list            List active sessions");
    println!("         -h, --help                Show this help message");
    println!();
    println!("CONFIG:");
    println!("    Local settings live in config.json, which is gitignored.");
    println!("    Public defaults live in config.example.json.");
}

fn print_models(cache: &codex_launchpad::ollama::ModelCache) {
    println!("Ollama models from {}", cache.fetched_from);
    for model in &cache.models {
        println!("{}", model.name);
    }
}

#[derive(Debug, Default)]
struct Args {
    config_path: Option<PathBuf>,
    gui: bool,
    serve_only: bool,
    port: Option<u16>,
    write_config_only: bool,
    restore: bool,
    refresh_models: bool,
    list_models: bool,
    launch: bool,
    launch_wait: bool,
    kill: bool,
    health: bool,
    diagnose: bool,
    build_check: bool,
    session_create: Option<String>,
    session_send: Option<(String, String)>,
    session_response: Option<String>,
    session_close: Option<String>,
    session_list: bool,
    unknown_args: Vec<String>,
}

impl Args {
    fn parse() -> Self {
        let mut parsed = Self::default();
        let raw_args: Vec<String> = std::env::args_os()
            .skip(1)
            .map(|a| a.to_string_lossy().to_string())
            .collect();
        let mut i = 0;
        while i < raw_args.len() {
            let arg = &raw_args[i];
            if arg == "--config" {
                parsed.config_path = raw_args.get(i + 1).map(PathBuf::from);
                i += 2;
            } else if arg == "--gui" {
                parsed.gui = true;
                i += 1;
            } else if arg == "--serve-only" {
                parsed.serve_only = true;
                i += 1;
            } else if arg == "--port" {
                parsed.port = raw_args.get(i + 1).and_then(|p| p.parse().ok());
                i += 2;
            } else if arg == "--write-config-only" {
                parsed.write_config_only = true;
                i += 1;
            } else if arg == "--restore" {
                parsed.restore = true;
                i += 1;
            } else if arg == "--refresh-models" {
                parsed.refresh_models = true;
                i += 1;
            } else if arg == "--list-models" {
                parsed.list_models = true;
                i += 1;
            } else if arg == "--launch" {
                parsed.launch = true;
                i += 1;
            } else if arg == "--launch-wait" {
                parsed.launch_wait = true;
                i += 1;
            } else if arg == "--kill" {
                parsed.kill = true;
                i += 1;
            } else if arg == "--health" {
                parsed.health = true;
                i += 1;
            } else if arg == "--diagnose" {
                parsed.diagnose = true;
                i += 1;
            } else if arg == "--build-check" {
                parsed.build_check = true;
                i += 1;
            } else if arg == "--session-create" {
                parsed.session_create = Some(
                    raw_args
                        .get(i + 1)
                        .map(|a| a.to_string())
                        .unwrap_or_default(),
                );
                i += 2;
            } else if arg == "--session-send" {
                if i + 2 < raw_args.len() {
                    parsed.session_send =
                        Some((raw_args[i + 1].to_string(), raw_args[i + 2].to_string()));
                }
                i += 3;
            } else if arg == "--session-response" {
                parsed.session_response = Some(
                    raw_args
                        .get(i + 1)
                        .map(|a| a.to_string())
                        .unwrap_or_default(),
                );
                i += 2;
            } else if arg == "--session-close" {
                parsed.session_close = Some(
                    raw_args
                        .get(i + 1)
                        .map(|a| a.to_string())
                        .unwrap_or_default(),
                );
                i += 2;
            } else if arg == "--session-list" {
                parsed.session_list = true;
                i += 1;
            } else if arg == "-h" || arg == "--help" {
                i += 1;
            } else {
                parsed.unknown_args.push(arg.clone());
                i += 1;
            }
        }
        parsed
    }

    fn needs_async(&self) -> bool {
        self.launch_wait
            || self.health
            || self.session_create.is_some()
            || self.session_send.is_some()
            || self.session_response.is_some()
            || self.session_close.is_some()
            || self.session_list
    }

    fn has_explicit_action(&self) -> bool {
        !self.unknown_args.is_empty()
            || self.config_path.is_some()
            || self.write_config_only
            || self.restore
            || self.refresh_models
            || self.list_models
            || self.launch
            || self.launch_wait
            || self.kill
            || self.health
            || self.diagnose
            || self.build_check
            || self.session_create.is_some()
            || self.session_send.is_some()
            || self.session_response.is_some()
            || self.session_close.is_some()
            || self.session_list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_args_are_empty() {
        let args = Args::default();
        assert!(!args.gui);
        assert!(!args.serve_only);
        assert!(args.port.is_none());
        assert!(!args.write_config_only);
        assert!(!args.restore);
        assert!(!args.refresh_models);
        assert!(!args.list_models);
        assert!(!args.launch);
        assert!(!args.launch_wait);
        assert!(!args.kill);
        assert!(!args.health);
        assert!(!args.diagnose);
        assert!(!args.build_check);
        assert!(args.session_create.is_none());
        assert!(args.session_send.is_none());
        assert!(args.session_response.is_none());
        assert!(args.session_close.is_none());
        assert!(!args.session_list);
        assert!(!args.needs_async());
    }

    #[test]
    fn health_flag_needs_async() {
        let args = Args {
            health: true,
            ..Args::default()
        };
        assert!(args.needs_async());
    }
}
