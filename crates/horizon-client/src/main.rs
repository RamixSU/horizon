//! Horizon: native Rust client for Minecraft Java Edition 26.3 gameplay.
#![allow(unsafe_code)]
mod game;

use horizon_client::{crash, logging};

/// The mod API level this game serves, kept in the program for the launcher
/// to read without running it (`horizon_mods::api_level::MARK`). `main`
/// reads it, so the linker keeps it.
static MOD_API_LEVEL: [u8; 27] = horizon_mods::api_level::MARK;

fn main() {
    std::hint::black_box(&MOD_API_LEVEL);
    #[cfg(target_os = "ios")]
    {
        let home = logging::game_home();
        let _ = std::fs::create_dir_all(&home);
        let _ = std::fs::create_dir_all(home.join("resources"));
        let _ = std::env::set_current_dir(&home);
        // SAFETY: Called early at startup before any threads are spawned.
        unsafe {
            std::env::set_var("HORIZON_HOME", &home);
            if std::env::var_os("HORIZON_RESOURCES").is_none() {
                std::env::set_var("HORIZON_RESOURCES", home.join("resources"));
            }
        }
    }
    horizon_client::telemetry::mark_start();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let game_args = args.iter().take_while(|a| *a != "--dedicated");
    let help = game_args.clone().any(|a| a == "--help" || a == "-h");
    // The dedicated server logs in its own folder, as vanilla's does; the game in its home.
    let (home, side, thread) = if args.iter().any(|a| a == "--dedicated") {
        (std::path::PathBuf::from("."), "server", "Server thread")
    } else {
        (logging::game_home(), "client", "Render thread")
    };
    if !help {
        logging::init(&home, thread);
        // The log's first line names the build, so a bug report that carries the log says which one it was.
        log::info!("Horizon {}", crash::version());
        crash::install(home.clone(), side);

        #[cfg(target_os = "ios")]
        {
            let res_dir = home.join("resources");
            let zip_path = home.join("resources.zip");
            if zip_path.is_file() && !res_dir.join("facts.json").is_file() {
                log::info!("Horizon iOS: Found resources.zip, extracting to {}...", res_dir.display());
                match std::fs::File::open(&zip_path) {
                    Ok(file) => match zip::ZipArchive::new(file) {
                        Ok(mut archive) => {
                            let mut ok = true;
                            for i in 0..archive.len() {
                                let mut entry = match archive.by_index(i) {
                                    Ok(e) => e,
                                    Err(err) => {
                                        log::error!("Failed to read zip entry {i}: {err}");
                                        ok = false;
                                        break;
                                    }
                                };
                                let Some(rel_path) = entry.enclosed_name().map(|p| p.to_path_buf()) else {
                                    continue;
                                };
                                let target = res_dir.join(rel_path);
                                if entry.is_dir() {
                                    let _ = std::fs::create_dir_all(&target);
                                } else {
                                    if let Some(parent) = target.parent() {
                                        let _ = std::fs::create_dir_all(parent);
                                    }
                                    match std::fs::File::create(&target) {
                                        Ok(mut out) => {
                                            if let Err(e) = std::io::copy(&mut entry, &mut out) {
                                                log::error!("Failed to extract {}: {e}", target.display());
                                                ok = false;
                                                break;
                                            }
                                        }
                                        Err(e) => {
                                            log::error!("Failed to create {}: {e}", target.display());
                                            ok = false;
                                            break;
                                        }
                                    }
                                }
                            }
                            if ok {
                                log::info!("Horizon iOS: resources.zip successfully extracted!");
                                let _ = std::fs::rename(&zip_path, home.join("resources.zip.extracted"));
                            }
                        }
                        Err(e) => log::error!("Failed to open resources.zip archive: {e}"),
                    },
                    Err(e) => log::error!("Failed to open resources.zip file: {e}"),
                }
            }

            if !res_dir.join("facts.json").is_file() {
                log::error!(
                    "Horizon iOS: GAME RESOURCES MISSING in {}! Place resources/ or resources.zip in the Horizon folder (via Files app).",
                    res_dir.display()
                );
            }
        }
    }
    if let Err(error) = game::run() {
        log::error!("Horizon: {error}");
        if !help {
            crash::report_error(&home, side, &*error);
        }
        // Started from Explorer, the console window closes with the game:
        // keep the message on screen until it is read.
        #[cfg(windows)]
        if std::io::IsTerminal::is_terminal(&std::io::stdin()) {
            eprintln!("Press Enter to close.");
            let _ = std::io::stdin().read_line(&mut String::new());
        }
        std::process::exit(1);
    }
    logging::flush();
}
