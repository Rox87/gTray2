#![windows_subsystem = "windows"]
//! main.rs - Ponto de entrada do gTray em Rust.

use eframe::egui::Vec2;
use std::env;
use std::fs;
use std::path::Path;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use gtray::data_manager::GTrayDataManager;
use gtray::escriba::Escriba;
use gtray::handler::GtrayHandler;
use gtray::tray::{TrayEvent, TrayIcon};
use gtray::ui::GTrayApp;

fn kill_previous_instances(logger: &Escriba) {
    let current_pid = std::process::id();
    logger.info(&format!("Verificando instâncias anteriores. PID atual: {}", current_pid));

    // 1. Finaliza outras instâncias em execução do gtray.exe
    let mut cmd = Command::new("taskkill");
    cmd.args(["/f", "/im", "gtray.exe", "/fi", &format!("PID ne {}", current_pid)]);
    #[cfg(windows)]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    let output = cmd.output();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        if !stdout.trim().is_empty() && !stdout.contains("Nenhum processo") && !stdout.contains("No tasks") {
            logger.info(&format!("taskkill: {}", stdout.trim()));
        }
    }

    // 2. Garante encerramento por lastpid.txt garantindo que seja apenas gtray.exe
    let pid_files = [
        Path::new("cmd").join("lastpid.txt"),
        Path::new("bin").join("cmd").join("lastpid.txt"),
        Path::new("..").join("cmd").join("lastpid.txt"),
    ];
    for lastpid_path in &pid_files {
        if let Ok(content) = fs::read_to_string(lastpid_path) {
            let prev_pid = content.trim();
            if !prev_pid.is_empty() && prev_pid != current_pid.to_string() {
                let mut cmd2 = Command::new("taskkill");
                cmd2.args(["/f", "/fi", "IMAGENAME eq gtray.exe", "/PID", prev_pid]);
                #[cfg(windows)]
                cmd2.creation_flags(0x08000000); // CREATE_NO_WINDOW
                let _ = cmd2.output();
            }
        }
    }

    // 3. Atualiza os lastpid.txt existentes com o PID atual
    if Path::new("cmd").exists() {
        let _ = fs::write(Path::new("cmd").join("lastpid.txt"), current_pid.to_string());
    }
    if Path::new("bin").join("cmd").exists() {
        let _ = fs::write(Path::new("bin").join("cmd").join("lastpid.txt"), current_pid.to_string());
    }

    // Pausa breve para o Windows liberar os atalhos globais registrados anteriormente
    std::thread::sleep(std::time::Duration::from_millis(200));
}

fn main() -> eframe::Result<()> {
    let logger = Escriba::default();
    let logger_panic = logger.clone();
    std::panic::set_hook(Box::new(move |info| {
        logger_panic.error(&format!("PANIC: {}", info));
    }));

    logger.info("Ponto de entrada main() atingido.");
    kill_previous_instances(&logger);
    logger.info("kill_previous_instances concluido.");

    logger.log_header("Iniciando gTray (Rust)");

    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("");

    let is_tray_mode = mode == "2"
        || args.iter().skip(1).any(|a| a == "2" || a == "-2" || a == "/2" || a == "--tray" || a == "tray" || a == "background" || a == "bg");
    let is_mode_1 = mode == "1"
        || args.iter().skip(1).any(|a| a == "1" || a == "-1" || a == "/1");

    let (visible, autostart) = if is_tray_mode {
        logger.info("Inicialização: Modo 2 (Segundo plano na bandeja do sistema).");
        (false, true)
    } else if is_mode_1 {
        logger.info("Inicialização: Modo 1 (Interface Gráfica + Serviço Ativo).");
        (true, true)
    } else {
        logger.info("Inicialização: Interface Gráfica de Configurações.");
        (true, false)
    };

    let start_hidden = !visible;

    let data_manager = Arc::new(Mutex::new(GTrayDataManager::default()));
    let handler = GtrayHandler::new(data_manager.clone(), logger.clone());

    let show_window_signal = Arc::new(AtomicBool::new(false));
    let show_signal_tray = show_window_signal.clone();

    // Inicializa ícone na bandeja do Windows
    let _tray = TrayIcon::new(move |event| match event {
        TrayEvent::ShowUI => {
            show_signal_tray.store(true, Ordering::SeqCst);
            gtray::tray::restore_app_window();
        }
        TrayEvent::Exit => {
            std::process::exit(0);
        }
    });

    let app_creator = {
        let dm = data_manager.clone();
        let h = handler.clone();
        let l = logger.clone();
        let show_sig = show_window_signal.clone();
        Box::new(move |_cc: &eframe::CreationContext| {
            Ok(Box::new(GTrayApp::new(dm, h, l, autostart, start_hidden, show_sig)) as Box<dyn eframe::App>)
        })
    };

    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_title("gTray - Assistente IA")
        .with_inner_size(Vec2::new(460.0, 640.0))
        .with_min_inner_size(Vec2::new(380.0, 480.0))
        .with_resizable(true)
        .with_visible(visible);

    if let Ok(bytes) = fs::read("assets/favicon.png") {
        if let Ok(img) = image::load_from_memory(&bytes) {
            let rgba = img.into_rgba8();
            let (width, height) = rgba.dimensions();
            viewport = viewport.with_icon(eframe::egui::IconData {
                rgba: rgba.into_raw(),
                width,
                height,
            });
        }
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    logger.info("Chamando eframe::run_native...");
    let res = eframe::run_native("gTray", native_options, app_creator);
    if let Err(ref e) = res {
        logger.error(&format!("eframe::run_native retornou erro: {:?}", e));
    } else {
        logger.info("eframe::run_native encerrou com Ok(()).");
    }
    res
}
