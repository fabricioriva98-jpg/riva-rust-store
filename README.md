# 🦀 RRS - Riva Rust Store

> **Loja & Launcher Verificado de Aplicativos e Jogos do GitHub + Tradutor C/C++ para Rust**  
> Desenvolvido 100% nativo em **Rust** (`egui` / `eframe` / `rusqlite` / `tokio`).

[![Rust](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)
[![GUI](https://img.shields.io/badge/GUI-egui%200.31-blue.svg)](https://github.com/emilk/egui)
[![Catalog](https://img.shields.io/badge/Catalog-75%20Verified%20Apps%20%26%20Games-green.svg)](#catálogo)
[![License](https://img.shields.io/badge/License-MIT%2FApache-lightgrey.svg)](LICENSE)

---

## 🌟 O que é a RRS?

A **RRS (Riva Rust Store)** é uma central desktop moderna e ultrarrápida que combina:
1. **Loja de Aplicativos e Jogos (Estilo Steam / Microsoft Store)** com 75+ projetos de código aberto do GitHub verificados com instaladores oficiais para PC.
2. **Launcher de Jogos & Aplicativos**:
   - **Auto-extração inteligente de arquivos `.zip`** (jogos portáteis como *ZDoom*, *Veloren*, *Augustus*, *The Powder Toy* ficam prontos para jogar com 1 clique).
   - **Botão `▶ JOGAR AGORA` / `▶ ABRIR APLICATIVO`**: Inicia o executável configurando a pasta de trabalho correta para assets e sons.
   - **Contador de Horas (Playtime Tracker)**: Registra quanto tempo você jogou cada título e a data da última sessão.
3. **Auditoria de Segurança Local e de Precedentes**:
   - Validação criptográfica de **SHA-256** contra o *digest* oficial do GitHub Releases (arquivos adulterados são descartados).
   - **Varredura nativa com o Windows Defender** (`MpCmdRun.exe`) em 1 clique, sem custos e sem precisar de API keys.
   - Verificação de assinatura digital **Authenticode** do Windows.
4. **Estúdio de Tradução C / C++ ➔ Rust**:
   - Editor split-screen com tradução em tempo real (modos *Safe Rust* e *FFI / ABI C*).
   - Validador sintático oficial via `rustc`.
   - **Gerador de Projetos Cargo**: Cria automaticamente uma pasta completa com `Cargo.toml` e `src/main.rs`, pronta para `cargo run`.
5. **Instalador Oficial do Windows**:
   - Assistente visual de instalação que cria atalhos oficiais na **Área de Trabalho**, no **Menu Iniciar** e adiciona o comando `rrs` ao seu terminal.
6. **Novidades de 24/10/2026** (detalhes em `IDEIAS-MELHORIAS-24.10.26.md`):
   - Seção e selo **🦀 Feito em Rust** (linguagem informada pelo GitHub), coleções curadas e "Lançados recentemente" na vitrine.
   - **Versões anteriores** de cada app, com SHA-256 conferido.
   - **Exportar/importar a biblioteca** para reinstalar em outro PC; o tempo de jogo vai junto.
   - **Modo portátil**: caminhos dos apps relativos à pasta da RRS, então funciona em pendrive mesmo se a letra da unidade mudar.
   - Selo de **atestado de build do GitHub** (indica se o atestado existe); a assinatura pode ser conferida pelo GitHub CLI no botão "🔏 Conferir atestado (gh)".
   - **Fila de downloads** (2 ao mesmo tempo), **Codeberg/Gitea/GitLab** além do GitHub, **catálogo online assinado** (ed25519).
   - **Apps do Windows** comparados com o catálogo, **backup de saves**, busca e instalação pelo **winget**.
   - **Inglês** além do português e nomes para **leitor de tela**.
   - **Jogos da Steam, Epic e GOG** na biblioteca, **Modo TV** com controle, **ferramentas de terminal em Rust** pelo nome do crate, **Windows Sandbox** (Windows Pro) e **tradução C/C++ com IA** (API do Claude, chave do usuário).
7. **Instruções para outras IAs** continuarem o projeto: [`CONTINUIDADE-IA.md`](CONTINUIDADE-IA.md).

---

## 📂 Arquivos Executáveis Prontos para Uso

Na raiz da pasta [`F:\NAS\Cods\Rust\RRS`](file:///f:/NAS/Cods/Rust/RRS):
* **[`RRS - Riva Rust Store.exe`](file:///f:/NAS/Cods/Rust/RRS/RRS%20-%20Riva%20Rust%20Store.exe)**: Executável portátil da loja. Basta dar **dois cliques** para abrir.
* **[`Instalador - RRS Riva Rust Store.exe`](file:///f:/NAS/Cods/Rust/RRS/Instalador%20-%20RRS%20Riva%20Rust%20Store.exe)**: Instalador oficial para instalar no PC e criar os atalhos.

---

## 💻 Uso via Terminal (PowerShell / CMD)

Se você adicionou ao PATH (ou executar na pasta do projeto):

```powershell
# Abrir a Loja Gráfica
rrs

# Buscar no catálogo local por jogos
rrs search --kind game

# Buscar por ports de engenharia reversa
rrs search --kind re-game

# Inspecionar detalhes e notas de segurança de um projeto
rrs show obsproject/obs-studio

# Baixar e validar um aplicativo com verificação de SHA-256
rrs download Devolutions/UniGetUI

# Baixar uma versão anterior específica
rrs download alacritty/alacritty --tag v0.16.1

# Levar a biblioteca para outro PC
rrs library-export --out D:\rrs-biblioteca.json
rrs library-import D:\rrs-biblioteca.json

# Ferramenta de terminal em Rust pelo nome do crate (binário pronto, sem compilar)
rrs crate ripgrep

# Jogos instalados pela Steam, Epic e GOG
rrs games

# Tradução com IA (chave em ANTHROPIC_API_KEY ou nas configurações; cada tradução é cobrada)
rrs translate meu_codigo.c --ai -o saida.rs

# Catálogo online assinado
rrs catalog-update --url https://exemplo/catalog.json

# Traduzir um arquivo C++ com validação do compilador rustc
rrs translate meu_codigo.cpp --check

# Traduzir um header C com modo FFI para bindings Rust
rrs translate header.h --ffi -o bindings.rs --check
```

---

## 📖 Documentação de Arquitetura para Desenvolvedores & IAs

Para entender as decisões de design, a estrutura interna do banco SQLite, o drible do rate-limit do GitHub, a manipulação de threads e como estender o projeto, consulte o manual detalhado:
👉 **[ARCHITECTURE.md](ARCHITECTURE.md)**
