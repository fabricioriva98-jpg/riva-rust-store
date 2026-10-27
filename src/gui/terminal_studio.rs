//! Interface gráfica do Terminal Dedicado e IDE do Rust e do Espelho de Editores Externos (VS Code & Google Antigravity).
//! Fornece um ambiente completo de desenvolvimento 100% Rust independente, e uma opção separada para espelhamento em IDEs externas.

use super::{draw_pill, BgEvent, StoreGui};
use crate::brand;
use crate::ide;
use crate::terminal_studio;
use eframe::egui::{self, Color32, CornerRadius, RichText, Stroke};
use std::sync::Arc;

impl StoreGui {
    /// Inicia a execução de um comando no terminal integrado em segundo plano sem travar a interface.
    pub(super) fn spawn_terminal_cmd(&mut self, cmd_line: &str) {
        let trimmed = cmd_line.trim().to_string();
        if trimmed.is_empty() || self.terminal_is_running {
            return;
        }

        self.terminal_is_running = true;
        self.terminal_cancel.store(false, std::sync::atomic::Ordering::Relaxed);

        if self.terminal_cmd_history.last().map(|s| s.as_str()) != Some(&trimmed) {
            self.terminal_cmd_history.push(trimmed.clone());
        }
        self.terminal_cmd_history_idx = None;

        // Adiciona o comando no histórico visível do terminal
        self.terminal_logs.push((
            terminal_studio::TerminalLineKind::Command,
            format!("$ {trimmed}"),
        ));

        let dir = self.terminal_working_dir.clone();
        let events = Arc::clone(&self.bg_events);
        let cancel = Arc::clone(&self.terminal_cancel);

        std::thread::spawn(move || {
            let out = terminal_studio::run_terminal_command(&dir, &trimmed, &cancel);
            events.lock().unwrap().push(BgEvent::TerminalCommandFinished(out));
        });

        self.terminal_cmd_input.clear();
    }

    /// Avalia uma expressão no avaliador (compila e roda em segundo plano, com tempo máximo).
    fn spawn_terminal_eval(&mut self, expr: String) {
        if self.terminal_is_running {
            return;
        }
        self.terminal_is_running = true;
        self.terminal_cancel.store(false, std::sync::atomic::Ordering::Relaxed);
        self.terminal_logs.push((terminal_studio::TerminalLineKind::Command, format!(">> eval: {}", expr.trim())));
        let events = Arc::clone(&self.bg_events);
        let cancel = Arc::clone(&self.terminal_cancel);
        std::thread::spawn(move || {
            let out = terminal_studio::eval_rust_expression(&expr, &cancel);
            events.lock().unwrap().push(BgEvent::TerminalCommandFinished(out));
        });
    }

    /// Detecta a toolchain Rust e os editores em segundo plano.
    pub(super) fn spawn_dev_env_detection(&mut self) {
        if self.dev_env_detecting {
            return;
        }
        self.dev_env_detecting = true;
        let events = Arc::clone(&self.bg_events);
        std::thread::spawn(move || {
            let env = terminal_studio::detect_rust_env();
            let ides = ide::detect_ides();
            events.lock().unwrap().push(BgEvent::DevEnvDetected { env, ides });
        });
    }

    // ─────────────────────────────────────────────────────────────────────────
    // 1. TELA DEDICADA: TERMINAL & IDE DEDICADA DO RUST (100% RUST)
    // ─────────────────────────────────────────────────────────────────────────

    /// Renderiza a IDE e Terminal Dedicado do Rust (ambiente autônomo, nativo e isolado).
    pub(super) fn render_rust_studio(&mut self, ui: &mut egui::Ui) {
        if self.terminal_working_dir.as_os_str().is_empty() {
            self.terminal_working_dir = terminal_studio::default_projects_dir();
            self.terminal_dir_input = self.terminal_working_dir.to_string_lossy().to_string();
        }

        // ─── CABEÇALHO DEDICADO DO RUST STUDIO ─────────────────────────────
        ui.horizontal(|ui| {
            brand::logo_badge(ui, 18.0);
            ui.add_space(6.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(self.tr("⚡ Terminal & IDE Dedicada do Rust"))
                            .size(20.0)
                            .strong()
                            .color(Color32::WHITE),
                    );
                    draw_pill(ui, &self.tr("Ambiente 100% Rust"), Color32::from_rgb(220, 105, 30));
                });
                ui.label(
                    RichText::new(self.tr(
                        "Ambiente autônomo e nativo para programar, compilar, testar e executar projetos Rust diretamente na RRS.",
                    ))
                    .size(12.5)
                    .color(Color32::from_rgb(145, 160, 185)),
                );
            });
        });

        ui.add_space(8.0);

        // ─── BARRA DA TOOLCHAIN RUST DO SISTEMA ────────────────────────────
        if self.rust_env_info.is_none() {
            self.spawn_dev_env_detection();
            ui.label(RichText::new(self.tr("⏳ Detectando a toolchain Rust e os editores...")).color(Color32::GRAY));
        }

        let mut refresh_env = false;
        if let Some(env_info) = self.rust_env_info.clone() {
            egui::Frame::NONE
                .fill(Color32::from_rgb(18, 24, 34))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(36, 48, 70)))
                .corner_radius(CornerRadius::same(8))
                .inner_margin(egui::Margin::symmetric(12, 9))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(self.tr("🦀 Toolchain Rust do Sistema:")).strong().color(Color32::WHITE));
                        ui.add_space(6.0);

                        if let Some(ref v) = env_info.rustc_version {
                            draw_pill(ui, v, Color32::from_rgb(30, 110, 60));
                        } else {
                            draw_pill(ui, &self.tr("rustc não detectado"), Color32::from_rgb(140, 40, 40));
                        }

                        if let Some(ref v) = env_info.cargo_version {
                            draw_pill(ui, v, Color32::from_rgb(35, 75, 140));
                        }

                        if let Some(ref v) = env_info.rustup_version {
                            draw_pill(ui, v, Color32::from_rgb(60, 60, 90));
                        }

                        if env_info.clippy_installed {
                            draw_pill(ui, "clippy: ✔", Color32::from_rgb(40, 100, 70));
                        }

                        if env_info.rustfmt_installed {
                            draw_pill(ui, "rustfmt: ✔", Color32::from_rgb(40, 100, 70));
                        }

                        if env_info.rust_analyzer_installed {
                            draw_pill(ui, "rust-analyzer: ✔", Color32::from_rgb(40, 100, 70));
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(self.tr("🔄 Atualizar")).clicked() {
                                refresh_env = true;
                            }
                        });
                    });
                });

            ui.add_space(8.0);
        }

        if refresh_env {
            self.spawn_dev_env_detection();
        }

        let p = self.palette();

        // ─── BARRA DE ABAS DO RUST STUDIO ──────────────────────────────────
        ui.horizontal(|ui| {
            let tabs = [
                (0, "💻 Editor & IDE"),
                (1, "⚡ Console & Terminal"),
                (2, "📦 Projetos & Ferramentas"),
            ];
            for (idx, label) in tabs {
                let active = self.terminal_active_tab == idx;
                let (fill, text_col, stroke) = if active {
                    (p.accent, Color32::WHITE, Stroke::new(1.5_f32, p.accent))
                } else {
                    (p.bg_card, p.text_secondary, Stroke::new(1.0_f32, p.border_subtle))
                };
                let resp = egui::Frame::NONE
                    .fill(fill)
                    .stroke(stroke)
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(egui::Margin::symmetric(14, 7))
                    .show(ui, |ui| {
                        ui.label(RichText::new(self.tr(label)).strong().size(12.5).color(text_col));
                    });
                if resp.response.interact(egui::Sense::click()).clicked() {
                    self.terminal_active_tab = idx;
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .button(self.tr("💻 Terminal Nativo"))
                    .on_hover_text(self.tr("Abre o Windows Terminal ou PowerShell nativo na pasta de trabalho"))
                    .clicked()
                {
                    if let Err(e) = terminal_studio::open_native_terminal(&self.terminal_working_dir) {
                        self.status_msg = e;
                        self.status_is_error = true;
                    }
                }
                if ui
                    .button(self.tr("📁 Abrir Explorer"))
                    .on_hover_text(self.tr("Abre a pasta de trabalho no Windows Explorer"))
                    .clicked()
                {
                    if let Err(e) = terminal_studio::open_in_explorer(&self.terminal_working_dir) {
                        self.status_msg = e;
                        self.status_is_error = true;
                    }
                }
            });
        });

        ui.add_space(8.0);

        match self.terminal_active_tab {
            0 => {
                // ─── ABA 0: EDITOR DE CÓDIGO RUST INTEGRADO (IDE) ───────────
                let total_avail = ui.available_height();
                let (editor_h, output_h) = if self.rust_ide_hide_output {
                    ((total_avail - 65.0).max(420.0), 0.0_f32)
                } else {
                    let content_avail = (total_avail - 125.0).max(350.0);
                    let ed_h = (content_avail * 0.65).clamp(260.0, 750.0);
                    let out_h = (content_avail - ed_h).clamp(120.0, 350.0);
                    (ed_h, out_h)
                };

                egui::Frame::NONE
                    .fill(Color32::from_rgb(16, 20, 28))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(32, 42, 60)))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(egui::Margin::same(10))
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new(self.tr("💻 Editor de Código Rust (IDE):")).strong().size(13.0).color(Color32::WHITE));
                            ui.add_space(4.0);

                            // Templates rápidos
                            ui.label(RichText::new(self.tr("Templates Rápidos:")).size(12.0).color(Color32::from_rgb(160, 175, 200)));
                            if ui.button(self.tr("Olá Mundo")).clicked() {
                                self.rust_ide_code = "fn main() {\n    println!(\"Olá, mundo do Rust!\");\n}\n".to_string();
                                self.status_msg = self.tr("Código do template carregado no editor.");
                                self.status_is_error = false;
                            }
                            if ui.button(self.tr("Calculadora / Funções")).clicked() {
                                self.rust_ide_code = "fn somar(a: i32, b: i32) -> i32 {\n    a + b\n}\n\nfn main() {\n    let x = 15;\n    let y = 27;\n    println!(\"{} + {} = {}\", x, y, somar(x, y));\n}\n".to_string();
                                self.status_msg = self.tr("Código do template carregado no editor.");
                                self.status_is_error = false;
                            }
                            if ui.button(self.tr("Concorrência / Threads")).clicked() {
                                self.rust_ide_code = "use std::thread;\nuse std::time::Duration;\n\nfn main() {\n    let handle = thread::spawn(|| {\n        for i in 1..=3 {\n            println!(\"Thread filha: {}\", i);\n            thread::sleep(Duration::from_millis(50));\n        }\n    });\n    handle.join().unwrap();\n    println!(\"Thread principal finalizada!\");\n}\n".to_string();
                                self.status_msg = self.tr("Código do template carregado no editor.");
                                self.status_is_error = false;
                            }
                            if ui.button(self.tr("Servidor HTTP / Tokio")).clicked() {
                                self.rust_ide_code = "// Exemplo assíncrono com runtime Tokio\n#[tokio::main]\nasync fn main() {\n    println!(\"🚀 Iniciando tarefa assíncrona com Tokio...\");\n    let msg = tokio::spawn(async {\n        \"Tarefa em background finalizada com sucesso!\"\n    }).await.unwrap();\n    println!(\"Resultado: {}\", msg);\n}\n".to_string();
                                self.status_msg = self.tr("Código do template carregado no editor.");
                                self.status_is_error = false;
                            }
                            if ui.button(self.tr("Leitura & JSON / Serde")).clicked() {
                                self.rust_ide_code = "// Exemplo de serialização JSON com Serde\nuse serde::{Serialize, Deserialize};\n\n#[derive(Serialize, Deserialize, Debug)]\nstruct Perfil {\n    nome: String,\n    nivel: u32,\n    ativo: bool,\n}\n\nfn main() {\n    let p = Perfil { nome: \"RustDev\".into(), nivel: 10, ativo: true };\n    let json = serde_json::to_string_pretty(&p).unwrap();\n    println!(\"JSON formatado:\\n{}\", json);\n}\n".to_string();
                                self.status_msg = self.tr("Código do template carregado no editor.");
                                self.status_is_error = false;
                            }
                            if ui.button(self.tr("CLI com Argumentos")).clicked() {
                                self.rust_ide_code = "fn main() {\n    let args: Vec<String> = std::env::args().collect();\n    println!(\"Argumentos passados pela CLI: {:?}\", args);\n    if args.len() > 1 {\n        println!(\"Primeiro parâmetro recebido: {}\", args[1]);\n    } else {\n        println!(\"Nenhum argumento adicional fornecido.\");\n    }\n}\n".to_string();
                                self.status_msg = self.tr("Código do template carregado no editor.");
                                self.status_is_error = false;
                            }
                            if ui.button(self.tr("Busca Binária")).clicked() {
                                self.rust_ide_code = "fn busca_binaria(arr: &[i32], alvo: i32) -> Option<usize> {\n    let mut ini = 0;\n    let mut fim = arr.len();\n    while ini < fim {\n        let meio = ini + (fim - ini) / 2;\n        if arr[meio] == alvo {\n            return Some(meio);\n        } else if arr[meio] < alvo {\n            ini = meio + 1;\n        } else {\n            fim = meio;\n        }\n    }\n    None\n}\n\nfn main() {\n    let numeros = [2, 5, 8, 12, 16, 23, 38, 56, 72, 91];\n    println!(\"Array ordenado: {:?}\", numeros);\n    let alvo = 23;\n    match busca_binaria(&numeros, alvo) {\n        Some(pos) => println!(\"Elemento {} encontrado no índice {}!\", alvo, pos),\n        None => println!(\"Elemento {} não encontrado.\", alvo),\n    }\n}\n".to_string();
                                self.status_msg = self.tr("Código do template carregado no editor.");
                                self.status_is_error = false;
                            }
                            if ui.button(self.tr("Árvore Binária")).clicked() {
                                self.rust_ide_code = "struct No {\n    valor: i32,\n    esq: Option<Box<No>>,\n    dir: Option<Box<No>>,\n}\n\nimpl No {\n    fn folha(valor: i32) -> Self {\n        Self { valor, esq: None, dir: None }\n    }\n    fn inserir(&mut self, novo_val: i32) {\n        if novo_val < self.valor {\n            match &mut self.esq {\n                Some(ref mut n) => n.inserir(novo_val),\n                None => self.esq = Some(Box::new(No::folha(novo_val))),\n            }\n        } else {\n            match &mut self.dir {\n                Some(ref mut n) => n.inserir(novo_val),\n                None => self.dir = Some(Box::new(No::folha(novo_val))),\n            }\n        }\n    }\n    fn em_ordem(&self) {\n        if let Some(ref e) = self.esq { e.em_ordem(); }\n        print!(\"{} \", self.valor);\n        if let Some(ref d) = self.dir { d.em_ordem(); }\n    }\n}\n\nfn main() {\n    let mut raiz = No::folha(50);\n    for v in [30, 70, 20, 40, 60, 80] {\n        raiz.inserir(v);\n    }\n    print!(\"Árvore Binária ordenada (em-ordem): \");\n    raiz.em_ordem();\n    println!();\n}\n".to_string();
                                self.status_msg = self.tr("Código do template carregado no editor.");
                                self.status_is_error = false;
                            }
                            if ui.button(self.tr("Regex / Validação")).clicked() {
                                self.rust_ide_code = "// Exemplo com crate regex\n// No Cargo.toml: regex = \"1.10\"\nuse regex::Regex;\n\nfn main() {\n    let re_email = Regex::new(r\"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$\").unwrap();\n    let testes = [\"usuario@rust-lang.org\", \"invalido@@email\", \"contato@rrs.app\", \"sem-arroba.com\"];\n    println!(\"=== Validador de E-mails com Regex ===\");\n    for email in testes {\n        let valido = re_email.is_match(email);\n        println!(\"{:<25} -> {}\", email, if valido { \"✔ Válido\" } else { \"❌ Inválido\" });\n    }\n}\n".to_string();
                                self.status_msg = self.tr("Código do template carregado no editor.");
                                self.status_is_error = false;
                            }
                            if ui.button(self.tr("Hash SHA-256")).clicked() {
                                self.rust_ide_code = "// Exemplo com crate sha2 e hex\n// No Cargo.toml: sha2 = \"0.10\", hex = \"0.4\"\nuse sha2::{Sha256, Digest};\n\nfn main() {\n    let mensagem = \"RRS - Riva Rust Store: Integridade & Velocidade\";\n    let mut hasher = Sha256::new();\n    hasher.update(mensagem.as_bytes());\n    let resultado = hasher.finalize();\n    println!(\"Mensagem: {}\", mensagem);\n    println!(\"Hash SHA-256: {}\", hex::encode(resultado));\n}\n".to_string();
                                self.status_msg = self.tr("Código do template carregado no editor.");
                                self.status_is_error = false;
                            }
                            if ui.button(self.tr("Web API / Axum")).clicked() {
                                self.rust_ide_code = "// Exemplo de Microserviço REST com Axum e Tokio\n// No Cargo.toml: axum = \"0.7\", tokio = { version = \"1.0\", features = [\"full\"] }, serde = { version = \"1.0\", features = [\"derive\"] }\nuse axum::{routing::get, Json, Router};\nuse serde::Serialize;\n\n#[derive(Serialize)]\nstruct StatusApp {\n    nome: &'static str,\n    versao: &'static str,\n    status: &'static str,\n}\n\nasync fn get_status() -> Json<StatusApp> {\n    Json(StatusApp {\n        nome: \"RRS Rust Service\",\n        versao: \"1.0.0\",\n        status: \"operacional\",\n    })\n}\n\n#[tokio::main]\nasync fn main() {\n    println!(\"Iniciando servidor Axum na porta 3000...\");\n    let app = Router::new().route(\"/api/status\", get(get_status));\n    println!(\"Rotas registradas: /api/status\");\n}\n".to_string();
                                self.status_msg = self.tr("Código do template carregado no editor.");
                                self.status_is_error = false;
                            }
                            if ui.button(self.tr("Barra de Progresso")).clicked() {
                                self.rust_ide_code = "// Exemplo de feedback visual de progresso com indicatif\n// No Cargo.toml: indicatif = \"0.17\"\nuse indicatif::{ProgressBar, ProgressStyle};\nuse std::thread;\nuse std::time::Duration;\n\nfn main() {\n    let pb = ProgressBar::new(100);\n    pb.set_style(ProgressStyle::default_bar()\n        .template(\"{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})\")\n        .unwrap()\n        .progress_chars(\"#>- \"));\n    println!(\"Processando itens em background...\");\n    for _ in 0..100 {\n        pb.inc(1);\n        thread::sleep(Duration::from_millis(20));\n    }\n    pb.finish_with_message(\"✔ Tarefas finalizadas com sucesso!\");\n}\n".to_string();
                                self.status_msg = self.tr("Código do template carregado no editor.");
                                self.status_is_error = false;
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if self.rust_ide_hide_output {
                                    if ui
                                        .button(self.tr("📊 Mostrar Saída"))
                                        .on_hover_text(self.tr("Exibe o painel de saída e logs de compilação."))
                                        .clicked()
                                    {
                                        self.rust_ide_hide_output = false;
                                    }
                                } else if ui
                                    .button(self.tr("🖥️ Maximizar Editor"))
                                    .on_hover_text(self.tr("Expande a área do editor de código ocupando toda a tela."))
                                    .clicked()
                                {
                                    self.rust_ide_hide_output = true;
                                }
                                if ui.button(self.tr("🧹 Limpar")).clicked() {
                                    self.rust_ide_code.clear();
                                }
                                if ui.button(self.tr("📋 Copiar")).clicked() {
                                    ui.ctx().copy_text(self.rust_ide_code.clone());
                                    self.status_msg = self.tr("Código copiado para a área de transferência.");
                                    self.status_is_error = false;
                                }
                                if ui
                                    .add_enabled(
                                        !self.rust_ide_code.trim().is_empty(),
                                        egui::Button::new(
                                            RichText::new(self.tr("✨ Formatar (rustfmt)"))
                                                .color(Color32::from_rgb(100, 215, 255)),
                                        ),
                                    )
                                    .on_hover_text(self.tr("Formata e indenta o código Rust seguindo os padrões oficiais com o rustfmt."))
                                    .clicked()
                                {
                                    match terminal_studio::format_rust_code(&self.rust_ide_code) {
                                        Ok(formatted) => {
                                            self.rust_ide_code = formatted;
                                            self.status_msg = self.tr("Código formatado com sucesso via rustfmt.");
                                            self.status_is_error = false;
                                        }
                                        Err(err) => {
                                            self.status_msg = format!("{}: {}", self.tr("Falha ao formatar"), err);
                                            self.status_is_error = true;
                                        }
                                    }
                                }
                                if ui
                                    .add_enabled(!self.terminal_is_running && !self.rust_ide_code.trim().is_empty(), egui::Button::new(RichText::new(self.tr("▶ Executar Código")).strong().color(Color32::from_rgb(80, 220, 120))))
                                    .on_hover_text(self.tr("Compila e executa este código imediatamente com o compilador rustc em segundo plano."))
                                    .clicked()
                                {
                                    self.rust_ide_hide_output = false;
                                    let code = self.rust_ide_code.clone();
                                    self.spawn_terminal_eval(code);
                                }
                            });
                        });

                        ui.add_space(6.0);

                        // Área do editor de texto com fonte monospace
                        let desired_rows = if self.rust_ide_hide_output { 35 } else { 20 };
                        egui::ScrollArea::vertical()
                            .id_salt("rust_ide_editor_scroll")
                            .max_height(editor_h)
                            .show(ui, |ui| {
                                ui.set_height(editor_h);
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.rust_ide_code)
                                        .font(egui::TextStyle::Monospace)
                                        .desired_width(ui.available_width())
                                        .desired_rows(desired_rows)
                                        .lock_focus(true),
                                );
                            });
                    });

                if !self.rust_ide_hide_output {
                    ui.add_space(8.0);

                    // Painel de saída da execução do código
                    egui::Frame::NONE
                        .fill(Color32::from_rgb(10, 12, 16))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(32, 40, 56)))
                        .corner_radius(CornerRadius::same(8))
                        .inner_margin(egui::Margin::symmetric(12, 8))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(self.tr("📋 Saída da Execução:")).strong().size(12.5).color(Color32::WHITE));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui
                                        .button(self.tr("▼ Ocultar"))
                                        .on_hover_text(self.tr("Oculta este painel de saída e maximiza o editor."))
                                        .clicked()
                                    {
                                        self.rust_ide_hide_output = true;
                                    }
                                    if ui.button(self.tr("🧹 Limpar Saída")).clicked() {
                                        self.terminal_logs.clear();
                                    }
                                    if self.terminal_is_running {
                                        draw_pill(ui, &self.tr("⏳ Compilando e Executando..."), Color32::from_rgb(200, 120, 30));
                                    }
                                });
                            });
                            ui.add_space(4.0);
                            ui.separator();
                            ui.add_space(4.0);

                            egui::ScrollArea::vertical()
                                .id_salt("rust_ide_output_scroll")
                                .stick_to_bottom(true)
                                .max_height(output_h)
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.set_height(output_h);
                                    if self.terminal_logs.is_empty() {
                                        ui.label(RichText::new(self.tr("Nenhum código executado ainda. Clique em '▶ Executar Código' acima para compilar e rodar.")).monospace().size(12.0).color(Color32::from_rgb(110, 125, 145)));
                                    }
                                    for (kind, text) in &self.terminal_logs {
                                        let text_col = match kind {
                                            terminal_studio::TerminalLineKind::Command => Color32::from_rgb(0, 240, 255),
                                            terminal_studio::TerminalLineKind::Stdout => Color32::from_rgb(220, 225, 235),
                                            terminal_studio::TerminalLineKind::Stderr => Color32::from_rgb(245, 90, 80),
                                            terminal_studio::TerminalLineKind::SystemInfo => Color32::from_rgb(90, 175, 240),
                                        };
                                        ui.label(RichText::new(text).monospace().size(12.0).color(text_col));
                                    }
                                });
                        });
                }
            }
            1 => {
                // ─── ABA 1: CONSOLE DO TERMINAL INTEGRADO (MODERNO COM 🔴🟡🟢) ──
                let available_h = (ui.available_height() - 95.0).max(220.0);
                egui::Frame::NONE
                    .fill(Color32::from_rgb(10, 12, 16))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(32, 40, 56)))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(egui::Margin::symmetric(12, 8))
                    .show(ui, |ui| {
                        // Barra superior da janela estilo terminal moderno
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🔴 🟡 🟢").size(11.0));
                            ui.add_space(6.0);
                            let folder_name = self.terminal_working_dir.file_name()
                                .map(|f| f.to_string_lossy().to_string())
                                .unwrap_or_else(|| "Projetos Rust".to_string());
                            ui.label(
                                RichText::new(format!("terminal ~ [{}]", folder_name))
                                    .monospace()
                                    .size(12.0)
                                    .color(Color32::from_rgb(160, 175, 200)),
                            );

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(self.tr("🧹 Limpar Saída")).clicked() {
                                    self.terminal_logs.clear();
                                }
                                if ui.button(self.tr("📋 Copiar Logs")).clicked() {
                                    let full_log: String = self.terminal_logs.iter()
                                        .map(|(_, text)| text.as_str())
                                        .collect::<Vec<_>>()
                                        .join("\n");
                                    ui.ctx().copy_text(full_log);
                                    self.status_msg = self.tr("Logs do terminal copiados.");
                                    self.status_is_error = false;
                                }
                                if self.terminal_is_running {
                                    draw_pill(ui, &self.tr("⏳ Executando..."), Color32::from_rgb(200, 120, 30));
                                } else {
                                    draw_pill(ui, &self.tr("● Pronto"), Color32::from_rgb(30, 120, 60));
                                }
                            });
                        });
                        ui.add_space(4.0);
                        ui.separator();
                        ui.add_space(4.0);

                        egui::ScrollArea::vertical()
                            .id_salt("rust_studio_output_scroll")
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.set_height(available_h);

                                if self.terminal_logs.is_empty() {
                                    ui.label(
                                        RichText::new(self.tr(
                                            "Terminal pronto. Digite um comando abaixo ou use os atalhos rápidos do Cargo.",
                                        ))
                                        .monospace()
                                        .size(12.0)
                                        .color(Color32::from_rgb(110, 125, 145)),
                                    );
                                }

                                for (kind, text) in &self.terminal_logs {
                                    let text_col = match kind {
                                        terminal_studio::TerminalLineKind::Command => Color32::from_rgb(0, 240, 255),
                                        terminal_studio::TerminalLineKind::Stdout => {
                                            if text.starts_with("Compiling") || text.starts_with("Finished") || text.starts_with("Running") {
                                                Color32::from_rgb(80, 220, 120)
                                            } else {
                                                Color32::from_rgb(220, 225, 235)
                                            }
                                        }
                                        terminal_studio::TerminalLineKind::Stderr => {
                                            if text.contains("warning:") {
                                                Color32::from_rgb(240, 190, 60)
                                            } else {
                                                Color32::from_rgb(245, 90, 80)
                                            }
                                        }
                                        terminal_studio::TerminalLineKind::SystemInfo => Color32::from_rgb(90, 175, 240),
                                    };

                                    ui.label(
                                        RichText::new(text)
                                            .monospace()
                                            .size(12.0)
                                            .color(text_col),
                                    );
                                }
                            });
                    });

                ui.add_space(6.0);

                // Atalhos rápidos Cargo
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(self.tr("Atalhos Cargo:")).strong().size(12.0).color(Color32::WHITE));

                    let shortcuts = [
                ("⚡ cargo check", "cargo check"),
                ("🔨 cargo build", "cargo build"),
                ("▶ cargo run", "cargo run"),
                ("🧪 cargo test", "cargo test"),
                ("📎 cargo clippy", "cargo clippy"),
                ("📚 cargo doc", "cargo doc --open"),
                ("🦀 rustc --version", "rustc --version"),
                ("🔍 rustup show", "rustup show"),
                ("📦 cargo clean", "cargo clean"),
            ];

                    for (label, cmd) in shortcuts {
                        if ui
                            .add_enabled(!self.terminal_is_running, egui::Button::new(RichText::new(label).size(12.0)))
                            .clicked()
                        {
                            self.spawn_terminal_cmd(cmd);
                        }
                    }
                });

                ui.add_space(6.0);

                // Barra de entrada do comando com histórico ↑/↓
                ui.horizontal(|ui| {
                    ui.label(RichText::new("$").monospace().strong().size(15.0).color(Color32::from_rgb(80, 220, 120)));

                    let input_hint = self.tr("Digite um comando (ex: cargo build, cargo add serde, rustc --version)...");
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut self.terminal_cmd_input)
                            .font(egui::TextStyle::Monospace)
                            .hint_text(input_hint)
                            .desired_width((ui.available_width() - 200.0).max(200.0)),
                    );

                    // Navegação de histórico de comandos com setas ↑ e ↓
                    if resp.has_focus() && !self.terminal_cmd_history.is_empty() {
                        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                            let next_idx = match self.terminal_cmd_history_idx {
                                None => self.terminal_cmd_history.len().saturating_sub(1),
                                Some(idx) => idx.saturating_sub(1),
                            };
                            self.terminal_cmd_history_idx = Some(next_idx);
                            if let Some(cmd) = self.terminal_cmd_history.get(next_idx) {
                                self.terminal_cmd_input = cmd.clone();
                            }
                        } else if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                            if let Some(idx) = self.terminal_cmd_history_idx {
                                if idx + 1 < self.terminal_cmd_history.len() {
                                    let next_idx = idx + 1;
                                    self.terminal_cmd_history_idx = Some(next_idx);
                                    if let Some(cmd) = self.terminal_cmd_history.get(next_idx) {
                                        self.terminal_cmd_input = cmd.clone();
                                    }
                                } else {
                                    self.terminal_cmd_history_idx = None;
                                    self.terminal_cmd_input.clear();
                                }
                            }
                        }
                    }

                    let enter_pressed = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    let btn_clicked = ui
                        .add_enabled(
                            !self.terminal_is_running && !self.terminal_cmd_input.trim().is_empty(),
                            egui::Button::new(RichText::new(self.tr("▶ Executar")).strong()),
                        )
                        .clicked();

                    if self.terminal_is_running
                        && ui
                            .add(egui::Button::new(RichText::new(self.tr("⏹ Parar")).strong()).fill(Color32::from_rgb(150, 50, 40)))
                            .on_hover_text(self.tr("Encerra o comando e os programas que ele abriu."))
                            .clicked()
                    {
                        self.terminal_cancel.store(true, std::sync::atomic::Ordering::Relaxed);
                    }

                    if enter_pressed || btn_clicked {
                        let cmd = self.terminal_cmd_input.clone();
                        self.spawn_terminal_cmd(&cmd);
                    }
                });
                ui.label(
                    RichText::new(self.tr(
                        "Os comandos rodam no PowerShell com as permissões da sua conta. Cole só comandos em que você confia. Tempo máximo: 30 min por comando.",
                    ))
                    .size(11.0)
                    .color(Color32::from_rgb(140, 150, 170)),
                );
            }
            _ => {
                // ─── ABA 2: PROJETOS CARGO & FERRAMENTAS ────────────────────
                // 1. Gerador de Projetos Cargo
                egui::Frame::NONE
                    .fill(Color32::from_rgb(18, 22, 30))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 42, 58)))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(egui::Margin::symmetric(12, 10))
                    .show(ui, |ui| {
                        ui.label(RichText::new(self.tr("📦 Novo Projeto Cargo:")).strong().size(13.0).color(Color32::WHITE));
                        ui.add_space(4.0);
                        ui.horizontal_wrapped(|ui| {
                            let cargo_hint = self.tr("nome_do_projeto");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.cargo_new_name)
                                    .hint_text(cargo_hint)
                                    .desired_width(160.0),
                            );

                            ui.radio_value(&mut self.cargo_new_is_bin, true, "--bin (App)");
                            ui.radio_value(&mut self.cargo_new_is_bin, false, "--lib (Lib)");

                            if ui
                                .add_enabled(
                                    !self.cargo_new_name.trim().is_empty() && !self.terminal_is_running,
                                    egui::Button::new(RichText::new(self.tr("➕ Criar")).strong()),
                                )
                                .clicked()
                            {
                                let name = self.cargo_new_name.trim().to_string();
                                match terminal_studio::create_cargo_project(&self.terminal_working_dir, &name, self.cargo_new_is_bin) {
                                    Ok(msg) => {
                                        self.terminal_logs.push((
                                            terminal_studio::TerminalLineKind::SystemInfo,
                                            format!("✔ {}", msg),
                                        ));
                                        let project = self.terminal_working_dir.join(&name);
                                        if project.is_dir() {
                                            self.terminal_dir_input = project.to_string_lossy().to_string();
                                            self.terminal_working_dir = project;
                                        }
                                        self.status_msg = msg;
                                        self.status_is_error = false;
                                        self.cargo_new_name.clear();
                                    }
                                    Err(e) => {
                                        self.terminal_logs.push((
                                            terminal_studio::TerminalLineKind::Stderr,
                                            format!("❌ {}", e),
                                        ));
                                        self.status_msg = e;
                                        self.status_is_error = true;
                                    }
                                }
                            }
                        });
                    });

                ui.add_space(8.0);

                // 2. Adicionar Dependências (cargo add)
                egui::Frame::NONE
                    .fill(Color32::from_rgb(18, 22, 30))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 42, 58)))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(egui::Margin::symmetric(12, 10))
                    .show(ui, |ui| {
                        ui.label(RichText::new(self.tr("📎 Adicionar Dependências (cargo add):")).strong().size(13.0).color(Color32::WHITE));
                        ui.add_space(4.0);
                        ui.horizontal_wrapped(|ui| {
                            let crate_hint = self.tr("nome da crate (ex: serde, tokio, rand)");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.cargo_add_input)
                                    .hint_text(crate_hint)
                                    .desired_width(220.0),
                            );

                            if ui
                                .add_enabled(!self.cargo_add_input.trim().is_empty() && !self.terminal_is_running, egui::Button::new(RichText::new(self.tr("➕ Adicionar")).strong()))
                                .clicked()
                            {
                                let pkg = self.cargo_add_input.trim().to_string();
                                self.terminal_cmd_input = format!("cargo add {pkg}");
                                self.cargo_add_input.clear();
                                self.terminal_active_tab = 1;
                                self.status_msg = format!("{} {}", self.tr("Comando preparado no terminal:"), self.terminal_cmd_input);
                                self.status_is_error = false;
                            }
                        });

                        ui.add_space(6.0);
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new(self.tr("Populares:")).size(12.0).color(Color32::from_rgb(160, 175, 200)));
                            let popular = [
                                ("serde (derive)", "serde --features derive"),
                                ("tokio (full)", "tokio --features full"),
                                ("anyhow", "anyhow"),
                                ("rand", "rand"),
                                ("clap (derive)", "clap --features derive"),
                                ("reqwest", "reqwest"),
                            ];
                            for (label, pkg) in popular {
                                if ui.button(format!("+ {label}")).clicked() {
                                    self.terminal_cmd_input = format!("cargo add {pkg}");
                                    self.terminal_active_tab = 1;
                                    self.status_msg = format!("{} {}", self.tr("Comando preparado no terminal:"), self.terminal_cmd_input);
                                    self.status_is_error = false;
                                }
                            }
                        });

                        ui.add_space(6.0);
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new(self.tr("Diagnóstico de Dependências:")).size(12.0).color(Color32::from_rgb(160, 175, 200)));
                            if ui.button(self.tr("🌳 Árvore de Dependências (cargo tree)")).clicked() {
                                self.terminal_cmd_input = "cargo tree".into();
                                self.terminal_active_tab = 1;
                                self.status_msg = format!("{} {}", self.tr("Comando preparado no terminal:"), self.terminal_cmd_input);
                                self.status_is_error = false;
                            }
                            if ui.button(self.tr("🔄 Atualizar Dependências (cargo update)")).clicked() {
                                self.terminal_cmd_input = "cargo update".into();
                                self.terminal_active_tab = 1;
                                self.status_msg = format!("{} {}", self.tr("Comando preparado no terminal:"), self.terminal_cmd_input);
                                self.status_is_error = false;
                            }
                        });
                    });

                ui.add_space(8.0);

                // 3. Scratchpad REPL
                egui::Frame::NONE
                    .fill(Color32::from_rgb(18, 22, 30))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 42, 58)))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(egui::Margin::symmetric(12, 10))
                    .show(ui, |ui| {
                        ui.label(RichText::new(self.tr("⚡ Scratchpad REPL (Avaliador de Expressões):")).strong().size(13.0).color(Color32::WHITE));
                        ui.add_space(4.0);
                        ui.horizontal_wrapped(|ui| {
                            let repl_hint = self.tr("Expressão Rust (ex: 2 + 2 * 10)");
                            let repl_resp = ui.add(
                                egui::TextEdit::singleline(&mut self.terminal_repl_input)
                                    .hint_text(repl_hint)
                                    .desired_width(280.0),
                            );

                            let eval_enter = repl_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                            let eval_clicked = ui
                                .add_enabled(
                                    !self.terminal_repl_input.trim().is_empty() && !self.terminal_is_running,
                                    egui::Button::new(RichText::new(self.tr("▶ Avaliar")).strong()),
                                )
                                .clicked();

                            if (eval_enter || eval_clicked) && !self.terminal_repl_input.trim().is_empty() && !self.terminal_is_running {
                                let expr = std::mem::take(&mut self.terminal_repl_input);
                                self.spawn_terminal_eval(expr);
                            }
                        });
                    });

                ui.add_space(8.0);

                // 4. Pasta de Trabalho do Terminal
                egui::Frame::NONE
                    .fill(Color32::from_rgb(18, 22, 30))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 42, 58)))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(egui::Margin::symmetric(12, 10))
                    .show(ui, |ui| {
                        ui.label(RichText::new(self.tr("📂 Pasta de Trabalho do Terminal:")).strong().size(13.0).color(Color32::WHITE));
                        ui.add_space(4.0);
                        ui.horizontal_wrapped(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.terminal_dir_input)
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(380.0),
                            );
                            if ui
                                .add_enabled(!self.terminal_is_running, egui::Button::new(self.tr("Usar pasta")))
                                .clicked()
                            {
                                let p = std::path::PathBuf::from(self.terminal_dir_input.trim());
                                if p.is_dir() {
                                    self.terminal_working_dir = p;
                                } else {
                                    self.status_msg = format!("❌ {} {}", self.tr("Pasta não encontrada:"), self.terminal_dir_input.trim());
                                    self.status_is_error = true;
                                }
                            }
                            if ui
                                .add_enabled(!self.terminal_is_running, egui::Button::new(self.tr("🔄 Resetar Pasta")))
                                .on_hover_text(self.tr("Volta para a pasta Projetos Rust, dentro de Documentos."))
                                .clicked()
                            {
                                self.terminal_working_dir = terminal_studio::default_projects_dir();
                                self.terminal_dir_input = self.terminal_working_dir.to_string_lossy().to_string();
                            }
                            if ui
                                .button(self.tr("📁 Abrir Explorer"))
                                .on_hover_text(self.tr("Abre a pasta de trabalho no Windows Explorer"))
                                .clicked()
                            {
                                if let Err(e) = terminal_studio::open_in_explorer(&self.terminal_working_dir) {
                                    self.status_msg = e;
                                    self.status_is_error = true;
                                }
                            }
                            if ui
                                .button(self.tr("💻 Terminal Nativo"))
                                .on_hover_text(self.tr("Abre o Windows Terminal ou PowerShell nativo na pasta de trabalho"))
                                .clicked()
                            {
                                if let Err(e) = terminal_studio::open_native_terminal(&self.terminal_working_dir) {
                                    self.status_msg = e;
                                    self.status_is_error = true;
                                }
                            }
                        });
                    });
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // 2. TELA DEDICADA: ESPELHO DE IDES EXTERNAS (VS CODE / ANTIGRAVITY)
    // ─────────────────────────────────────────────────────────────────────────

    /// Renderiza a tela dedicada para espelhamento e integração com IDEs externas (VS Code & Google Antigravity).
    pub(super) fn render_ide_mirror(&mut self, ui: &mut egui::Ui) {
        if self.terminal_working_dir.as_os_str().is_empty() {
            self.terminal_working_dir = terminal_studio::default_projects_dir();
            self.terminal_dir_input = self.terminal_working_dir.to_string_lossy().to_string();
        }

        // ─── CABEÇALHO DO ESPELHO DE IDES ──────────────────────────────────
        ui.horizontal(|ui| {
            brand::logo_badge(ui, 18.0);
            ui.add_space(6.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(self.tr("🖥️ Espelho de Programas & IDEs de Codificação"))
                            .size(20.0)
                            .strong()
                            .color(Color32::WHITE),
                    );
                    draw_pill(ui, &self.tr("Espelho de IDEs"), Color32::from_rgb(30, 120, 200));
                });
                ui.label(
                    RichText::new(self.tr(
                        "Conecte seu projeto com editores externos instalados no computador (Visual Studio Code e Google Antigravity).",
                    ))
                    .size(12.5)
                    .color(Color32::from_rgb(145, 160, 185)),
                );
            });
        });

        ui.add_space(10.0);

        // ─── PAINEL DE PROGRAMAS DETECTADOS ────────────────────────────────
        if self.detected_ides.is_empty() && !self.dev_env_detecting {
            self.spawn_dev_env_detection();
        }

        let ides = self.detected_ides.clone();
        egui::Frame::NONE
            .fill(Color32::from_rgb(20, 24, 34))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(38, 46, 62)))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(egui::Margin::same(14))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(self.tr("🚀 Programas de Codificação Detectados:")).strong().color(Color32::WHITE));
                    ui.add_space(8.0);

                    for ide_item in &ides {
                        if ide_item.is_installed {
                            draw_pill(ui, &format!("{} (Pronto)", ide_item.badge), Color32::from_rgb(30, 110, 60));
                        } else {
                            draw_pill(ui, &format!("{} (Não detectado)", ide_item.badge), Color32::from_rgb(60, 66, 80));
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(self.tr("🔄 Atualizar")).clicked() {
                            self.spawn_dev_env_detection();
                        }
                    });
                });

                ui.add_space(10.0);
                ui.horizontal_wrapped(|ui| {
                    let dir = self.terminal_working_dir.clone();

                    if ui
                        .add(egui::Button::new(RichText::new(self.tr("🟦 Abrir no VS Code")).strong()))
                        .on_hover_text(self.tr("Abre a pasta atual do projeto no Visual Studio Code."))
                        .clicked()
                    {
                        match ide::open_workspace_in_ide("vscode", &dir) {
                            Ok(msg) => {
                                self.status_msg = format!("✔ {msg}");
                                self.status_is_error = false;
                            }
                            Err(e) => {
                                self.status_msg = format!("❌ {e}");
                                self.status_is_error = true;
                            }
                        }
                    }

                    if ui
                        .add(egui::Button::new(RichText::new(self.tr("⚛ Abrir no Antigravity")).strong()))
                        .on_hover_text(self.tr("Abre o workspace atual no Google Antigravity."))
                        .clicked()
                    {
                        match ide::open_workspace_in_ide("antigravity", &dir) {
                            Ok(msg) => {
                                self.status_msg = format!("✔ {msg}");
                                self.status_is_error = false;
                            }
                            Err(e) => {
                                self.status_msg = format!("❌ {e}");
                                self.status_is_error = true;
                            }
                        }
                    }

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);

                    if ui
                        .add_enabled(self.busy_task.is_none(), egui::Button::new(self.tr("⚙ Instalar rust-analyzer no VS Code")))
                        .on_hover_text(self.tr("Instala a extensão oficial de autocompletar e conferência de tipos no VS Code."))
                        .clicked()
                    {
                        self.busy_task = Some(self.tr("Instalando rust-analyzer no VS Code..."));
                        let events = Arc::clone(&self.bg_events);
                        std::thread::spawn(move || {
                            let res = ide::install_vscode_extension("rust-lang.rust-analyzer").map_err(|e| format!("{e:#}"));
                            events.lock().unwrap().push(BgEvent::IdeActionFinished(res));
                        });
                    }

                    if ui
                        .add(egui::Button::new(self.tr("📁 Abrir Pasta no Explorer")))
                        .on_hover_text(self.tr("Abre a pasta atual no Windows Explorer."))
                        .clicked()
                    {
                        let _ = crate::downloader::reveal_in_folder(&dir.to_string_lossy());
                    }
                });
            });

        ui.add_space(10.0);

        // ─── CONFIGURAÇÃO DA PASTA DE TRABALHO DO PROJETO ──────────────────
        egui::Frame::NONE
            .fill(Color32::from_rgb(16, 20, 28))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(32, 40, 56)))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(egui::Margin::symmetric(12, 10))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(self.tr("📂 Pasta de Trabalho do Workspace:")).strong().size(12.5).color(Color32::from_rgb(160, 175, 200)));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.terminal_dir_input)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(400.0),
                    );
                    if ui
                        .add_enabled(!self.terminal_is_running, egui::Button::new(self.tr("Usar pasta")))
                        .clicked()
                    {
                        let p = std::path::PathBuf::from(self.terminal_dir_input.trim());
                        if p.is_dir() {
                            self.terminal_working_dir = p;
                        } else {
                            self.status_msg = format!("❌ {} {}", self.tr("Pasta não encontrada:"), self.terminal_dir_input.trim());
                            self.status_is_error = true;
                        }
                    }
                    if ui
                        .add_enabled(!self.terminal_is_running, egui::Button::new(self.tr("🔄 Resetar Pasta")))
                        .on_hover_text(self.tr("Volta para a pasta Projetos Rust, dentro de Documentos."))
                        .clicked()
                    {
                        self.terminal_working_dir = terminal_studio::default_projects_dir();
                        self.terminal_dir_input = self.terminal_working_dir.to_string_lossy().to_string();
                    }
                });
            });

        ui.add_space(10.0);

        // ─── CARTÃO EXPLICATIVO DE FLUXO DE TRABALHO ───────────────────────
        egui::Frame::NONE
            .fill(Color32::from_rgb(14, 18, 26))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(28, 36, 52)))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(egui::Margin::same(14))
            .show(ui, |ui| {
                ui.label(RichText::new(self.tr("💡 Como Funciona o Espelhamento de Editores:")).strong().size(13.5).color(Color32::WHITE));
                ui.add_space(6.0);
                ui.label(
                    RichText::new(self.tr(
                        "Ao clicar em 'Abrir no VS Code' ou 'Abrir no Antigravity', a RRS passa o caminho da pasta de trabalho selecionada diretamente para o executável do editor com parâmetros seguros. Qualquer alteração feita nos arquivos é refletida imediatamente em ambos os ambientes sem conflitos.",
                    ))
                    .size(12.0)
                    .color(Color32::from_rgb(170, 185, 210)),
                );
            });
    }
}
