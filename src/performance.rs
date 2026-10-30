//! Gerenciador de Perfis de Desempenho da RRS.
//! Parada, a interface não é redesenhada em nenhum perfil (o egui só pinta com entrada do usuário
//! ou evento de tarefa). O perfil decide o ritmo de pintura enquanto há algo em andamento
//! (download, tarefa em segundo plano): Eco economiza, Performance deixa mais fluido.

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
            PerformanceMode::Eco => "Atualiza o progresso de downloads e tarefas com menos frequência. Parada, a loja não usa o processador.",
            PerformanceMode::Normal => "Equilíbrio entre fluidez e consumo durante downloads e tarefas. Parada, a loja não usa o processador.",
            PerformanceMode::Performance => "Progresso de downloads e tarefas mais fluido; gasta mais processador só enquanto algo está em andamento.",
        }
    }

    /// Intervalo entre pinturas enquanto há download ou tarefa em andamento.
    pub fn active_interval(self) -> Duration {
        match self {
            PerformanceMode::Eco => Duration::from_millis(250),
            PerformanceMode::Normal => Duration::from_millis(100),
            PerformanceMode::Performance => Duration::from_millis(33),
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
    fn intervalos_seguem_a_ordem_dos_perfis() {
        let f = PerformanceMode::active_interval;
        assert!(f(PerformanceMode::Performance) < f(PerformanceMode::Normal));
        assert!(f(PerformanceMode::Normal) < f(PerformanceMode::Eco));
    }

    #[test]
    fn nenhum_perfil_pinta_mais_de_60_vezes_por_segundo_em_tarefa() {
        // 33 ms já é fluido para barra de progresso; menos que isso só gastaria CPU
        for m in PerformanceMode::ALL {
            assert!(m.active_interval() >= Duration::from_millis(33), "{m:?}");
        }
    }
}
