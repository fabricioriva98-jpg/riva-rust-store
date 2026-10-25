//! Sistema de Temas e Cores Personalizáveis da RRS.
//! Suporta múltiplos temas (Escuro Clássico, Antigravity Deep Space, Cyberpunk Neon, Matrix Hacker, Dracula Synthwave e Modo Claro)
//! e paletas de cores de destaque selecionáveis em tempo real.

use eframe::egui::{self, Color32, Stroke, Visuals};

pub const SETTING_THEME: &str = "tema";
pub const SETTING_ACCENT: &str = "cor_destaque";

/// Temas visuais completos da loja.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppTheme {
    #[default]
    DarkClassic,
    Antigravity,
    CyberpunkNeon,
    MatrixHacker,
    DraculaSynthwave,
    LightClean,
}

impl AppTheme {
    pub const ALL: &'static [AppTheme] = &[
        AppTheme::DarkClassic,
        AppTheme::Antigravity,
        AppTheme::CyberpunkNeon,
        AppTheme::MatrixHacker,
        AppTheme::DraculaSynthwave,
        AppTheme::LightClean,
    ];

    pub fn id(self) -> &'static str {
        match self {
            AppTheme::DarkClassic => "dark_classic",
            AppTheme::Antigravity => "antigravity",
            AppTheme::CyberpunkNeon => "cyberpunk_neon",
            AppTheme::MatrixHacker => "matrix_hacker",
            AppTheme::DraculaSynthwave => "dracula_synthwave",
            AppTheme::LightClean => "light_clean",
        }
    }

    pub fn from_id(id: &str) -> Self {
        match id.trim().to_lowercase().as_str() {
            "antigravity" => AppTheme::Antigravity,
            "cyberpunk" | "cyberpunk_neon" | "neon" => AppTheme::CyberpunkNeon,
            "matrix" | "matrix_hacker" | "hacker" => AppTheme::MatrixHacker,
            "dracula" | "dracula_synthwave" | "synthwave" => AppTheme::DraculaSynthwave,
            "light" | "light_clean" | "claro" => AppTheme::LightClean,
            _ => AppTheme::DarkClassic,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            AppTheme::DarkClassic => "🦀 Escuro Clássico (Rust Dark)",
            AppTheme::Antigravity => "🌌 Antigravity (Deep Space)",
            AppTheme::CyberpunkNeon => "⚡ Cyberpunk 2077 (Neon High-Tech)",
            AppTheme::MatrixHacker => "🟢 Matrix Terminal (Phosphor)",
            AppTheme::DraculaSynthwave => "🔮 Dracula Synthwave (Violeta)",
            AppTheme::LightClean => "☀️ Claro (Clean Light)",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            AppTheme::DarkClassic => "Visual dark grafite clássico com os acentos oficiais em tom ferrugem de Rust.",
            AppTheme::Antigravity => "Inspirado no Google Antigravity: azul cósmico profundo com acabamento futurista.",
            AppTheme::CyberpunkNeon => "Estilo Cyberpunk Neon: preto absoluto, ciano elétrico, magenta vibrante e contornos brilhantes.",
            AppTheme::MatrixHacker => "Visual de terminal retrô: preto fosco com fósforo verde neon e código puro.",
            AppTheme::DraculaSynthwave => "Tons elegantes de violeta noturno, lilás e rosa neon no estilo synthwave.",
            AppTheme::LightClean => "Tema claro, limpo e de alto contraste para ambientes com muita luz.",
        }
    }

    /// Retorna a paleta completa de cores de acordo com o tema e a cor de realce escolhida.
    pub fn palette(self, accent: AccentColor) -> ThemePalette {
        let acc = accent.color(self);

        match self {
            AppTheme::DarkClassic => ThemePalette {
                bg_app: Color32::from_rgb(14, 16, 22),
                bg_sidebar: Color32::from_rgb(18, 21, 28),
                bg_topbar: Color32::from_rgb(20, 24, 32),
                bg_card: Color32::from_rgb(21, 25, 34),
                bg_card_hover: Color32::from_rgb(28, 34, 46),
                bg_status: Color32::from_rgb(16, 18, 24),
                border_subtle: Color32::from_rgb(32, 38, 50),
                border_card: Color32::from_rgb(38, 46, 62),
                border_accent: acc,
                text_primary: Color32::from_rgb(240, 243, 248),
                text_secondary: Color32::from_rgb(160, 172, 192),
                text_muted: Color32::from_rgb(110, 122, 142),
                accent: acc,
                accent_hover: accent.hover_color(self),
                accent_bg: Color32::from_rgba_unmultiplied(acc.r(), acc.g(), acc.b(), 40),
                is_light: false,
            },
            AppTheme::Antigravity => ThemePalette {
                bg_app: Color32::from_rgb(10, 14, 23),
                bg_sidebar: Color32::from_rgb(13, 19, 32),
                bg_topbar: Color32::from_rgb(15, 22, 38),
                bg_card: Color32::from_rgb(18, 26, 44),
                bg_card_hover: Color32::from_rgb(26, 38, 64),
                bg_status: Color32::from_rgb(11, 16, 28),
                border_subtle: Color32::from_rgb(28, 42, 68),
                border_card: Color32::from_rgb(34, 52, 84),
                border_accent: acc,
                text_primary: Color32::from_rgb(245, 248, 255),
                text_secondary: Color32::from_rgb(165, 185, 215),
                text_muted: Color32::from_rgb(105, 125, 155),
                accent: acc,
                accent_hover: accent.hover_color(self),
                accent_bg: Color32::from_rgba_unmultiplied(acc.r(), acc.g(), acc.b(), 45),
                is_light: false,
            },
            AppTheme::CyberpunkNeon => {
                // No tema Cyberpunk, usamos acentos de alto contraste neon com preto profundo
                let cyan_neon = Color32::from_rgb(0, 240, 255);
                let magenta_neon = Color32::from_rgb(255, 0, 128);
                let active_acc = if accent == AccentColor::RustOrange {
                    cyan_neon
                } else {
                    acc
                };
                ThemePalette {
                    bg_app: Color32::from_rgb(6, 7, 12),
                    bg_sidebar: Color32::from_rgb(10, 11, 18),
                    bg_topbar: Color32::from_rgb(12, 14, 22),
                    bg_card: Color32::from_rgb(14, 16, 26),
                    bg_card_hover: Color32::from_rgb(24, 28, 44),
                    bg_status: Color32::from_rgb(8, 9, 15),
                    border_subtle: Color32::from_rgb(45, 25, 60),
                    border_card: Color32::from_rgb(0, 180, 210),
                    border_accent: magenta_neon,
                    text_primary: Color32::from_rgb(255, 255, 255),
                    text_secondary: Color32::from_rgb(0, 235, 255),
                    text_muted: Color32::from_rgb(180, 130, 220),
                    accent: active_acc,
                    accent_hover: Color32::from_rgb(255, 230, 0), // Amarelo Cyberpunk
                    accent_bg: Color32::from_rgba_unmultiplied(active_acc.r(), active_acc.g(), active_acc.b(), 55),
                    is_light: false,
                }
            }
            AppTheme::MatrixHacker => {
                let green_neon = Color32::from_rgb(0, 255, 102);
                ThemePalette {
                    bg_app: Color32::from_rgb(5, 8, 5),
                    bg_sidebar: Color32::from_rgb(8, 14, 8),
                    bg_topbar: Color32::from_rgb(10, 16, 10),
                    bg_card: Color32::from_rgb(12, 20, 12),
                    bg_card_hover: Color32::from_rgb(18, 32, 18),
                    bg_status: Color32::from_rgb(6, 10, 6),
                    border_subtle: Color32::from_rgb(20, 48, 20),
                    border_card: Color32::from_rgb(28, 70, 28),
                    border_accent: green_neon,
                    text_primary: Color32::from_rgb(220, 255, 230),
                    text_secondary: Color32::from_rgb(120, 220, 140),
                    text_muted: Color32::from_rgb(70, 140, 85),
                    accent: if accent == AccentColor::RustOrange { green_neon } else { acc },
                    accent_hover: Color32::from_rgb(100, 255, 160),
                    accent_bg: Color32::from_rgba_unmultiplied(0, 255, 102, 45),
                    is_light: false,
                }
            }
            AppTheme::DraculaSynthwave => {
                let purple_accent = Color32::from_rgb(189, 147, 249);
                ThemePalette {
                    bg_app: Color32::from_rgb(24, 20, 36),
                    bg_sidebar: Color32::from_rgb(30, 24, 45),
                    bg_topbar: Color32::from_rgb(34, 28, 52),
                    bg_card: Color32::from_rgb(40, 32, 60),
                    bg_card_hover: Color32::from_rgb(52, 42, 78),
                    bg_status: Color32::from_rgb(20, 16, 30),
                    border_subtle: Color32::from_rgb(58, 46, 85),
                    border_card: Color32::from_rgb(76, 60, 110),
                    border_accent: purple_accent,
                    text_primary: Color32::from_rgb(248, 248, 242),
                    text_secondary: Color32::from_rgb(255, 121, 198),
                    text_muted: Color32::from_rgb(139, 130, 170),
                    accent: if accent == AccentColor::RustOrange { purple_accent } else { acc },
                    accent_hover: Color32::from_rgb(255, 121, 198),
                    accent_bg: Color32::from_rgba_unmultiplied(189, 147, 249, 45),
                    is_light: false,
                }
            }
            AppTheme::LightClean => ThemePalette {
                bg_app: Color32::from_rgb(244, 246, 250),
                bg_sidebar: Color32::from_rgb(235, 238, 245),
                bg_topbar: Color32::from_rgb(238, 242, 248),
                bg_card: Color32::from_rgb(255, 255, 255),
                bg_card_hover: Color32::from_rgb(240, 244, 252),
                bg_status: Color32::from_rgb(230, 234, 242),
                border_subtle: Color32::from_rgb(215, 222, 232),
                border_card: Color32::from_rgb(200, 210, 225),
                border_accent: acc,
                text_primary: Color32::from_rgb(26, 32, 44),
                text_secondary: Color32::from_rgb(74, 85, 104),
                text_muted: Color32::from_rgb(120, 135, 155),
                accent: acc,
                accent_hover: accent.hover_color(self),
                accent_bg: Color32::from_rgba_unmultiplied(acc.r(), acc.g(), acc.b(), 35),
                is_light: true,
            },
        }
    }

    /// Aplica as propriedades globais do egui para o tema selecionado.
    pub fn apply(self, ctx: &egui::Context, accent: AccentColor) {
        let palette = self.palette(accent);
        let mut visuals = if palette.is_light {
            Visuals::light()
        } else {
            Visuals::dark()
        };

        visuals.window_fill = palette.bg_card;
        visuals.panel_fill = palette.bg_app;
        visuals.extreme_bg_color = palette.bg_sidebar;
        visuals.override_text_color = Some(palette.text_primary);

        visuals.widgets.noninteractive.bg_fill = palette.bg_card;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, palette.border_subtle);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, palette.text_secondary);

        visuals.widgets.inactive.bg_fill = palette.bg_card;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, palette.border_card);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, palette.text_primary);

        visuals.widgets.hovered.bg_fill = palette.bg_card_hover;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.5_f32, palette.accent);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, palette.text_primary);

        visuals.widgets.active.bg_fill = palette.accent;
        visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, palette.accent_hover);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);

        visuals.selection.bg_fill = palette.accent;
        visuals.selection.stroke = Stroke::new(1.0_f32, Color32::WHITE);

        ctx.set_visuals(visuals);
    }
}

/// Cores de destaque (Accent Colors) para botões, bordas ativas e realces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AccentColor {
    #[default]
    RustOrange,
    NeonCyan,
    CyberMagenta,
    MatrixGreen,
    ElectricBlue,
    GoldAmber,
    PurpleNeon,
}

impl AccentColor {
    pub const ALL: &'static [AccentColor] = &[
        AccentColor::RustOrange,
        AccentColor::NeonCyan,
        AccentColor::CyberMagenta,
        AccentColor::MatrixGreen,
        AccentColor::ElectricBlue,
        AccentColor::GoldAmber,
        AccentColor::PurpleNeon,
    ];

    pub fn id(self) -> &'static str {
        match self {
            AccentColor::RustOrange => "rust_orange",
            AccentColor::NeonCyan => "neon_cyan",
            AccentColor::CyberMagenta => "cyber_magenta",
            AccentColor::MatrixGreen => "matrix_green",
            AccentColor::ElectricBlue => "electric_blue",
            AccentColor::GoldAmber => "gold_amber",
            AccentColor::PurpleNeon => "purple_neon",
        }
    }

    pub fn from_id(id: &str) -> Self {
        match id.trim().to_lowercase().as_str() {
            "neon_cyan" | "cyan" => AccentColor::NeonCyan,
            "cyber_magenta" | "magenta" | "pink" => AccentColor::CyberMagenta,
            "matrix_green" | "green" => AccentColor::MatrixGreen,
            "electric_blue" | "blue" => AccentColor::ElectricBlue,
            "gold_amber" | "gold" | "yellow" => AccentColor::GoldAmber,
            "purple_neon" | "purple" => AccentColor::PurpleNeon,
            _ => AccentColor::RustOrange,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            AccentColor::RustOrange => "🦀 Laranja Rust (Oficial)",
            AccentColor::NeonCyan => "💎 Ciano Neon (Antigravity)",
            AccentColor::CyberMagenta => "💖 Magenta Cyberpunk",
            AccentColor::MatrixGreen => "❇️ Verde Matrix",
            AccentColor::ElectricBlue => "⚡ Azul Elétrico",
            AccentColor::GoldAmber => "👑 Dourado Âmbar",
            AccentColor::PurpleNeon => "🔮 Roxo Neon",
        }
    }

    pub fn color(self, _theme: AppTheme) -> Color32 {
        match self {
            AccentColor::RustOrange => Color32::from_rgb(225, 95, 30),
            AccentColor::NeonCyan => Color32::from_rgb(0, 229, 255),
            AccentColor::CyberMagenta => Color32::from_rgb(255, 0, 128),
            AccentColor::MatrixGreen => Color32::from_rgb(0, 230, 90),
            AccentColor::ElectricBlue => Color32::from_rgb(41, 121, 255),
            AccentColor::GoldAmber => Color32::from_rgb(255, 179, 0),
            AccentColor::PurpleNeon => Color32::from_rgb(180, 70, 240),
        }
    }

    pub fn hover_color(self, _theme: AppTheme) -> Color32 {
        match self {
            AccentColor::RustOrange => Color32::from_rgb(245, 120, 50),
            AccentColor::NeonCyan => Color32::from_rgb(80, 240, 255),
            AccentColor::CyberMagenta => Color32::from_rgb(255, 60, 160),
            AccentColor::MatrixGreen => Color32::from_rgb(50, 255, 130),
            AccentColor::ElectricBlue => Color32::from_rgb(80, 150, 255),
            AccentColor::GoldAmber => Color32::from_rgb(255, 200, 50),
            AccentColor::PurpleNeon => Color32::from_rgb(205, 105, 255),
        }
    }
}

/// Paleta completa de cores de um tema com acento.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ThemePalette {
    pub bg_app: Color32,
    pub bg_sidebar: Color32,
    pub bg_topbar: Color32,
    pub bg_card: Color32,
    pub bg_card_hover: Color32,
    pub bg_status: Color32,
    pub border_subtle: Color32,
    pub border_card: Color32,
    pub border_accent: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    pub accent_bg: Color32,
    pub is_light: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn todos_os_temas_geram_paletas_validas() {
        for theme in AppTheme::ALL {
            for accent in AccentColor::ALL {
                let p = theme.palette(*accent);
                assert_ne!(p.bg_app, Color32::TRANSPARENT);
                assert_ne!(p.bg_sidebar, Color32::TRANSPARENT);
                assert_ne!(p.accent, Color32::TRANSPARENT);
                assert_ne!(p.text_primary, p.bg_app);
            }
        }
    }

    #[test]
    fn conversao_de_id_de_tema_e_cor_funciona() {
        assert_eq!(AppTheme::from_id("cyberpunk"), AppTheme::CyberpunkNeon);
        assert_eq!(AppTheme::from_id("antigravity"), AppTheme::Antigravity);
        assert_eq!(AppTheme::from_id("matrix"), AppTheme::MatrixHacker);
        assert_eq!(AppTheme::from_id("dracula"), AppTheme::DraculaSynthwave);
        assert_eq!(AppTheme::from_id("light"), AppTheme::LightClean);
        assert_eq!(AppTheme::from_id("desconhecido"), AppTheme::DarkClassic);

        assert_eq!(AccentColor::from_id("neon_cyan"), AccentColor::NeonCyan);
        assert_eq!(AccentColor::from_id("cyber_magenta"), AccentColor::CyberMagenta);
        assert_eq!(AccentColor::from_id("green"), AccentColor::MatrixGreen);
    }
}
