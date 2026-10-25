//! Identidade visual da RRS: o logo oficial (selo laranja com o caranguejo branco e "RRS").
//!
//! Um lugar só para o logo: o selo desenhado pela interface (barra lateral, instalador, Modo TV, Sobre)
//! e o ícone da janela vêm daqui. O ícone do `.exe` e dos atalhos é `assets/rrs.ico`, embutido pelo
//! `build.rs`. Os arquivos de `assets/` são gerados por `assets/gerar_logo.py` com as mesmas cores.

use eframe::egui::{self, Color32, CornerRadius, RichText};
use std::sync::Arc;

/// Laranja do selo (o mesmo de `assets/gerar_logo.py`).
pub const LOGO_ORANGE: Color32 = Color32::from_rgb(215, 75, 40);

/// Logo quadrado 512×512 (caranguejo em cima, "RRS" embaixo).
pub const LOGO_PNG: &[u8] = include_bytes!("../assets/logo.png");

/// Ícone da janela e da barra de tarefas.
pub fn app_icon() -> Arc<egui::IconData> {
    Arc::new(eframe::icon_data::from_png_bytes(LOGO_PNG).unwrap_or_default())
}

/// Selo do logo desenhado pela interface (nítido em qualquer escala). `size` é o tamanho do texto.
pub fn logo_badge(ui: &mut egui::Ui, size: f32) -> egui::Response {
    let pad_x = (size * 0.6).round() as i8;
    let pad_y = (size * 0.45).round() as i8;
    egui::Frame::NONE
        .fill(LOGO_ORANGE)
        .corner_radius(CornerRadius::same((size * 0.5).round() as u8))
        .inner_margin(egui::Margin::symmetric(pad_x, pad_y))
        .show(ui, |ui| {
            ui.label(RichText::new("🦀 RRS").size(size).strong().color(Color32::WHITE));
        })
        .response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icone_da_janela_e_o_logo_quadrado() {
        let icon = app_icon();
        assert_eq!((icon.width, icon.height), (512, 512), "o logo.png deve ser 512x512");
        assert_eq!(icon.rgba.len(), 512 * 512 * 4);
        // O centro do topo é o fundo laranja do selo
        let i = ((40 * 512) + 256) * 4;
        let px = &icon.rgba[i..i + 3];
        assert_eq!(px, [LOGO_ORANGE.r(), LOGO_ORANGE.g(), LOGO_ORANGE.b()], "cor do logo diferente de LOGO_ORANGE");
    }

    #[test]
    fn icone_do_exe_tem_todos_os_tamanhos() {
        let ico = include_bytes!("../assets/rrs.ico");
        // Cabeçalho ICO: reservado 0, tipo 1, quantidade de imagens
        assert_eq!(&ico[0..4], &[0, 0, 1, 0]);
        let count = u16::from_le_bytes([ico[4], ico[5]]) as usize;
        let sizes: Vec<u32> = (0..count)
            .map(|k| match ico[6 + k * 16] {
                0 => 256,
                n => n as u32,
            })
            .collect();
        for s in [16, 32, 48, 256] {
            assert!(sizes.contains(&s), "rrs.ico sem o tamanho {s}: {sizes:?}");
        }
    }
}
