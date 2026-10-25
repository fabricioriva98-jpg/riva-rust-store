//! Modo TV: tela cheia com os jogos e apps instalados em blocos grandes, navegável por controle
//! (via `gilrs`: XInput/DirectInput no Windows) ou teclado. Entra pelo botão da barra lateral.

use super::{friendly_title, StoreGui};
use crate::launchers::{self, ExternalGame};
use eframe::egui::{self, Color32, CornerRadius, Key, RichText, Stroke, Vec2};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TvItem {
    /// App instalado pela RRS (nome no catálogo).
    Rrs(String),
    External(ExternalGame),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TvAction {
    Left,
    Right,
    Up,
    Down,
    Activate,
    Back,
}

/// Nova posição na grade de `cols` colunas com `len` itens.
pub fn tv_move(index: usize, len: usize, cols: usize, action: TvAction) -> usize {
    if len == 0 {
        return 0;
    }
    let cols = cols.max(1);
    let i = index.min(len - 1);
    match action {
        TvAction::Left => i.saturating_sub(1),
        TvAction::Right => (i + 1).min(len - 1),
        TvAction::Up => i.checked_sub(cols).unwrap_or(i),
        TvAction::Down => {
            if i + cols < len {
                i + cols
            } else {
                i
            }
        }
        TvAction::Activate | TvAction::Back => i,
    }
}

/// Direção do analógico com zona morta; só gera ação quando sai do centro (sem repetir enquanto segura).
pub fn stick_actions(prev: (i8, i8), x: f32, y: f32) -> ((i8, i8), Vec<TvAction>) {
    let dir = |v: f32| -> i8 {
        if v > 0.6 {
            1
        } else if v < -0.6 {
            -1
        } else {
            0
        }
    };
    let now = (dir(x), dir(y));
    let mut out = Vec::new();
    if now.0 != prev.0 && now.0 != 0 {
        out.push(if now.0 > 0 { TvAction::Right } else { TvAction::Left });
    }
    if now.1 != prev.1 && now.1 != 0 {
        // No gilrs o eixo Y do analógico é positivo para cima
        out.push(if now.1 > 0 { TvAction::Up } else { TvAction::Down });
    }
    (now, out)
}

impl StoreGui {
    pub(super) fn enter_tv_mode(&mut self, ctx: &egui::Context) {
        if self.external_games.is_none() {
            self.external_games = Some(launchers::all_games());
        }
        if self.gamepads.is_none() {
            self.gamepads = gilrs::Gilrs::new().ok();
        }
        self.tv_mode = true;
        self.record_stat(crate::achievements::STAT_TV);
        self.tv_index = 0;
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
    }

    pub(super) fn exit_tv_mode(&mut self, ctx: &egui::Context) {
        self.tv_mode = false;
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
    }

    pub(super) fn tv_items(&self) -> Vec<TvItem> {
        let mut items: Vec<TvItem> = self
            .apps
            .iter()
            .filter(|a| a.installed.is_some())
            .map(|a| TvItem::Rrs(a.full_name.clone()))
            .collect();
        if let Some(ext) = &self.external_games {
            items.extend(ext.iter().cloned().map(TvItem::External));
        }
        items
    }

    fn poll_tv_input(&mut self, ctx: &egui::Context) -> Vec<TvAction> {
        let mut actions: Vec<TvAction> = ctx.input(|i| {
            let mut v = Vec::new();
            for (key, action) in [
                (Key::ArrowLeft, TvAction::Left),
                (Key::ArrowRight, TvAction::Right),
                (Key::ArrowUp, TvAction::Up),
                (Key::ArrowDown, TvAction::Down),
                (Key::Enter, TvAction::Activate),
                (Key::Escape, TvAction::Back),
                (Key::Backspace, TvAction::Back),
            ] {
                if i.key_pressed(key) {
                    v.push(action);
                }
            }
            v
        });
        if let Some(gilrs) = self.gamepads.as_mut() {
            use gilrs::{Axis, Button, EventType};
            while let Some(ev) = gilrs.next_event() {
                match ev.event {
                    EventType::ButtonPressed(b, _) => match b {
                        Button::DPadLeft => actions.push(TvAction::Left),
                        Button::DPadRight => actions.push(TvAction::Right),
                        Button::DPadUp => actions.push(TvAction::Up),
                        Button::DPadDown => actions.push(TvAction::Down),
                        Button::South | Button::Start => actions.push(TvAction::Activate),
                        Button::East | Button::Select => actions.push(TvAction::Back),
                        _ => {}
                    },
                    EventType::AxisChanged(axis @ (Axis::LeftStickX | Axis::LeftStickY), value, _) => {
                        let (mut x, mut y) = (0.0, 0.0);
                        if let Some(pad) = gilrs.gamepads().next().map(|(_, g)| g) {
                            x = pad.value(Axis::LeftStickX);
                            y = pad.value(Axis::LeftStickY);
                        }
                        if axis == Axis::LeftStickX {
                            x = value;
                        } else {
                            y = value;
                        }
                        let (state, acts) = stick_actions(self.tv_stick, x, y);
                        self.tv_stick = state;
                        actions.extend(acts);
                    }
                    _ => {}
                }
            }
        }
        actions
    }

    fn tv_activate(&mut self, item: &TvItem) {
        match item {
            TvItem::Rrs(name) => {
                if let Some(app) = self.apps.iter().find(|a| &a.full_name == name).cloned() {
                    self.launch_installed(&app);
                }
            }
            TvItem::External(game) => {
                (self.status_msg, self.status_is_error) = match launchers::launch(game) {
                    Ok(()) => (format!("▶ {} ({})", game.name, game.store.label()), false),
                    Err(e) => (format!("{e:#}"), true),
                };
            }
        }
    }

    pub(super) fn render_tv(&mut self, ctx: &egui::Context) {
        let items = self.tv_items();
        let cols = ((ctx.screen_rect().width() - 80.0) / 300.0).floor().max(1.0) as usize;
        for action in self.poll_tv_input(ctx) {
            match action {
                TvAction::Back => {
                    self.exit_tv_mode(ctx);
                    return;
                }
                TvAction::Activate => {
                    if let Some(item) = items.get(self.tv_index).cloned() {
                        self.tv_activate(&item);
                    }
                }
                other => self.tv_index = tv_move(self.tv_index, items.len(), cols, other),
            }
        }
        // O controle não gera eventos de janela: a tela consulta o gilrs no ritmo do perfil de desempenho
        ctx.request_repaint_after(self.performance_mode.gamepad_poll_interval());

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(Color32::from_rgb(8, 10, 16)).inner_margin(egui::Margin::same(40)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    crate::brand::logo_badge(ui, 26.0);
                    ui.add_space(12.0);
                    ui.label(RichText::new(self.tr("🎮 Modo TV")).size(34.0).strong().color(Color32::WHITE));
                    ui.add_space(20.0);
                    let pads = self.gamepads.as_ref().map(|g| g.gamepads().count()).unwrap_or(0);
                    ui.label(
                        RichText::new(format!("{pads} {}", self.tr("controle(s) conectado(s)")))
                            .size(16.0)
                            .color(Color32::GRAY),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(RichText::new(self.tr("✖ Sair do modo TV")).size(16.0)).clicked() {
                            self.exit_tv_mode(ui.ctx());
                        }
                    });
                });
                ui.label(
                    RichText::new(self.tr("Setas ou direcional: mover · Enter ou A: abrir · Esc ou B: sair"))
                        .size(16.0)
                        .color(Color32::from_rgb(150, 160, 180)),
                );
                ui.add_space(20.0);
                if items.is_empty() {
                    ui.label(
                        RichText::new(self.tr("Nada instalado ainda. Baixe um jogo na loja ou instale pela Steam, Epic ou GOG."))
                            .size(20.0)
                            .color(Color32::GRAY),
                    );
                    return;
                }
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (row_i, row) in items.chunks(cols).enumerate() {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 20.0;
                            for (col_i, item) in row.iter().enumerate() {
                                let idx = row_i * cols + col_i;
                                let selected = idx == self.tv_index;
                                let (title, subtitle) = match item {
                                    TvItem::Rrs(name) => (friendly_title(name), "RRS".to_string()),
                                    TvItem::External(g) => (g.name.clone(), g.store.label().to_string()),
                                };
                                let resp = ui
                                    .vertical(|ui| {
                                        egui::Frame::NONE
                                            .fill(if selected { Color32::from_rgb(35, 70, 130) } else { Color32::from_rgb(22, 26, 36) })
                                            .stroke(Stroke::new(
                                                if selected { 4.0_f32 } else { 1.0_f32 },
                                                if selected { Color32::from_rgb(120, 190, 255) } else { Color32::from_rgb(40, 46, 60) },
                                            ))
                                            .corner_radius(CornerRadius::same(14))
                                            .inner_margin(egui::Margin::same(18))
                                            .show(ui, |ui| {
                                                ui.set_min_size(Vec2::new(260.0, 120.0));
                                                ui.set_max_width(260.0);
                                                ui.label(RichText::new(&title).size(22.0).strong().color(Color32::WHITE));
                                                ui.label(RichText::new(&subtitle).size(15.0).color(Color32::from_rgb(160, 175, 200)));
                                            })
                                            .response
                                    })
                                    .inner;
                                if selected {
                                    ui.scroll_to_rect(resp.rect, None);
                                }
                                if super::clickable_card(resp, &title).clicked() {
                                    self.tv_index = idx;
                                    self.tv_activate(item);
                                }
                            }
                        });
                        ui.add_space(20.0);
                    }
                });
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navegacao_na_grade() {
        // 7 itens em 3 colunas: 0 1 2 / 3 4 5 / 6
        assert_eq!(tv_move(0, 7, 3, TvAction::Left), 0);
        assert_eq!(tv_move(0, 7, 3, TvAction::Right), 1);
        assert_eq!(tv_move(2, 7, 3, TvAction::Down), 5);
        assert_eq!(tv_move(5, 7, 3, TvAction::Down), 5, "não há item abaixo do 5");
        assert_eq!(tv_move(3, 7, 3, TvAction::Down), 6);
        assert_eq!(tv_move(6, 7, 3, TvAction::Up), 3);
        assert_eq!(tv_move(1, 7, 3, TvAction::Up), 1);
        assert_eq!(tv_move(6, 7, 3, TvAction::Right), 6);
        assert_eq!(tv_move(9, 7, 3, TvAction::Activate), 6, "índice fora da lista volta para o último");
        assert_eq!(tv_move(0, 0, 3, TvAction::Right), 0);
    }

    #[test]
    fn analogico_so_gera_acao_ao_sair_do_centro() {
        let (s, a) = stick_actions((0, 0), 0.9, 0.0);
        assert_eq!(a, vec![TvAction::Right]);
        let (s, a) = stick_actions(s, 0.95, 0.1);
        assert!(a.is_empty(), "segurando para a direita não repete");
        let (s, a) = stick_actions(s, 0.0, 0.0);
        assert!(a.is_empty());
        let (_, a) = stick_actions(s, 0.0, -0.8);
        assert_eq!(a, vec![TvAction::Down]);
    }
}
