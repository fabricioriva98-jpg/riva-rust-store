# 🦀 RRS - Riva Rust Store

> **Central Desktop de Aplicativos e Ferramentas Open Source, Ambiente de Aprendizado Interativo & IDE Rust**  
> Desenvolvido 100% nativo em **Rust** com [`egui`](https://github.com/emilk/egui) / [`eframe`](https://github.com/emilk/egui/tree/master/crates/eframe), [`rusqlite`](https://github.com/rusqlite/rusqlite) e [`tokio`](https://github.com/tokio-rs/tokio).

[![Rust](https://img.shields.io/badge/Language-Rust%202021-orange.svg)](https://www.rust-lang.org/)
[![GUI](https://img.shields.io/badge/GUI-egui%200.31%20(Native)-blue.svg)](https://github.com/emilk/egui)
[![Catalog](https://img.shields.io/badge/Catalog-170%20Verified%20Apps-green.svg)](#-catálogo-e-gerenciador-de-aplicativos)
[![License](https://img.shields.io/badge/License-MIT-brightgreen.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%20x64-lightgrey.svg)](#-instalação-e-execução)

---

## 🌟 Visão Geral

A **RRS (Riva Rust Store)** é uma estação de trabalho completa desenvolvida para a comunidade Rust e entusiastas de software livre. Combinando uma loja de aplicativos verificados do ecossistema GitHub com uma central integrada de aprendizado e desenvolvimento em Rust, a RRS é **ultrarrápida**, inicia instantaneamente, consome menos de 35 MB de memória RAM e não usa Electron.

---

## 🚀 Principais Funcionalidades

### 🏪 1. Catálogo e Gerenciador de Aplicativos
* **170 Aplicativos e Jogos Curados**: Softwares utilitários de alta performance (ripgrep, bat, eza, zellij, fd, alacritty, nushell, uv, ollama, etc.) e jogos de código aberto.
* **Segurança e Integridade Criptográfica**:
  * Conferência automática de **SHA-256** contra os *digests* oficiais das Releases do GitHub.
  * Validação de atestados de compilação oficial (*Build Attestation*) via GitHub CLI (`gh attestation verify`).
  * Varredura sob demanda com o **Windows Defender** nativo em 1 clique.
  * Inspeção de assinaturas digitais **Authenticode**.
* **Instalação e Execução Inteligente**:
  * Extração e configuração automática de arquivos compactados (`.zip`, `.tar.gz`) e instaladores (`.msi`, `.exe`).
  * Modo isolado via **Windows Sandbox** (`.wsb`) para testar softwares em ambiente virtual descartável.
  * Rastreador local de tempo de uso (*playtime tracker*) e sistema de backup seguro de dados/saves.

---

### 📖 2. Estúdio Aprender Rust (Iniciante ao Avançado)
* **30 Lições Práticas Integradas**:
  * 🐣 **Básico (1-10)**: Sintaxe, tipos primitivos, variáveis, controle de fluxo, funções, ownership e borrowing.
  * 🚀 **Intermediário (11-20)**: Structs, enums, pattern matching, error handling (`Result`/`Option`), traits, generics e closures.
  * 🔥 **Avançado (21-30)**: Concorrência com `Arc<Mutex<T>>` e `RwLock<T>`, lifetimes avançados, metaprogramação com `macro_rules!`, typestate pattern, FFI com C e pointers brutos em Unsafe Rust.
* **Compilação e Execução Direta na Interface**:
  * Compilação local instantânea usando o compilador oficial `rustc`.
  * Painel de diagnóstico inteligente de erros com orientações e dicas práticas.
  * Botão de **Exportar Projeto Cargo**: Transforma a lição em um projeto Cargo completo (`Cargo.toml` + `src/main.rs`) pronto para o terminal.

---

### 📚 3. Biblioteca Rust (std & 80+ Crates Oficiais)
* Referência técnica offline rápida dos módulos da biblioteca padrão (`std::collections`, `std::sync`, `std::fs`, `std::net`, `std::time`, etc.).
* Documentação sintetizada, casos de uso e snippets funcionais das **80+ crates mais importantes do ecossistema**:
  * *Serialização & Web*: `serde`, `serde_json`, `tokio`, `axum`, `reqwest`, `tower`.
  * *Concorrência & Dados*: `rayon`, `crossbeam`, `dashmap`, `parking_lot`, `flume`.
  * *CLI & Terminal*: `clap`, `dialoguer`, `indicatif`, `crossterm`.
  * *Utilitários*: `anyhow`, `thiserror`, `tracing`, `rand`, `uuid`, `dotenvy`.
  * *TUI & Terminal*: `ratatui`, `crossterm`, `indicatif`, `dialoguer`.
  * *Bancos de Dados & ORM*: `sqlx`, `rusqlite`, `diesel`, `sea-orm`.
  * *Ciência de Dados & ML*: `polars`, `ndarray`, `candle-core`.
  * *Parsers, Macros & Metaprogramação*: `nom`, `syn`, `quote`.
  * *Segurança & Criptografia*: `rustls`, `ring`, `zeroize`.
  * *Testes Avançados*: `criterion`, `mockall`, `proptest`.

---

### 💻 4. Terminal & IDE Rust Integrada
* **Editor de Código Expansível**:
  * Realce de sintaxe nativo, barra de ferramentas rápida com templates de código injetáveis.
  * Modos de visualização flexíveis: tela dividida proporcional (Split 65/35) ou **Modo Maximizado (100% de área de trabalho)**.
  * **Formatação Automática com `rustfmt`**: Formata e indenta o código do editor com 1 clique.
* **Console Interativo de Terminal**:
  * Buffer estilizado com colorização inteligente de mensagens do Cargo (`Compiling`, `warning:`, `error:`).
  * Histórico de comandos interativo navegável pelas teclas **Seta para Cima (↑)** e **Seta para Baixo (↓)**.
  * Atalhos rápidos: `cargo check`, `cargo build`, `cargo run`, `cargo test`, `cargo clippy`, `cargo clean`.
* **Espelho de IDEs Externas**:
  * Detecção e abertura imediata do projeto no **VS Code**, no **Google Antigravity**, no Windows Terminal ou no Explorador de Arquivos.

---

### 🔄 5. Tradutor C/C++ para Rust
* Tradução estática offline por padrões sintáticos (Safe Rust e bindings FFI `#[repr(C)]`).
* Validação de sintaxe gerada em tempo real pelo `rustc`.
* Suporte opcional à tradução com IA (API Claude com chave própria do usuário e laço de auto-correção via compilador).

---

### 🛡️ 6. Privacidade e Não-Invasividade Total
* **Sem Telemetria Oculta**: Seus dados, comandos, caminhos e chaves nunca saem da sua máquina.
* **Zero Invasão de Sistema**: O aplicativo não altera registros do Windows, não muda planos de energia e não monitora outros processos do computador.
* **100% Portátil**: Funciona diretamente da pasta ou de um pendrive sem exigir instalação no sistema operacional.

---

## 📦 Instalação e Execução

### Opção 1: Baixar os Executáveis Prontos (Windows x64)

Você pode baixar os executáveis diretamente na aba [**Releases**](https://github.com/fabricioriva98-jpg/riva-rust-store/releases) deste repositório:
* **[RRS-Riva-Rust-Store.exe](https://github.com/fabricioriva98-jpg/riva-rust-store/releases/download/v0.3.0/RRS-Riva-Rust-Store.exe)**: Versão portátil oficial. Basta baixar e executar diretamente (não precisa instalar).
* **[Instalador-RRS.exe](https://github.com/fabricioriva98-jpg/riva-rust-store/releases/download/v0.3.0/Instalador-RRS.exe)**: Instalador oficial com assistente de instalação e atalhos.

### Opção 2: Compilar a partir do Código-Fonte

Pré-requisitos:
* [Rust e Cargo](https://rustup.rs/) instalados (versão estável 1.80+).
* Sistema operacional Windows 10/11 (64 bits).

```powershell
# 1. Clonar o repositório
git clone https://github.com/fabricioriva98-jpg/riva-rust-store.git
cd riva-rust-store

# 2. Executar a suíte de testes (120+ testes unitários)
cargo test

# 3. Compilar em modo Release (otimizado)
cargo build --release

# 4. Executar o binário gerado
.\target\release\gitstore.exe
```

---

## 💻 Uso via Linha de Comando (CLI)

O executável da RRS também funciona como uma poderosa ferramenta de linha de comando:

```powershell
# Abrir a interface gráfica principal
rrs

# Buscar aplicativos no catálogo local
rrs search ripgrep
rrs search --kind cli

# Ver detalhes técnicos e notas de segurança de um projeto
rrs show BurntSushi/ripgrep

# Baixar e validar um software com conferência de SHA-256
rrs download BurntSushi/ripgrep

# Baixar uma versão específica de release
rrs download alacritty/alacritty --tag v0.15.0

# Instalar binário de terminal a partir do nome de um crate
rrs crate ripgrep

# Exportar e importar biblioteca de apps entre computadores
rrs library-export --out C:\minha-biblioteca.json
rrs library-import C:\minha-biblioteca.json

# Traduzir código C/C++ para Rust com validação do rustc
rrs translate algoritmo.cpp --check
```

---

## 🏗️ Estrutura do Código e Arquitetura

```
RRS/
├── src/
│   ├── main.rs              # Ponto de entrada CLI (clap) e inicialização da GUI
│   ├── gui.rs               # Interface gráfica principal e loop de eventos (egui)
│   ├── gui/
│   │   ├── learn.rs         # Interface do Estúdio Aprender Rust (30 lições)
│   │   ├── terminal_studio.rs # Editor, Console e Gerenciador de Projetos Cargo
│   │   ├── rust_lib.rs      # Referência visual de 80+ Crates e módulos std
│   │   ├── settings.rs      # Painel de configurações, temas e credenciais
│   │   └── social.rs        # Compartilhamento seguro de biblioteca e discussões
│   ├── db.rs                # Camada SQLite embutida (rusqlite), catálogo e histórico
│   ├── downloader.rs        # Fila assíncrona (tokio), SHA-256, extração de pacotes
│   ├── learn.rs             # Conteúdo das 30 lições de Rust e avaliação no rustc
│   ├── rust_lib.rs          # Catálogo estruturado de crates e documentação
│   ├── terminal_studio.rs   # Runner de comandos de terminal, rustfmt e avaliação REPL
│   ├── translator.rs        # Motor de conversão estática C/C++ -> Rust
│   ├── verify.rs            # Auditoria de segurança, índices de confiança e VirusTotal
│   ├── theme.rs             # Sistema de paletas e temas visuais
│   └── i18n.rs              # Suporte bilíngue (Português / Inglês)
├── Cargo.toml               # Dependências e metadados do pacote
└── LICENSE                  # Licença de código aberto (MIT)
```

Para mais detalhes sobre a modelagem interna do banco, concorrência e estratégias de segurança, consulte [`ARCHITECTURE.md`](ARCHITECTURE.md).

---

## 📄 Licença

Distribuído sob a licença **MIT**. Consulte o arquivo [`LICENSE`](LICENSE) para mais detalhes.

---

Desenvolvido com 🦀 e paixão pelo ecossistema Rust por **Fabricio Riva**.
