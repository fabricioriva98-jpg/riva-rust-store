//! Gerenciador de Perfis de Desempenho da RRS.
//! Permite alternar a taxa de renderização (FPS) e o consumo de recursos da interface entre
//! Modo Eco (economia de bateria/CPU), Modo Normal (60 FPS equilibrado) e Modo Performance (144 FPS+ alta fluidez).

use std::time::Duration;

pub const SETTING_PERFORMANCE: &str = "modo_desempenho";

/// Perfil de desempenho do aplicativo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PerformanceMode {
    Eco,
    #[default]
    Normal,
    Performance,
}

impl PerformanceMode {
    pub const ALL: &'static [PerformanceMode] = &[
        PerformanceMode::Eco,
        PerformanceMode::Normal,
        PerformanceMode::Performance,
    ];

    pub fn id(self) -> &'static str {
        match self {
            PerformanceMode::Eco => "eco",
            PerformanceMode::Normal => "normal",
            PerformanceMode::Performance => "performance",
        }
    }

    pub fn from_id(id: &str) -> Self {
        match id.trim().to_lowercase().as_str() {
            "eco" | "economia" => PerformanceMode::Eco,
            "performance" | "desempenho" | "alto" => PerformanceMode::Performance,
            _ => PerformanceMode::Normal,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PerformanceMode::Eco => "🍃 Modo Eco (Economia)",
            PerformanceMode::Normal => "⚖️ Modo Normal (Equilibrado)",
            PerformanceMode::Performance => "🚀 Modo Performance (Máximo)",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            PerformanceMode::Eco => "🍃 Eco",
            PerformanceMode::Normal => "⚖️ Normal",
            PerformanceMode::Performance => "🚀 Performance",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            PerformanceMode::Eco => "Reduz a taxa de quadros (~25-30 FPS) e poupa bateria e CPU. Ideal para uso em segundo plano.",
            PerformanceMode::Normal => "Taxa padrão de 60 FPS com animações suaves e consumo equilibrado para o dia a dia.",
            PerformanceMode::Performance => "Fluidez máxima para monitores gamers (120/144 FPS+) e resposta imediata nos cliques e console.",
        }
    }

    /// Retorna o intervalo recomendado entre repaints de tela.
    pub fn frame_interval(self) -> Duration {
        match self {
            PerformanceMode::Eco => Duration::from_millis(40),        // ~25 FPS
            PerformanceMode::Normal => Duration::from_millis(16),     // ~60 FPS
            PerformanceMode::Performance => Duration::from_millis(7), // ~144 FPS
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversoes_de_modo_desempenho_funcionam() {
        assert_eq!(PerformanceMode::from_id("eco"), PerformanceMode::Eco);
        assert_eq!(PerformanceMode::from_id("normal"), PerformanceMode::Normal);
        assert_eq!(PerformanceMode::from_id("performance"), PerformanceMode::Performance);
        assert_eq!(PerformanceMode::from_id("desconhecido"), PerformanceMode::Normal);
    }

    #[test]
    fn frame_intervals_sao_consistentes() {
        assert!(PerformanceMode::Performance.frame_interval() < PerformanceMode::Normal.frame_interval());
        assert!(PerformanceMode::Normal.frame_interval() < PerformanceMode::Eco.frame_interval());
    }
}
