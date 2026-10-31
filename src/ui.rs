//! ui.rs - Interface gráfica nativa (eframe / egui) moderna com temas Dark/Light e controles completos.

use eframe::egui::{
    self, Align, Button, Color32, ComboBox, CornerRadius, Frame, Layout, Margin,
    RichText, ScrollArea, Stroke, TextEdit, Ui, Vec2,
};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::data_manager::{get_default_provider_base_url, GTrayDataManager};
use crate::escriba::Escriba;
use crate::handler::GtrayHandler;
use crate::klisten::format_key_combination;
use crate::theme::{apply_theme, ThemeMode, UIColors};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlashType {
    Success,
    Warning,
    Danger,
}

pub struct GTrayApp {
    pub data_manager: Arc<Mutex<GTrayDataManager>>,
    pub handler: GtrayHandler,
    pub logger: Escriba,

    pub is_running: bool,
    pub keep_visible: bool,
    pub current_tab: usize,
    pub theme: ThemeMode,
    pub start_hidden: bool,
    pub hide_frames_left: u32,
    pub show_window_signal: Arc<AtomicBool>,

    // Flash message
    pub flash_message: Option<(String, FlashType, Instant)>,

    // Modal de confirmação de exclusão: (tipo, nome)
    pub confirm_delete: Option<(String, String)>,

    // Atalhos tab state
    pub selected_shortcut: String,
    pub input_shortcut_name: String,
    pub input_shortcut_key: String,
    pub input_shortcut_model: String,
    pub input_shortcut_prompt: String,
    pub input_shortcut_extract: bool,
    pub input_shortcut_comment: String,
    pub input_shortcut_enabled: bool,
    pub recording_key: bool,

    // Shortcut quick tester
    pub show_test_panel: bool,
    pub test_input: String,
    pub test_output: Arc<Mutex<Option<Result<String, String>>>>,
    pub test_is_running: Arc<AtomicBool>,

    // Modelos tab state
    pub selected_model: String,
    pub input_model_name: String,
    pub input_model_provider: String,

    // Provedores tab state
    pub selected_provider: String,
    pub input_provider_name: String,
    pub input_provider_key: String,
    pub input_provider_url: String,
    pub show_provider_key: bool,
    pub provider_test_result: Arc<Mutex<Option<Result<String, String>>>>,
    pub provider_testing: Arc<AtomicBool>,

    // Logs tab state
    pub log_filter: String,
    pub cached_logs: Vec<String>,
    pub last_log_refresh: Instant,
}

impl GTrayApp {
    pub fn new(
        data_manager: Arc<Mutex<GTrayDataManager>>,
        handler: GtrayHandler,
        logger: Escriba,
        autostart: bool,
        start_hidden: bool,
        show_window_signal: Arc<AtomicBool>,
    ) -> Self {
        let theme_str = {
            let dm = data_manager.lock().unwrap();
            dm.get_theme()
        };
        let theme = ThemeMode::from_str_name(&theme_str);

        let hide_frames_left = if start_hidden { 2 } else { 0 };

        let mut app = Self {
            data_manager,
            handler,
            logger,
            is_running: false,
            keep_visible: false,
            current_tab: 0,
            theme,
            start_hidden,
            hide_frames_left,
            show_window_signal,
            flash_message: None,
            confirm_delete: None,

            selected_shortcut: String::new(),
            input_shortcut_name: String::new(),
            input_shortcut_key: "(nenhuma)".to_string(),
            input_shortcut_model: String::new(),
            input_shortcut_prompt: String::new(),
            input_shortcut_extract: false,
            input_shortcut_comment: "#".to_string(),
            input_shortcut_enabled: true,
            recording_key: false,

            show_test_panel: false,
            test_input: String::new(),
            test_output: Arc::new(Mutex::new(None)),
            test_is_running: Arc::new(AtomicBool::new(false)),

            selected_model: String::new(),
            input_model_name: String::new(),
            input_model_provider: String::new(),

            selected_provider: String::new(),
            input_provider_name: String::new(),
            input_provider_key: String::new(),
            input_provider_url: String::new(),
            show_provider_key: false,
            provider_test_result: Arc::new(Mutex::new(None)),
            provider_testing: Arc::new(AtomicBool::new(false)),

            log_filter: String::new(),
            cached_logs: Vec::new(),
            last_log_refresh: Instant::now() - Duration::from_secs(60),
        };

        app.sync_from_data_manager();
        app.refresh_logs();

        if autostart {
            app.start_service();
        }

        app
    }

    pub fn sync_from_data_manager(&mut self) {
        let first_prov = {
            let dm = self.data_manager.lock().unwrap();
            dm.get_providers().keys().next().cloned()
        };
        if let Some(ref prov) = first_prov {
            let dm = self.data_manager.lock().unwrap();
            self.selected_provider = prov.clone();
            self.input_provider_name = prov.clone();
            self.input_provider_key = dm.get_provider_key(prov).unwrap_or_default();
            self.input_provider_url = dm
                .get_providers()
                .get(prov)
                .and_then(|p| p.base_url.clone())
                .unwrap_or_default();
        }

        let first_model = {
            let dm = self.data_manager.lock().unwrap();
            dm.get_models().keys().next().cloned()
        };
        if let Some(ref model) = first_model {
            let dm = self.data_manager.lock().unwrap();
            self.selected_model = model.clone();
            self.input_model_name = model.clone();
            self.input_model_provider = dm.get_model_provider(model).unwrap_or_default();
        }

        let first_sc = {
            let dm = self.data_manager.lock().unwrap();
            dm.get_shortcuts().keys().next().cloned()
        };
        if let Some(ref sc) = first_sc {
            self.load_shortcut_into_form(sc);
        }
    }

    pub fn toggle_theme(&mut self) {
        self.theme = match self.theme {
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
        };
        let is_dark = self.theme.is_dark();
        {
            let mut dm = self.data_manager.lock().unwrap();
            dm.set_theme(self.theme.name());
        }
        self.show_flash(
            &format!("Tema alterado para {}", if is_dark { "Escuro" } else { "Claro" }),
            FlashType::Success,
        );
    }

    fn load_shortcut_into_form(&mut self, name: &str) {
        let sc = {
            let dm = self.data_manager.lock().unwrap();
            dm.get_shortcut(name)
        };
        self.selected_shortcut = name.to_string();
        if let Some(sc) = sc {
            self.input_shortcut_name = name.to_string();
            self.input_shortcut_key = if sc.key.trim().is_empty() {
                "(nenhuma)".to_string()
            } else {
                sc.key
            };
            self.input_shortcut_model = sc.model;
            self.input_shortcut_prompt = sc.system_prompt;
            self.input_shortcut_extract = sc.extract_blocks;
            self.input_shortcut_comment = sc.comment_char;
            self.input_shortcut_enabled = sc.enabled;
        }
    }

    pub fn show_flash(&mut self, text: &str, flash_type: FlashType) {
        self.flash_message = Some((text.to_string(), flash_type, Instant::now()));
    }

    pub fn start_service(&mut self) {
        self.is_running = true;
        self.handler.start();
        self.logger.info("Serviço iniciado com sucesso.");
        self.show_flash("Serviço de Atalhos gTray Iniciado!", FlashType::Success);
    }

    pub fn stop_service(&mut self) {
        self.is_running = false;
        self.handler.stop();
        self.logger.info("Serviço pausado.");
        self.show_flash("Serviço de Atalhos gTray Pausado.", FlashType::Warning);
    }

    pub fn refresh_logs(&mut self) {
        let log_path = Path::new("logs/app.log");
        if log_path.exists() {
            if let Ok(content) = fs::read_to_string(log_path) {
                let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
                let start = if lines.len() > 100 {
                    lines.len() - 100
                } else {
                    0
                };
                let mut slice = lines[start..].to_vec();
                slice.reverse();
                self.cached_logs = slice;
            }
        } else {
            self.cached_logs.clear();
        }
        self.last_log_refresh = Instant::now();
    }

    fn handle_key_recording(&mut self, ctx: &egui::Context) {
        if !self.recording_key {
            return;
        }

        ctx.input(|i| {
            if i.key_pressed(egui::Key::Escape) {
                self.recording_key = false;
                if self.input_shortcut_key.is_empty() {
                    self.input_shortcut_key = "(nenhuma)".to_string();
                }
                return;
            }

            let mut keys = Vec::new();
            if i.modifiers.ctrl {
                keys.push("Ctrl");
            }
            if i.modifiers.alt {
                keys.push("Alt");
            }
            if i.modifiers.shift {
                keys.push("Shift");
            }
            if i.modifiers.command {
                keys.push("Win");
            }

            let has_modifiers = !keys.is_empty();

            for event in &i.raw.events {
                if let egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    ..
                } = event
                {
                    let key_name = format!("{:?}", key);
                    let is_f_key = key_name.starts_with('F')
                        && key_name[1..].chars().all(|c| c.is_ascii_digit());
                    let is_char = key_name.len() == 1;

                    if is_f_key || (has_modifiers && is_char) {
                        keys.push(&key_name);
                        let combo = format_key_combination(&keys);
                        self.input_shortcut_key = combo;
                        self.recording_key = false;
                        break;
                    }
                }
            }
        });
    }
}

impl eframe::App for GTrayApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Se a janela estiver em segundo plano/invisível e a bandeja solicitar exibição
        if self.show_window_signal.swap(false, Ordering::SeqCst) {
            self.start_hidden = false;
            self.hide_frames_left = 0;
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            crate::tray::restore_app_window();
            ctx.request_repaint();
        }
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Se a bandeja do sistema solicitou restaurar e exibir a interface
        if self.show_window_signal.swap(false, Ordering::SeqCst) {
            self.start_hidden = false;
            self.hide_frames_left = 0;
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            crate::tray::restore_app_window();
        }

        // Se configurado para iniciar apenas na bandeja do sistema (modo 2)
        if self.start_hidden {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            crate::tray::hide_app_window();
            if self.hide_frames_left > 0 {
                self.hide_frames_left -= 1;
                ctx.request_repaint_after(Duration::from_millis(50));
            } else {
                self.start_hidden = false;
            }
            return;
        }

        // Se a janela solicitou fechar pelo botão 'X', minimiza para a bandeja
        if ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            crate::tray::hide_app_window();
        }

        self.handle_key_recording(&ctx);

        // Se a janela foi minimizada (ex: pelo botão '_' da barra de título do Windows),
        // oculta a visibilidade para que não ocupe espaço na barra de tarefas, ficando apenas no systray.
        if ctx.input(|i| i.viewport().minimized == Some(true)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
            crate::tray::hide_app_window();
        }

        // Aplica o tema selecionado
        apply_theme(&ctx, self.theme);
        let colors = self.theme.colors();

        // Controla 'Manter no Topo' no Viewport do Windows
        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(if self.keep_visible {
            egui::WindowLevel::AlwaysOnTop
        } else {
            egui::WindowLevel::Normal
        }));

        // ---------------------------------------------------------
        // 1. TOP PANEL: Cabeçalho Studio + Abas de Navegação
        // ---------------------------------------------------------
        egui::Panel::top("top_header_panel")
            .frame(
                Frame::new()
                    .fill(colors.bg_panel)
                    .inner_margin(Margin::symmetric(10, 6)),
            )
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(6.0, 4.0);
                self.render_header(ui, &colors);
                ui.add_space(2.0);
                self.render_tab_bar(ui, &colors);
            });

        // ---------------------------------------------------------
        // 2. BOTTOM PANEL: Rodapé com Controles Fixos e Feedback
        // ---------------------------------------------------------
        egui::Panel::bottom("bottom_footer_panel")
            .frame(
                Frame::new()
                    .fill(colors.bg_panel)
                    .inner_margin(Margin::symmetric(10, 6)),
            )
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(6.0, 4.0);
                self.render_footer(ui, &ctx, &colors);
            });

        // ---------------------------------------------------------
        // 3. CENTRAL PANEL: Conteúdo com Scroll da Aba Selecionada
        // ---------------------------------------------------------
        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(colors.bg_panel)
                    .inner_margin(Margin::symmetric(10, 2)),
            )
            .show(ui, |ui| {
                Frame::new()
                    .fill(colors.bg_card)
                    .stroke(Stroke::new(1.0, colors.border))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::same(10))
                    .show(ui, |ui| {
                        ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                ui.spacing_mut().item_spacing = Vec2::new(6.0, 5.0);
                                match self.current_tab {
                                    0 => self.render_shortcuts_tab(ui, &colors),
                                    1 => self.render_models_tab(ui, &colors),
                                    2 => self.render_providers_tab(ui, &colors),
                                    3 => self.render_logs_tab(ui, &colors),
                                    4 => self.render_help_tab(ui, &colors),
                                    _ => {}
                                }
                            });
                    });
            });
    }
}

// -----------------------------------------------------------------
// Sub-telas / Renderers de Seções Principais
// -----------------------------------------------------------------
impl GTrayApp {
    fn render_header(&mut self, ui: &mut Ui, colors: &UIColors) {
        Frame::new()
            .fill(colors.bg_card)
            .stroke(Stroke::new(1.0, colors.border))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::symmetric(10, 6))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Logo Icon com destaque
                    Frame::new()
                        .fill(colors.accent_blue)
                        .corner_radius(CornerRadius::same(6))
                        .inner_margin(Margin::symmetric(7, 3))
                        .show(ui, |ui| {
                            ui.label(RichText::new("⚡").size(14.0).color(Color32::WHITE).strong());
                        });

                    ui.add_space(2.0);

                    // Nome e versão
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("gTray")
                                .size(14.5)
                                .strong()
                                .color(colors.text_primary),
                        );
                        ui.label(
                            RichText::new("Assistente IA para Windows")
                                .size(10.0)
                                .color(colors.text_secondary),
                        );
                    });

                    // Lado direito: Status e Botão de Tema
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let theme_icon = if self.theme.is_dark() { "☀️ Claro" } else { "🌙 Escuro" };
                        let btn_theme = Button::new(RichText::new(theme_icon).size(10.5).strong())
                            .fill(colors.bg_card_sub)
                            .stroke(Stroke::new(1.0, colors.border))
                            .corner_radius(CornerRadius::same(5));
                        if ui.add(btn_theme).on_hover_text("Alternar tema").clicked() {
                            self.toggle_theme();
                        }

                        ui.add_space(4.0);

                        let (badge_bg, badge_fg, badge_text) = if self.is_running {
                            (colors.badge_active_bg, colors.badge_active_fg, "● Ativo")
                        } else {
                            (colors.badge_paused_bg, colors.badge_paused_fg, "● Pausado")
                        };

                        Frame::new()
                            .fill(badge_bg)
                            .stroke(Stroke::new(1.0, badge_fg))
                            .corner_radius(CornerRadius::same(5))
                            .inner_margin(Margin::symmetric(8, 3))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(badge_text)
                                        .size(11.0)
                                        .strong()
                                        .color(badge_fg),
                                );
                            });
                    });
                });
            });
    }

    fn render_tab_bar(&mut self, ui: &mut Ui, colors: &UIColors) {
        let shortcuts_count = {
            let dm = self.data_manager.lock().unwrap();
            dm.get_shortcuts().len()
        };
        let models_count = {
            let dm = self.data_manager.lock().unwrap();
            dm.get_models().len()
        };
        let providers_count = {
            let dm = self.data_manager.lock().unwrap();
            dm.get_providers().len()
        };

        Frame::new()
            .fill(colors.bg_card_sub)
            .stroke(Stroke::new(1.0, colors.border_subtle))
            .corner_radius(CornerRadius::same(7))
            .inner_margin(Margin::same(3))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let avail = ui.available_width();
                    let show_counts = avail >= 430.0;
                    let tabs = [
                        (if show_counts { format!("⚡ Atalhos ({})", shortcuts_count) } else { "⚡ Atalhos".to_string() }, 0),
                        (if show_counts { format!("🤖 Modelos ({})", models_count) } else { "🤖 Modelos".to_string() }, 1),
                        (if show_counts { format!("🔑 Provedores ({})", providers_count) } else { "🔑 Provedores".to_string() }, 2),
                        ("📜 Logs".to_string(), 3),
                        ("ℹ️ Ajuda".to_string(), 4),
                    ];

                    let btn_width = ((avail - 16.0) / (tabs.len() as f32)).max(42.0);

                    for (label, idx) in tabs {
                        let is_active = self.current_tab == idx;
                        let text_color = if is_active {
                            colors.text_primary
                        } else {
                            colors.text_secondary
                        };
                        let bg_color = if is_active {
                            colors.bg_card
                        } else {
                            Color32::TRANSPARENT
                        };
                        let stroke = if is_active {
                            Stroke::new(1.0, colors.accent_blue)
                        } else {
                            Stroke::NONE
                        };

                        let btn = Button::new(RichText::new(&label).size(10.0).strong().color(text_color))
                            .fill(bg_color)
                            .stroke(stroke)
                            .corner_radius(CornerRadius::same(5))
                            .min_size(Vec2::new(btn_width, 24.0));

                        if ui.add(btn).clicked() {
                            self.current_tab = idx;
                            if idx == 3 {
                                self.refresh_logs();
                            }
                        }
                    }
                });
            });
    }

    fn render_footer(&mut self, ui: &mut Ui, ctx: &egui::Context, colors: &UIColors) {
        // Se houver confirmação de exclusão
        if let Some((kind, name)) = self.confirm_delete.clone() {
            Frame::new()
                .fill(colors.bg_card_sub)
                .stroke(Stroke::new(1.5, colors.accent_red))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("Excluir {} '{}'?", kind, name))
                                .size(11.0)
                                .strong()
                                .color(colors.accent_red),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.button(RichText::new("Cancelar").size(10.5)).clicked() {
                                self.confirm_delete = None;
                            }
                            let btn_confirm = Button::new(
                                RichText::new("Sim, Excluir").size(10.5).strong().color(Color32::WHITE),
                            )
                            .fill(colors.accent_red);
                            if ui.add(btn_confirm).clicked() {
                                self.execute_confirmed_delete(&kind, &name);
                                self.confirm_delete = None;
                            }
                        });
                    });
                });
            ui.add_space(2.0);
        }

        // Se houver mensagem flash ativa
        if let Some((msg, msg_type, time)) = &self.flash_message {
            if time.elapsed() < Duration::from_millis(3500) {
                let (bg, fg, icon) = match msg_type {
                    FlashType::Success => (
                        Color32::from_rgba_unmultiplied(16, 185, 129, 45),
                        Color32::from_rgb(52, 211, 153),
                        "✓ ",
                    ),
                    FlashType::Warning => (
                        Color32::from_rgba_unmultiplied(245, 158, 11, 45),
                        Color32::from_rgb(251, 191, 36),
                        "⚠ ",
                    ),
                    FlashType::Danger => (
                        Color32::from_rgba_unmultiplied(239, 68, 68, 45),
                        Color32::from_rgb(248, 113, 113),
                        "🗑 ",
                    ),
                };

                Frame::new()
                    .fill(bg)
                    .stroke(Stroke::new(1.0, fg))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::symmetric(8, 4))
                    .show(ui, |ui| {
                        ui.centered_and_justified(|ui| {
                            ui.label(RichText::new(format!("{}{}", icon, msg)).size(11.0).strong().color(fg));
                        });
                    });
                ui.add_space(2.0);
            }
        }

        Frame::new()
            .fill(colors.bg_card)
            .stroke(Stroke::new(1.0, colors.border))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::symmetric(10, 6))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.keep_visible, RichText::new("📌 Sempre no Topo").size(11.0));

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let active_count = {
                            let dm = self.data_manager.lock().unwrap();
                            dm.get_shortcuts()
                                .values()
                                .filter(|s| s.enabled && !s.key.is_empty() && s.key != "(nenhuma)")
                                .count()
                        };
                        ui.label(
                            RichText::new(format!("⚡ {} ativos", active_count))
                                .size(10.5)
                                .color(colors.text_muted),
                        );
                    });
                });

                ui.add_space(3.0);

                ui.horizontal(|ui| {
                    let (btn_label, btn_color) = if self.is_running {
                        ("⏸ Pausar Serviço", colors.accent_red)
                    } else {
                        ("▶ Iniciar Serviço", colors.accent_blue)
                    };

                    let btn_start = Button::new(
                        RichText::new(btn_label)
                            .size(12.0)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(btn_color)
                    .corner_radius(CornerRadius::same(5))
                    .min_size(Vec2::new(ui.available_width() * 0.68, 30.0));

                    if ui.add(btn_start).clicked() {
                        if self.is_running {
                            self.stop_service();
                        } else {
                            self.start_service();
                        }
                    }

                    let btn_min = Button::new(RichText::new("⬇ Minimizar").size(11.5))
                        .fill(colors.bg_card_sub)
                        .stroke(Stroke::new(1.0, colors.border))
                        .corner_radius(CornerRadius::same(5))
                        .min_size(Vec2::new(ui.available_width(), 30.0));

                    if ui.add(btn_min).on_hover_text("Minimiza para a bandeja do sistema (oculta da barra de tarefas)").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                        crate::tray::hide_app_window();
                        if !self.is_running {
                            self.start_service();
                        }
                    }
                });
            });
    }
}

// -----------------------------------------------------------------
// Sub-telas / Renderers de Abas
// -----------------------------------------------------------------
impl GTrayApp {
    fn execute_confirmed_delete(&mut self, kind: &str, name: &str) {
        match kind {
            "atalho" => {
                let mut dm = self.data_manager.lock().unwrap();
                dm.delete_shortcut(name);
                drop(dm);
                if self.is_running {
                    self.handler.register_shortcuts();
                }
                self.sync_from_data_manager();
                self.show_flash(&format!("Atalho '{}' excluído.", name), FlashType::Danger);
            }
            "modelo" => {
                let mut dm = self.data_manager.lock().unwrap();
                dm.delete_model(name);
                drop(dm);
                self.sync_from_data_manager();
                self.show_flash(&format!("Modelo '{}' excluído.", name), FlashType::Danger);
            }
            "provedor" => {
                let mut dm = self.data_manager.lock().unwrap();
                dm.delete_provider(name);
                drop(dm);
                self.sync_from_data_manager();
                self.show_flash(&format!("Provedor '{}' excluído.", name), FlashType::Danger);
            }
            _ => {}
        }
    }

    // -------------------------------------------------------------
    // RENDER: ABA ATALHOS
    // -------------------------------------------------------------
    fn render_shortcuts_tab(&mut self, ui: &mut Ui, colors: &UIColors) {
        let shortcuts = {
            let dm = self.data_manager.lock().unwrap();
            dm.get_shortcuts().clone()
        };
        let models = {
            let dm = self.data_manager.lock().unwrap();
            dm.get_models().clone()
        };

        // 1. SELETOR RÁPIDO EM CHIPS / CARDS HORIZONTAIS
        ui.horizontal_wrapped(|ui| {
            for (name, sc) in &shortcuts {
                let is_selected = self.selected_shortcut == *name;
                let (dot_color, _) = if sc.enabled {
                    (colors.accent_green, "Ativo")
                } else {
                    (colors.text_muted, "Inativo")
                };

                let bg = if is_selected {
                    colors.bg_card_sub
                } else {
                    colors.bg_card
                };
                let border = if is_selected {
                    Stroke::new(1.5, colors.accent_blue)
                } else {
                    Stroke::new(1.0, colors.border)
                };

                let card_resp = Frame::new()
                    .fill(bg)
                    .stroke(border)
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::symmetric(8, 4))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("●").size(9.0).color(dot_color));
                            ui.label(RichText::new(name).strong().size(12.0).color(colors.text_primary));
                            if !sc.key.is_empty() && sc.key != "(nenhuma)" {
                                ui.label(
                                    RichText::new(&sc.key)
                                        .monospace()
                                        .size(10.0)
                                        .color(colors.accent_blue),
                                );
                            }
                        });
                    });

                if card_resp.response.interact(egui::Sense::click()).clicked() {
                    self.load_shortcut_into_form(name);
                }
            }

            // Botão de Novo Atalho
            let btn_novo = Button::new(RichText::new("+ Novo Atalho").size(11.0).strong())
                .fill(colors.bg_card_sub)
                .stroke(Stroke::new(1.0, colors.accent_blue))
                .corner_radius(CornerRadius::same(6));
            if ui.add(btn_novo).clicked() {
                self.selected_shortcut.clear();
                self.input_shortcut_name.clear();
                self.input_shortcut_key = "(nenhuma)".to_string();
                self.input_shortcut_prompt.clear();
                self.input_shortcut_extract = false;
                self.input_shortcut_comment = "#".to_string();
                self.input_shortcut_enabled = true;
            }
        });

        ui.add_space(8.0);

        // 2. FORMULÁRIO DE EDIÇÃO DO ATALHO
        ui.label(
            RichText::new("NOME DO ATALHO:")
                .size(11.0)
                .strong()
                .color(colors.text_secondary),
        );
        ui.add(
            TextEdit::singleline(&mut self.input_shortcut_name)
                .hint_text("ex: melhorar_texto, python_script, resumir")
                .desired_width(ui.available_width()),
        );

        ui.add_space(4.0);

        // COMBINAÇÃO DE TECLAS / KEYCAPS
        ui.label(
            RichText::new("TECLA / COMBINAÇÃO DE ATALHO:")
                .size(11.0)
                .strong()
                .color(colors.text_secondary),
        );

        if self.recording_key {
            Frame::new()
                .fill(Color32::from_rgba_unmultiplied(245, 158, 11, 25))
                .stroke(Stroke::new(1.5, colors.accent_amber))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::symmetric(10, 8))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new("⏺ GRAVANDO: Pressione o atalho desejado no teclado (ex: F8, Ctrl+Alt+U)... Pressione Esc para cancelar.")
                            .size(11.0)
                            .strong()
                            .color(colors.accent_amber),
                    );
                });
        } else {
            ui.horizontal(|ui| {
                // Keycaps renderizados
                Frame::new()
                    .fill(colors.bg_input)
                    .stroke(Stroke::new(1.0, colors.border))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::symmetric(8, 4))
                    .show(ui, |ui| {
                        Self::render_key_combination(ui, &self.input_shortcut_key, colors);
                    });

                let record_label = if self.recording_key {
                    "Parar"
                } else {
                    "⌨ Gravar Tecla"
                };
                if ui.button(record_label).clicked() {
                    self.recording_key = !self.recording_key;
                }

                if ui.button("✕ Limpar").clicked() {
                    self.input_shortcut_key = "(nenhuma)".to_string();
                }
            });
        }

        ui.add_space(4.0);

        // MODELO DE IA ASSOCIADO
        ui.label(
            RichText::new("MODELO DE IA ASSOCIADO:")
                .size(11.0)
                .strong()
                .color(colors.text_secondary),
        );
        let selected_model_label = if self.input_shortcut_model.is_empty() {
            "(Selecione um modelo)".to_string()
        } else if let Some(m) = models.get(&self.input_shortcut_model) {
            format!("{}  •  [{}]", self.input_shortcut_model, m.provider)
        } else {
            self.input_shortcut_model.clone()
        };

        ComboBox::from_id_salt("combo_sc_models")
            .selected_text(selected_model_label)
            .width(ui.available_width() - 20.0)
            .show_ui(ui, |ui| {
                for (name, m) in &models {
                    let label = format!("{} [{}]", name, m.provider);
                    ui.selectable_value(&mut self.input_shortcut_model, name.clone(), label);
                }
            });

        ui.add_space(4.0);

        // PRÉ-COMANDO / SYSTEM PROMPT
        ui.label(
            RichText::new("INSTRUÇÃO / PAPEL DA IA (SYSTEM PROMPT):")
                .size(11.0)
                .strong()
                .color(colors.text_secondary),
        );

        ui.add(
            TextEdit::multiline(&mut self.input_shortcut_prompt)
                .hint_text("Instrução ou papel da IA (ex: Responda apenas com o texto melhorado, sem rodeios)")
                .desired_rows(3)
                .desired_width(ui.available_width()),
        );

        ui.add_space(4.0);

        // OPÇÕES AVANÇADAS: MODO PROGRAMAÇÃO & ATALHO ATIVO
        Frame::new()
            .fill(colors.bg_card_sub)
            .stroke(Stroke::new(1.0, colors.border))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::symmetric(10, 6))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.input_shortcut_extract, "Modo Programação")
                        .on_hover_text("Extrai apenas blocos de código (markdown) e comenta o restante do texto");
                    ui.add_space(10.0);
                    ui.label(RichText::new("Caractere Comentário:").size(11.0).color(colors.text_secondary));
                    ui.add(TextEdit::singleline(&mut self.input_shortcut_comment).desired_width(30.0));
                    ui.add_space(10.0);
                    ui.checkbox(&mut self.input_shortcut_enabled, "Atalho Ativo");
                });
            });

        ui.add_space(6.0);

        // SEÇÃO COLAPSÁVEL: TESTAR ATALHO INTERATIVAMENTE
        ui.collapsing("🧪 Testar este Atalho Diretamente na Interface", |ui| {
            ui.label(RichText::new("Texto de entrada para teste:").size(11.0).color(colors.text_secondary));
            ui.add(
                TextEdit::singleline(&mut self.test_input)
                    .hint_text("Digite ou cole algo aqui para enviar à IA...")
                    .desired_width(ui.available_width()),
            );

            let is_testing = self.test_is_running.load(Ordering::SeqCst);
            if is_testing {
                ui.label(RichText::new("⏳ Aguardando resposta da IA...").color(colors.accent_blue).strong());
            } else {
                if ui.button("🚀 Enviar Teste para IA").clicked() {
                    let prompt_in = self.test_input.clone();
                    let sys_prompt = self.input_shortcut_prompt.clone();
                    let model = self.input_shortcut_model.clone();
                    let extract = self.input_shortcut_extract;
                    let comment = self.input_shortcut_comment.clone();

                    let dm = self.data_manager.lock().unwrap();
                    let prov_name = dm.get_model_provider(&model).unwrap_or_default();
                    let api_key = dm.get_provider_key(&prov_name).unwrap_or_default();
                    let base_url = dm.get_provider_base_url(&prov_name);
                    drop(dm);

                    let logger = self.logger.clone();
                    let res_store = self.test_output.clone();
                    let flag = self.test_is_running.clone();

                    std::thread::spawn(move || {
                        flag.store(true, Ordering::SeqCst);
                        *res_store.lock().unwrap() = None;
                        let ia = crate::ia::IA::new(&api_key, base_url.as_deref());
                        let res = if extract {
                            match ia.responder(&prompt_in, &sys_prompt, &model, &api_key, base_url.as_deref(), 1, &logger) {
                                Ok(raw) => Ok(crate::ia::extract_code_blocks(&raw, &comment)),
                                Err(e) => Err(e),
                            }
                        } else {
                            ia.responder(&prompt_in, &sys_prompt, &model, &api_key, base_url.as_deref(), 1, &logger)
                        };
                        *res_store.lock().unwrap() = Some(res);
                        flag.store(false, Ordering::SeqCst);
                    });
                }
            }

            if let Some(res) = self.test_output.lock().unwrap().as_ref() {
                match res {
                    Ok(resp_text) => {
                        Frame::new()
                            .fill(colors.bg_input)
                            .stroke(Stroke::new(1.0, colors.accent_green))
                            .corner_radius(CornerRadius::same(6))
                            .inner_margin(Margin::same(8))
                            .show(ui, |ui| {
                                ui.label(RichText::new("Resposta da IA:").strong().color(colors.accent_green));
                                ui.label(RichText::new(resp_text).color(colors.text_primary));
                            });
                    }
                    Err(err) => {
                        Frame::new()
                            .fill(colors.bg_input)
                            .stroke(Stroke::new(1.0, colors.accent_red))
                            .corner_radius(CornerRadius::same(6))
                            .inner_margin(Margin::same(8))
                            .show(ui, |ui| {
                                ui.label(RichText::new(format!("Erro: {}", err)).color(colors.accent_red));
                            });
                    }
                }
            }
        });

        ui.add_space(8.0);

        // BOTÕES DE AÇÃO: SALVAR / EXCLUIR
        ui.horizontal(|ui| {
            let avail = ui.available_width();
            let save_w = (avail * 0.68).max(120.0);
            let btn_save = Button::new(
                RichText::new("💾 Salvar Atalho")
                    .size(12.5)
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(colors.accent_blue)
            .corner_radius(CornerRadius::same(6))
            .min_size(Vec2::new(save_w, 30.0));

            if ui.add(btn_save).clicked() {
                let name = self.input_shortcut_name.trim().to_string();
                if name.is_empty() {
                    self.show_flash("Informe o nome do atalho para salvar.", FlashType::Warning);
                    return;
                }

                let key = if self.input_shortcut_key == "(nenhuma)" {
                    String::new()
                } else {
                    self.input_shortcut_key.trim().to_string()
                };

                let mut dm = self.data_manager.lock().unwrap();
                dm.set_shortcut(
                    &name,
                    &key,
                    &self.input_shortcut_model,
                    &self.input_shortcut_prompt,
                    self.input_shortcut_extract,
                    &self.input_shortcut_comment,
                    self.input_shortcut_enabled,
                );
                drop(dm);

                self.selected_shortcut = name.clone();
                if self.is_running {
                    self.handler.register_shortcuts();
                }

                let status_txt = if self.input_shortcut_enabled { "ativo" } else { "inativo" };
                self.show_flash(
                    &format!("Atalho '{}' salvo com sucesso ({})!", name, status_txt),
                    FlashType::Success,
                );
            }

            let btn_del = Button::new(RichText::new("🗑 Excluir").size(11.5).color(Color32::WHITE))
                .fill(colors.accent_red)
                .corner_radius(CornerRadius::same(6))
                .min_size(Vec2::new(ui.available_width(), 30.0));

            if ui.add(btn_del).clicked() {
                let name = self.input_shortcut_name.trim().to_string();
                if name.is_empty() {
                    self.show_flash("Selecione um atalho para excluir.", FlashType::Warning);
                    return;
                }
                self.confirm_delete = Some(("atalho".to_string(), name));
            }
        });
    }

    // -------------------------------------------------------------
    // RENDER: ABA MODELOS
    // -------------------------------------------------------------
    fn render_models_tab(&mut self, ui: &mut Ui, colors: &UIColors) {
        let models = {
            let dm = self.data_manager.lock().unwrap();
            dm.get_models().clone()
        };
        let providers = {
            let dm = self.data_manager.lock().unwrap();
            dm.get_providers().keys().cloned().collect::<Vec<_>>()
        };

        // Chips dos Modelos Existentes
        ui.label(
            RichText::new("MODELOS CADASTRADOS:")
                .size(11.0)
                .strong()
                .color(colors.text_secondary),
        );

        ui.horizontal_wrapped(|ui| {
            for (name, m) in &models {
                let is_selected = self.selected_model == *name;
                let bg = if is_selected {
                    colors.bg_card_sub
                } else {
                    colors.bg_card
                };
                let border = if is_selected {
                    Stroke::new(1.5, colors.accent_blue)
                } else {
                    Stroke::new(1.0, colors.border)
                };

                let card_resp = Frame::new()
                    .fill(bg)
                    .stroke(border)
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::symmetric(8, 4))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🤖").size(10.0));
                            ui.label(RichText::new(name).strong().size(12.0).color(colors.text_primary));
                            ui.label(
                                RichText::new(format!("[{}]", m.provider))
                                    .size(10.0)
                                    .color(colors.text_secondary),
                            );
                        });
                    });

                if card_resp.response.interact(egui::Sense::click()).clicked() {
                    self.selected_model = name.clone();
                    self.input_model_name = name.clone();
                    self.input_model_provider = m.provider.clone();
                }
            }

            let btn_novo = Button::new(RichText::new("+ Novo Modelo").size(11.0).strong())
                .fill(colors.bg_card_sub)
                .stroke(Stroke::new(1.0, colors.accent_blue))
                .corner_radius(CornerRadius::same(6));
            if ui.add(btn_novo).clicked() {
                self.selected_model.clear();
                self.input_model_name.clear();
                self.input_model_provider.clear();
            }
        });

        ui.add_space(8.0);

        // Presets Rápidos de Modelos Populares
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Adicionar Rápido:").size(10.0).color(colors.text_muted));
            let presets = [
                ("deepseek-chat", "deepseek"),
                ("deepseek-reasoner", "deepseek"),
                ("gpt-4o-mini", "openai"),
                ("claude-3-5-sonnet", "anthropic"),
                ("gemini-1.5-flash", "gemini"),
                ("llama-3.3-70b", "groq"),
            ];
            for (m_name, p_name) in presets {
                if ui.small_button(format!("+ {}", m_name)).clicked() {
                    self.input_model_name = m_name.to_string();
                    self.input_model_provider = p_name.to_string();
                }
            }
        });

        ui.add_space(8.0);

        // Formulário de Edição de Modelo
        ui.label(
            RichText::new("NOME DO MODELO:")
                .size(11.0)
                .strong()
                .color(colors.text_secondary),
        );
        ui.add(
            TextEdit::singleline(&mut self.input_model_name)
                .hint_text("ex: gpt-4o-mini, deepseek-chat, claude-3-5-sonnet")
                .desired_width(ui.available_width()),
        );

        ui.add_space(4.0);

        ui.label(
            RichText::new("PROVEDOR ASSOCIADO:")
                .size(11.0)
                .strong()
                .color(colors.text_secondary),
        );
        ComboBox::from_id_salt("combo_model_provider")
            .selected_text(if self.input_model_provider.is_empty() {
                "(Selecione um provedor)".to_string()
            } else {
                self.input_model_provider.clone()
            })
            .width(ui.available_width() - 20.0)
            .show_ui(ui, |ui| {
                for p in &providers {
                    ui.selectable_value(&mut self.input_model_provider, p.clone(), p);
                }
            });

        ui.add_space(12.0);

        // Ações: Salvar / Excluir
        ui.horizontal(|ui| {
            let avail = ui.available_width();
            let save_w = (avail * 0.68).max(120.0);
            let btn_save = Button::new(
                RichText::new("💾 Salvar Modelo")
                    .size(12.5)
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(colors.accent_blue)
            .corner_radius(CornerRadius::same(6))
            .min_size(Vec2::new(save_w, 30.0));

            if ui.add(btn_save).clicked() {
                let name = self.input_model_name.trim().to_string();
                let provider = self.input_model_provider.trim().to_string();
                if name.is_empty() {
                    self.show_flash("Informe o nome do modelo para salvar.", FlashType::Warning);
                    return;
                }
                if provider.is_empty() {
                    self.show_flash("Selecione um provedor associado ao modelo.", FlashType::Warning);
                    return;
                }

                let mut dm = self.data_manager.lock().unwrap();
                dm.set_model(&name, &provider);
                drop(dm);

                self.selected_model = name.clone();
                self.show_flash(&format!("Modelo '{}' salvo com sucesso!", name), FlashType::Success);
            }

            let btn_del = Button::new(RichText::new("🗑 Excluir").size(11.5).color(Color32::WHITE))
                .fill(colors.accent_red)
                .corner_radius(CornerRadius::same(6))
                .min_size(Vec2::new(ui.available_width(), 30.0));

            if ui.add(btn_del).clicked() {
                let name = self.input_model_name.trim().to_string();
                if name.is_empty() {
                    self.show_flash("Selecione um modelo para excluir.", FlashType::Warning);
                    return;
                }
                self.confirm_delete = Some(("modelo".to_string(), name));
            }
        });
    }

    // -------------------------------------------------------------
    // RENDER: ABA PROVEDORES
    // -------------------------------------------------------------
    fn render_providers_tab(&mut self, ui: &mut Ui, colors: &UIColors) {
        let providers = {
            let dm = self.data_manager.lock().unwrap();
            dm.get_providers().clone()
        };

        // Chips dos Provedores Existentes
        ui.label(
            RichText::new("PROVEDORES CADASTRADOS:")
                .size(11.0)
                .strong()
                .color(colors.text_secondary),
        );

        ui.horizontal_wrapped(|ui| {
            for (name, p) in &providers {
                let is_selected = self.selected_provider == *name;
                let bg = if is_selected {
                    colors.bg_card_sub
                } else {
                    colors.bg_card
                };
                let border = if is_selected {
                    Stroke::new(1.5, colors.accent_blue)
                } else {
                    Stroke::new(1.0, colors.border)
                };

                let key_status = if p.api_key.trim().is_empty() {
                    "⚠ Sem chave"
                } else {
                    "✓ Configurada"
                };

                let card_resp = Frame::new()
                    .fill(bg)
                    .stroke(border)
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::symmetric(8, 4))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🔑").size(10.0));
                            ui.label(RichText::new(name).strong().size(12.0).color(colors.text_primary));
                            ui.label(RichText::new(key_status).size(10.0).color(if p.api_key.trim().is_empty() {
                                colors.accent_amber
                            } else {
                                colors.accent_green
                            }));
                        });
                    });

                if card_resp.response.interact(egui::Sense::click()).clicked() {
                    self.selected_provider = name.clone();
                    self.input_provider_name = name.clone();
                    self.input_provider_key = p.api_key.clone();
                    self.input_provider_url = p.base_url.clone().unwrap_or_default();
                    *self.provider_test_result.lock().unwrap() = None;
                }
            }

            let btn_novo = Button::new(RichText::new("+ Novo Provedor").size(11.0).strong())
                .fill(colors.bg_card_sub)
                .stroke(Stroke::new(1.0, colors.accent_blue))
                .corner_radius(CornerRadius::same(6));
            if ui.add(btn_novo).clicked() {
                self.selected_provider.clear();
                self.input_provider_name.clear();
                self.input_provider_key.clear();
                self.input_provider_url.clear();
                *self.provider_test_result.lock().unwrap() = None;
            }
        });

        ui.add_space(8.0);

        // Presets Rápidos de Provedores
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Preencher Provedor:").size(10.0).color(colors.text_muted));
            let prov_presets = [
                ("openai", "https://api.openai.com/v1"),
                ("deepseek", "https://api.deepseek.com"),
                ("groq", "https://api.groq.com/openai/v1"),
                ("anthropic", "https://api.anthropic.com/v1"),
                ("ollama", "http://localhost:11434/v1"),
                ("openrouter", "https://openrouter.ai/api/v1"),
            ];
            for (p_name, p_url) in prov_presets {
                if ui.small_button(p_name).clicked() {
                    self.input_provider_name = p_name.to_string();
                    self.input_provider_url = p_url.to_string();
                }
            }
        });

        ui.add_space(8.0);

        // Formulário de Edição de Provedor
        ui.label(
            RichText::new("NOME DO PROVEDOR:")
                .size(11.0)
                .strong()
                .color(colors.text_secondary),
        );
        ui.add(
            TextEdit::singleline(&mut self.input_provider_name)
                .hint_text("ex: openai, deepseek, groq, ollama")
                .desired_width(ui.available_width()),
        );

        ui.add_space(4.0);

        ui.label(
            RichText::new("CHAVE DE API (API KEY):")
                .size(11.0)
                .strong()
                .color(colors.text_secondary),
        );
        ui.horizontal(|ui| {
            let edit = TextEdit::singleline(&mut self.input_provider_key)
                .password(!self.show_provider_key)
                .hint_text("sk-...")
                .desired_width(ui.available_width() - 40.0);
            ui.add(edit);

            let eye_label = if self.show_provider_key { "🔒" } else { "👁" };
            if ui.button(eye_label).on_hover_text("Mostrar/Ocultar chave").clicked() {
                self.show_provider_key = !self.show_provider_key;
            }
        });

        ui.add_space(4.0);

        ui.label(
            RichText::new("URL BASE DA API (OPCIONAL):")
                .size(11.0)
                .strong()
                .color(colors.text_secondary),
        );
        ui.add(
            TextEdit::singleline(&mut self.input_provider_url)
                .hint_text("ex: https://api.deepseek.com (automático se conhecido)")
                .desired_width(ui.available_width()),
        );

        ui.add_space(6.0);

        // BOTÃO: TESTAR CONEXÃO COM A API
        let is_testing_prov = self.provider_testing.load(Ordering::SeqCst);
        if is_testing_prov {
            ui.label(
                RichText::new("⏳ Testando comunicação com o servidor...")
                    .color(colors.accent_blue)
                    .strong(),
            );
        } else {
            let btn_test = Button::new(RichText::new("⚡ Testar Conexão com a API").size(11.0))
                .fill(colors.bg_card_sub)
                .stroke(Stroke::new(1.0, colors.border));

            if ui.add(btn_test).clicked() {
                let name = self.input_provider_name.trim().to_string();
                let key = self.input_provider_key.trim().to_string();
                let url = self.input_provider_url.trim().to_string();
                let flag = self.provider_testing.clone();
                let res_store = self.provider_test_result.clone();

                std::thread::spawn(move || {
                    flag.store(true, Ordering::SeqCst);
                    *res_store.lock().unwrap() = None;

                    let base = if !url.is_empty() {
                        url
                    } else {
                        get_default_provider_base_url(&name)
                            .unwrap_or("https://api.openai.com/v1")
                            .to_string()
                    };

                    let endpoint = if base.ends_with('/') {
                        format!("{}models", base)
                    } else {
                        format!("{}/models", base)
                    };

                    let mut req = ureq::get(&endpoint).timeout(Duration::from_secs(5));
                    if !key.is_empty() {
                        req = req.set("Authorization", &format!("Bearer {}", key));
                    }

                    let outcome = match req.call() {
                        Ok(resp) => {
                            let status = resp.status();
                            Ok(format!("Conexão bem-sucedida! Servidor respondeu com HTTP {}.", status))
                        }
                        Err(ureq::Error::Status(code, _)) => {
                            if code == 401 || code == 403 {
                                Err(format!("Erro {}: Chave de API inválida ou não autorizada.", code))
                            } else {
                                Ok(format!("Servidor respondeu HTTP {} (endpoint alcançado).", code))
                            }
                        }
                        Err(e) => Err(format!("Falha na conexão: {}", e)),
                    };

                    *res_store.lock().unwrap() = Some(outcome);
                    flag.store(false, Ordering::SeqCst);
                });
            }
        }

        // Exibe resultado do teste de conexão
        if let Some(res) = self.provider_test_result.lock().unwrap().as_ref() {
            ui.add_space(2.0);
            match res {
                Ok(msg) => {
                    ui.label(RichText::new(format!("✓ {}", msg)).color(colors.accent_green).strong());
                }
                Err(err) => {
                    ui.label(RichText::new(format!("✕ {}", err)).color(colors.accent_red).strong());
                }
            }
        }

        ui.add_space(10.0);

        // Ações: Salvar / Excluir
        ui.horizontal(|ui| {
            let avail = ui.available_width();
            let save_w = (avail * 0.68).max(120.0);
            let btn_save = Button::new(
                RichText::new("💾 Salvar Provedor")
                    .size(12.5)
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(colors.accent_blue)
            .corner_radius(CornerRadius::same(6))
            .min_size(Vec2::new(save_w, 30.0));

            if ui.add(btn_save).clicked() {
                let name = self.input_provider_name.trim().to_string();
                let key = self.input_provider_key.trim().to_string();
                let url = self.input_provider_url.trim().to_string();
                if name.is_empty() {
                    self.show_flash("Informe o nome do provedor para salvar.", FlashType::Warning);
                    return;
                }

                let url_opt = if url.is_empty() {
                    None
                } else {
                    Some(url.as_str())
                };

                let mut dm = self.data_manager.lock().unwrap();
                dm.set_provider(&name, &key, url_opt);
                drop(dm);

                self.selected_provider = name.clone();
                self.show_flash(&format!("Provedor '{}' salvo com sucesso!", name), FlashType::Success);
            }

            let btn_del = Button::new(RichText::new("🗑 Excluir").size(11.5).color(Color32::WHITE))
                .fill(colors.accent_red)
                .corner_radius(CornerRadius::same(6))
                .min_size(Vec2::new(ui.available_width(), 30.0));

            if ui.add(btn_del).clicked() {
                let name = self.input_provider_name.trim().to_string();
                if name.is_empty() {
                    self.show_flash("Selecione um provedor para excluir.", FlashType::Warning);
                    return;
                }
                self.confirm_delete = Some(("provedor".to_string(), name));
            }
        });
    }

    // -------------------------------------------------------------
    // RENDER: ABA LOGS & HISTÓRICO
    // -------------------------------------------------------------
    fn render_logs_tab(&mut self, ui: &mut Ui, colors: &UIColors) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("HISTÓRICO DE ATIVIDADES E LOGS:")
                    .size(11.0)
                    .strong()
                    .color(colors.text_secondary),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("🗑 Limpar").clicked() {
                    let _ = fs::write("logs/app.log", "");
                    self.refresh_logs();
                    self.show_flash("Logs apagados com sucesso.", FlashType::Success);
                }
                if ui.button("📂 Abrir Pasta").clicked() {
                    let _ = std::process::Command::new("explorer").arg("logs").spawn();
                }
                if ui.button("🔄 Atualizar").clicked() {
                    self.refresh_logs();
                }
            });
        });

        ui.add_space(4.0);

        ui.add(
            TextEdit::singleline(&mut self.log_filter)
                .hint_text("Filtrar logs por palavra-chave...")
                .desired_width(ui.available_width()),
        );

        ui.add_space(4.0);

        Frame::new()
            .fill(colors.bg_input)
            .stroke(Stroke::new(1.0, colors.border))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::same(8))
            .show(ui, |ui| {
                let filter = self.log_filter.to_lowercase();
                let matching_logs: Vec<&String> = self
                    .cached_logs
                    .iter()
                    .filter(|l| filter.is_empty() || l.to_lowercase().contains(&filter))
                    .collect();

                if matching_logs.is_empty() {
                    ui.label(RichText::new("Nenhum registro de log encontrado.").color(colors.text_muted));
                } else {
                    for line in matching_logs {
                        let line_color = if line.contains("[ERROR]") {
                            colors.accent_red
                        } else if line.contains("[WARN]") {
                            colors.accent_amber
                        } else if line.contains("[INFO]") {
                            colors.text_primary
                        } else {
                            colors.text_secondary
                        };

                        ui.label(RichText::new(line).monospace().size(11.0).color(line_color));
                    }
                }
            });
    }

    // -------------------------------------------------------------
    // RENDER: ABA AJUDA & GUIA RÁPIDO
    // -------------------------------------------------------------
    fn render_help_tab(&mut self, ui: &mut Ui, colors: &UIColors) {
        ui.label(
            RichText::new("COMO UTILIZAR O GTRAY:")
                .size(13.0)
                .strong()
                .color(colors.text_primary),
        );

        ui.add_space(4.0);

        let steps = [
            ("1. Configurar Provedor e Chave", "Na aba 'Provedores', cadastre sua chave de API (OpenAI, DeepSeek, Groq, Ollama, etc.)."),
            ("2. Cadastrar ou Escolher um Atalho", "Na aba 'Atalhos', escolha uma tecla global (ex: F8, F9, Ctrl+Alt+U) e vincule ao modelo de sua preferência."),
            ("3. Iniciar o Serviço", "Clique em '▶ Iniciar Serviço'. O assistente fica ativo na bandeja do Windows em segundo plano."),
            ("4. Usar em Qualquer Programa!", "Selecione qualquer texto (Word, VS Code, Navegador, Bloco de Notas) e pressione o atalho configurado. A IA processará e substituirá o texto automaticamente!"),
        ];

        for (title, desc) in steps {
            Frame::new()
                .fill(colors.bg_card_sub)
                .stroke(Stroke::new(1.0, colors.border))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::same(8))
                .show(ui, |ui| {
                    ui.label(RichText::new(title).strong().size(12.0).color(colors.accent_blue));
                    ui.label(RichText::new(desc).size(11.0).color(colors.text_secondary));
                });
            ui.add_space(4.0);
        }

        ui.add_space(6.0);
        ui.label(
            RichText::new("📁 Arquivos de Configuração:")
                .size(11.0)
                .strong()
                .color(colors.text_secondary),
        );
        ui.label(RichText::new("• Dados: assets/gtray_data.json").monospace().size(10.0).color(colors.text_muted));
        ui.label(RichText::new("• Logs: logs/app.log").monospace().size(10.0).color(colors.text_muted));
    }

    // -------------------------------------------------------------
    // HELPER: RENDERIZADOR DE COMBINAÇÃO DE TECLAS (KEYCAPS TÁTEIS)
    // -------------------------------------------------------------
    fn render_key_combination(ui: &mut Ui, combo: &str, colors: &UIColors) {
        let trimmed = combo.trim();
        if trimmed.is_empty() || trimmed == "(nenhuma)" {
            ui.label(
                RichText::new("(nenhuma tecla gravada)")
                    .size(11.0)
                    .color(colors.text_muted),
            );
            return;
        }

        let parts: Vec<&str> = trimmed.split('+').collect();
        for (idx, key) in parts.iter().enumerate() {
            if idx > 0 {
                ui.label(RichText::new("+").size(11.0).strong().color(colors.text_secondary));
            }

            Frame::new()
                .fill(colors.keycap_bg)
                .stroke(Stroke::new(1.0, colors.keycap_border))
                .corner_radius(CornerRadius::same(4))
                .inner_margin(Margin::symmetric(6, 2))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(key.trim())
                            .size(11.0)
                            .monospace()
                            .strong()
                            .color(colors.keycap_text),
                    );
                });
        }
    }
}
