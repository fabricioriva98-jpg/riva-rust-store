---
tipo: instrucoes-continuidade
projeto: RRS - Riva Rust Store (gitstore 0.3.0)
atualizado: 2026-10-25
autor: Claude (Opus 5.5) / Antigravity (Gemini 4 Argon)
---
# Instruções para outra IA continuar a RRS

Pedido do Fabricio (24/10/2026): "Quero que voce deixe intruções tambem caso precise colocar outra IA para dar continuidade no projeto se esgotar o limite de uso, porque tenho O Antigravity que tem o Gemini 4 argon pra usar e o Claude Opus 5.5 tambem que posso usar depois".

Este arquivo vale para qualquer IA: Gemini 4 Argon (Antigravity), Claude Opus 5.5 ou outra. Leia tudo antes de alterar qualquer arquivo. Exceção registrada em [[ADR-2026-10-24-b-continuidade-da-rrs-por-outra-ia]]: no resto do vault `F:\NAS`, só o Claude trabalha.

## 1. Regras que não mudam

1. **Nada se apaga.** O que sair do projeto vai para `F:\NAS\Obsoleto\`, com um registro `MOVIMENTACOES-<data>-<assunto>.csv` e, quando der, um script `DESFAZER-<data>-<assunto>.ps1`. Para trocar um arquivo, guarde a cópia antiga antes.
2. **Cópia do código antes de mexer.** Copie `src\`, `Cargo.toml`, `Cargo.lock` e `catalog.json` para `F:\NAS\Obsoleto\gitstore-src-antes-<assunto>-DD.MM.AA\`.
3. **Sem segredos.** Não leia nem mostre `.env`, tokens ou chaves. O `gitstore.db` pode ter chaves salvas na tabela `settings`: só confira se existem, nunca imprima os valores.
4. **Sem publicar sem autorização.** Push, release, deploy, envio de mensagem e qualquer ação financeira só com autorização explícita do Fabricio.
5. **Português**, escrita técnica e objetiva. Fato com fonte; incerteza marcada como `[Não verificado]`, `[Inferência]` ou `[Especulação]`. Ideia sua fica marcada `Proposta (pendente)` até o Fabricio aprovar.
6. **Evite laço.** Se a mesma correção falhar duas vezes, pare e relate o erro ao Fabricio.
7. **Textos de segurança honestos.** Nunca escreva "sem vírus", "100% seguro" ou "verificado" sem uma verificação real por trás (ver a avaliação em `AVALIACAO-24.10.26.md`, achados M1 e M2).
8. **Pasta `WHB\` do vault**: dados industriais, não abrir.

**Gemini 4 Argon no Antigravity**: o Claude tem um bloqueio automático ("guarda do vault") que impede apagar arquivos; o Antigravity não tem. Siga as regras 1 e 2 manualmente: nada de `rm`, `del`, `Remove-Item`, `git clean` ou `git reset --hard` dentro de `F:\NAS`.
**Claude Opus 5.5**: o `F:\NAS\CLAUDE.md` vale para o vault inteiro. A guarda recusa gravar direto em `Obsoleto\`: grave o registro na pasta temporária e mova para lá.

## 2. Ambiente

- Windows 11 Home. Rust estável `x86_64-pc-windows-msvc` (rustc 1.99.1 em 24/10/2026) e `clippy` instalados. O `rustfmt` **não** está instalado: não formate arquivos inteiros, porque o diff fica enorme.
- O projeto fica no NAS (`F:`, exFAT). Desde 25/10 o build vai para **`C:\cargo-target\rrs`** (`target-dir` no `.cargo\config.toml`): não volte a compilar dentro do vault (o Obsidian carregava os 9 mil arquivos) e não recoloque `incremental = false` (no C: o cache incremental funciona). A pasta `target` antiga está em `F:\NAS\Obsoleto\.gitstore-target-antes-m8-25.10.26` (pode ser apagada pelo Fabricio).
- **Git local, sem remoto**: histórico desde 24/10 (reconstruído das cópias do `Obsoleto`, uma por rodada). Ao fim de **cada rodada**, um commit com mensagem `<IA>: <o que mudou>` e a linha `Co-Authored-By` da IA. **Nunca** `git push`, `git reset --hard`, `git clean` nem reescrever commits. O exFAT não grava dono de arquivo, então o git exige `-c safe.directory=F:/NAS/Cods/Rust/RRS` em cada comando (ou o Fabricio roda uma vez `git config --global --add safe.directory F:/NAS/Cods/Rust/RRS`). A identidade (`user.name`/`user.email`) já está no `.git\config` deste repositório. Os `.exe` da raiz e o `gitstore.db` ficam fora do git (`.gitignore`).

## 3. Comandos

```powershell
Set-Location F:\NAS\Cods\Rust\RRS
cargo build                      # desenvolvimento
cargo test                       # 103 testes em 25/10/2026; todos devem passar
cargo clippy --all-targets       # 8 avisos antigos conhecidos; não deixe aparecer aviso novo
cargo build --release            # .exe final (cerca de 4 min no NAS)
```

Teste a interface sempre com banco e catálogo **separados**, para não mexer nos dados do Fabricio:

```powershell
$T = "$env:TEMP\rrs-teste"; New-Item -ItemType Directory -Force $T | Out-Null
Copy-Item .\catalog.json "$T\catalog.json" -Force
C:\cargo-target\rrs\debug\gitstore.exe --db "$T\t.db" --catalog "$T\catalog.json" gui
```

Linha de comando útil: `search`, `show dono/repo`, `verify dono/repo`, `download dono/repo [--tag vX]`, `library-export`, `library-import`, `catalog-keygen | catalog-sign | catalog-update | catalog-refresh`, `translate arquivo.c [--check] [--ai]`, `crate <nome>`, `games`, `check-updates`. Sem `GITHUB_TOKEN`, a API do GitHub dá 60 consultas por hora (cada `verify` gasta 4).

## 4. Mapa do código (`src\`)

| Arquivo | Papel |
|---|---|
| `main.rs` | linha de comando (clap) e abertura |
| `db.rs` | SQLite, importar/exportar `catalog.json`, biblioteca exportável, caminhos portáteis, lista "Para reinstalar" |
| `github.rs` | API do GitHub, escolha do instalador (`pick_best_asset`), atestados |
| `forge.rs` | Codeberg/Forgejo/Gitea e GitLab convertidos para as estruturas do GitHub; nome `host/dono/repo` |
| `verify.rs` | nota de confiança (precedentes) e VirusTotal |
| `downloader.rs` | fila de downloads (2 ao mesmo tempo, registra no banco ao terminar), SHA-256, extração de .zip, abrir apps **sem cmd.exe** (`ShellExecuteW`), Defender e Authenticode |
| `catalog_sign.rs` | catálogo online assinado (ed25519, versão crescente e validade) |
| `installer.rs` | instalador do Windows (marcador `rrs-install.json`, desinstalação só dos arquivos da RRS), PATH |
| `windows_apps.rs` | programas do Windows (registro) comparados com o catálogo |
| `saves.rs` | backup e restauração de saves (PowerShell Compress/Expand-Archive) |
| `winget.rs` | busca e instalação pelo winget (o winget confere o hash) |
| `attestation.rs` | conferência do atestado pelo GitHub CLI (`gh attestation verify`) |
| `sandbox.rs` | abrir instalador no Windows Sandbox (só Pro/Enterprise/Education) |
| `launchers.rs` | jogos da Steam (VDF), Epic (manifestos) e GOG (registro) |
| `crates_io.rs` | nome do crate → repositório → binário pronto; "Adicionar ao terminal" |
| `translator.rs` | tradutor C/C++ → Rust por padrões (experimental, avisa linha não reconhecida) e validação pelo `rustc` |
| `translator_ai.rs` | tradução com a API do Claude (`claude-opus-5-5`, `anthropic-version: 2026-10-21`) e laço do `rustc` |
| `achievements.rs` | conquistas (20), XP, nível (o nível N começa em 100·(N−1)² XP) e título; contadores na tabela `stats`, desbloqueios em `achievements` |
| `github_auth.rs` | login com o GitHub (OAuth Device Flow, escopos `read:user gist`); token no Gerenciador de Credenciais do Windows (crate `keyring`), nunca no banco |
| `social.rs` | perfil público opcional (Gist público `rrs-perfil.json`, sem caminhos, hashes nem chaves), amigos (quem o usuário segue e também publicou), biblioteca por link (Gist secreto) |
| `community.rs` | GitHub Discussions pela API GraphQL (com variáveis): lista e conversa; responder e abrir conversa nova é no site do GitHub |
| `i18n.rs` | idioma (português/inglês): tabela de traduções |
| `rust_lib.rs` | catálogo e referência completa da biblioteca Rust (`std` coleções/concorrência/io/net/process/env/path/time e top crates serde/tokio/reqwest/anyhow/clap/rayon/rand) |
| `ide.rs` | detecção e espelhamento para IDEs externas (VS Code e Google Antigravity), abertura de workspace/arquivos e instalador de extensões |
| `terminal_studio.rs` | execução não-bloqueante de comandos de terminal (cargo, rustc, rustup, git) com histórico categorizado |
| `theme.rs` | sistema de temas visuais (Escuro Clássico, Antigravity Deep Space, Cyberpunk Neon, Matrix Hacker, Dracula Synthwave, Modo Claro) e paleta de cores de destaque |
| `gui.rs` | interface principal (egui); submódulos `gui\extras.rs` (fase 2), `gui\phase3.rs` (fase 3), `gui\tv.rs` (Modo TV), `gui\settings.rs` (página Configurações), `gui\social.rs` (perfil, conquistas, avisos, comunidade, compartilhar), `gui\learn.rs` (Estúdio Aprender Rust), `gui\optimizer.rs` (Otimizador de PC), `gui\rust_lib.rs` (Biblioteca Rust) e `gui\terminal_studio.rs` (Terminal & IDEs) |

## 5. Regras do código (os testes cobram)

- **Texto novo na interface**: escreva em português dentro de `tr(self.lang, "...")` (ou `self.tr("...")`) e acrescente a tradução em `i18n.rs::TABLE`. O teste `todo_texto_traduzivel_tem_ingles` falha se faltar; `tabela_sem_repeticao_nem_sobra` falha se sobrar.
- **Emoji ou símbolo novo**: o teste `todos_os_caracteres_da_interface_tem_glifo` confere se a fonte desenha o caractere (a interface usa as fontes Segoe do Windows como reserva).
- **Nunca** chame `Command::new("cmd")`: o teste `nenhum_codigo_chama_cmd_exe` falha. Use `downloader::shell_open`, `open_url` (só https) ou `open_store_uri` (endereços validados).
- **Downloads** passam pelo `DownloadManager` (`start_download` na interface).
- **Catálogo**: `catalog.json` é embutido no .exe com `include_str!`; depois de editar, compile de novo. Os campos `kind`, `category`, `tags` e `summary` são curadoria manual; `language`, `released_at` e `attestation` vêm das forjas.
- **Conquistas**: ação nova que deve contar XP chama `self.record_stat(achievements::STAT_...)` na interface (ou `db::bump_stat` fora dela); conquista nova entra em `ACHIEVEMENTS` com nome e descrição traduzidos na `TABLE`.
- **Dados de terceiros** (perfil de amigo, conversa do GitHub) não são confiáveis: passam por `social::parse_profile` (corta tamanho, só conquistas conhecidas) e aparecem como texto puro.
- **Token do GitHub**: só pelo `github_auth::save_token` / `load_token`; nunca grave no banco nem em log.
- Funcionalidade nova = teste novo. Rode `cargo test` e `cargo clippy --all-targets` antes de declarar pronto.
- Interface: confira na tela com o banco separado (seção 3) e descreva o que viu.

## 6. Estado em 25/10/2026

**Análise do Claude (25/10, 13:25)**: `ANALISE-25.10.26.md`. Os 3 graves foram **corrigidos às 13:50**, os médios M1–M6 **às 14:10** e M7/M8 (git local e build no C:) **às 14:25** (testes: 137); os baixos B1–B8 continuam pendentes. Regras novas que os testes cobram:
- A interface **não** pede pintura contínua: tarefa em segundo plano entrega o resultado pela fila `BgQueue` (que acorda a tela); `request_repaint_after` só enquanto há download, tarefa (`busy_task`), aviso ou Modo TV, no ritmo de `PerformanceMode::active_interval` / `gamepad_poll_interval`.
- Nada que abre processo ou varre disco roda dentro do desenho da tela (`render_*`): use uma thread e um `BgEvent` (exemplo: `spawn_dev_env_detection`).
- Ação do Otimizador que apaga algo leva `confirm: Some(pergunta)` e só roda no segundo clique; a limpeza do %TEMP% só apaga arquivos da raiz com mais de 7 dias (`optimizer::TEMP_MIN_AGE`).
- Processo de código do usuário ou de comando digitado roda por `subprocess::run_limited` (tempo máximo, Parar, saída limitada); nunca `Command::output()` direto nesses casos.
- O terminal só recebe o campo digitado e os atalhos fixos (`terminal_so_recebe_o_que_o_usuario_digita`).
- Otimizador não inventa valor: dado que não foi lido aparece como `optimizer::NOT_READ`.
- Nenhum caminho deste PC (`E:\Fabricio`, `C:\Users\fabri`) no código (`codigo_distribuido_nao_tem_caminhos_deste_pc`).

Feito e testado: avaliação, correções G1–G3, fases 1, 2 e 3 do `IDEIAS-MELHORIAS-24.10.26.md` (resumo lá), rodada social e identidade visual com espaço educacional interativo. Catálogo com 88 itens (16 em Rust, 6 ferramentas de terminal). Testes automatizados aumentados para 116 (todos passando).

Rodada Expansão Vasta da Biblioteca Rust, Livro Oficial (PDF & Web), Terminal Studio Dedicado & Seletor de Desempenho (25/10, tarde — Antigravity / Gemini):
- **Expansão Vasta da Biblioteca Rust (45+ Módulos e Crates)**: catálogo em `rust_lib.rs` expandido para mais de 45 itens documentados com documentação técnica em português, principais tipos, métodos, dependências para `Cargo.toml` e exemplos executáveis integrados ao editor. Categorias adicionadas: Bancos de Dados (`sqlx`, `rusqlite`), Jogos & Gráficos (`bevy`, `wgpu`), Criptografia & Utilidades (`sha2`, `regex`, `indicatif`, `colored`, `tempfile`, `toml`, `bincode`, `sysinfo`), Coleções avançadas (`HashSet`, `BTreeSet`), Primitivas atômicas lock-free (`AtomicBool`, `AtomicUsize`, `OnceLock`), Mutabilidade interior (`RefCell`, `Cell`, `std::mem`) e Sockets de rede (`TcpListener`, `UdpSocket`).
- **Navegador do Livro Oficial de Rust (The Rust Programming Language - PDF & Web)**: nova aba dedicada em `gui/rust_lib.rs` com índice completo dos 20 capítulos da obra oficial de Steve Klabnik e Carol Nichols. Links diretos para leitura online oficial em Inglês e Português (PT-BR), além da versão para impressão/salvar em PDF com guia para download offline (Ctrl+P no navegador).
- **Terminal Rust Studio Dedicado**: expansão de `terminal_studio.rs` e `gui/terminal_studio.rs` com barra de status de ferramentas do sistema (detecção de `rustc`, `cargo`, `rustup`, `clippy` e `rust-analyzer`), gerador em 1 clique de novos projetos Cargo (`--bin` e `--lib`) e avaliador de expressões instantâneo (Scratchpad REPL) com compilação e execução rápida via `rustc`.
- **Seletor de Perfis de Desempenho (Eco / Normal / Performance)**: novo módulo `performance.rs` gerenciando o frame pacing de renderização e consumo de recursos da GPU/CPU (`Eco` a ~25 FPS para poupar bateria, `Normal` a 60 FPS equilibrado, `Performance` a 144 FPS+ de fluidez máxima). Seletor rápido na barra superior e painel interativo em Configurações, com persistência no banco SQLite (`modo_desempenho`).
- **Qualidade, i18n e Testes**: paridade rigorosa com 26 novos termos em `i18n.rs` (100% de cobertura nos testes de tradução), suite expandida para 116 testes unitários passando (100% de sucesso), 0 novos avisos no Clippy, e binários de release `RRS - Riva Rust Store.exe` e `Instalador - RRS Riva Rust Store.exe` atualizados na raiz com backups prévios em `Obsoleto`.

Rodada Temas Visuais, Cyberpunk Neon & Cores de Destaque (25/10, tarde — Antigravity / Gemini):
- **Sistema de Temas e Cores (`theme.rs`, `gui/settings.rs` e `gui.rs`)**:
  - Implementado motor completo de temas visuais persistido no SQLite (`tema` e `cor_destaque`):
    - 🌌 **Antigravity (Deep Space)**: inspirado no Google Antigravity, com fundo azul espacial cósmico profundo e bordas elegantes.
    - ⚡ **Cyberpunk 2077 (Neon High-Tech)**: preto absoluto de alto contraste (`#06070c`), ciano neon vibrante (`#00f0ff`), magenta neon (`#ff007f`) e realces em amarelo elétrico.
    - 🦀 **Escuro Clássico (Rust Dark)**: fundo grafite escuro com o laranja ferrugem oficial de Rust.
    - 🟢 **Matrix Terminal (Phosphor)**: preto fosco e verde fósforo hacker (`#00ff66`).
    - 🔮 **Dracula Synthwave (Violeta)**: tons noturnos de violeta, lilás e rosa neon.
    - ☀️ **Claro (Clean Light)**: tema limpo de alto contraste para ambientes claros.
  - **7 Cores de Destaque Selecionáveis (Accent Colors)**: Laranja Rust (Oficial), Ciano Neon (Antigravity), Magenta Cyberpunk, Verde Matrix, Azul Elétrico, Dourado Âmbar e Roxo Neon.
  - **Seletor Rápido na Barra Superior**: ComboBox de troca imediata de tema no topo de qualquer tela da loja e painel completo com amostras interativas de cores em Configurações › `🎨 Aparência, Temas & Cores`.
  - **Internacionalização e Testes**: 24 novas entradas em `i18n.rs`, 111 testes automatizados (100% de aprovação), 0 novos avisos no Clippy e executáveis atualizados em release: `RRS - Riva Rust Store.exe` e `Instalador - RRS Riva Rust Store.exe`.

Rodada Biblioteca Rust Completa & Terminal Integrado / Espelho de IDEs (25/10, tarde — Antigravity / Gemini):
- **Biblioteca Rust (`std` & Top Crates do Ecossistema)**: novo módulo `rust_lib.rs` e tela gráfica `gui/rust_lib.rs` com 24 entradas detalhadas cobrindo a Standard Library (`std::collections::{HashMap, BTreeMap, VecDeque, BinaryHeap}`, `std::sync::{Arc, Mutex, RwLock, mpsc}`, `std::fs`, `std::io`, `std::net`, `std::process`, `std::env`, `std::path`, `std::time`) e os principais crates (`serde`, `serde_json`, `tokio`, `reqwest`, `anyhow`, `thiserror`, `clap`, `rayon`, `rand`). Apresenta documentação técnica em português, tipos essenciais, métodos mais usados, snippet para `Cargo.toml` com botão de cópia e exemplos executáveis com botão `▶ Testar no Editor` (injeta direto no Estúdio Aprender Rust).
- **Terminal Integrado & Espelho de IDEs (VS Code / Antigravity)**: novos módulos `ide.rs`, `terminal_studio.rs` e tela gráfica `gui/terminal_studio.rs`. Detecta automaticamente a instalação do Visual Studio Code e do Google Antigravity na máquina do Fabricio. Permite abrir pastas de projeto nas IDEs com 1 clique, instalar a extensão oficial `rust-analyzer` no VS Code, inspecionar arquivos e executar comandos Cargo (`cargo check`, `cargo build`, `cargo run`, `cargo test`, `rustc --version`, `rustup show`, `cargo clean` ou qualquer comando digitado) em tempo real sem travar a interface da loja.
- **Internacionalização e Testes**: paridade rigorosa em `i18n.rs` com 40 novas traduções adicionadas (100% dos testes `todo_texto_traduzivel_tem_ingles`, `tabela_sem_repeticao_nem_sobra` e `troca_de_idioma` aprovados). Suite total expandida para 109 testes (todos passando), 0 novos avisos no Clippy e executáveis atualizados em release: `RRS - Riva Rust Store.exe` e `Instalador - RRS Riva Rust Store.exe`.

Rodada Sincronização de Lojas, Rust Intermediário & Otimizador do PC (25/10, manhã — Antigravity / Gemini):
- **Sincronização Multi-Lojas de Games**: suporte expandido em `launchers.rs` e `gui/phase3.rs` para 8 lojas principais: Steam, Epic Games, GOG, Xbox / Microsoft Store / PC Game Pass, EA App, Ubisoft Connect, Battle.net e Riot Games (Vanguard/LoL/Valorant). Botão de sincronização em 1 clique e filtros por loja com contador dinâmico.
- **Espaço Aprender Rust Intermediário**: expansão de `learn.rs` para 20 lições completas (10 básicas + 10 intermediárias: Ownership & Borrowing, Traits, Generics, Closures, Iteradores avançados, Multithreading com std::thread, Canais mpsc, Smart Pointers Box, Leitura/Escrita de Arquivos fs e Mini-Projeto CLI Gerenciador de Tarefas). Filtros de nível no topo (`Todos`, `Básico`, `Intermediário`), pills visuais e diagnóstico pedagógico com suporte a erros de tempo de vida e bounds de traits.
- **Otimizador do PC & Diagnóstico de Sistema**: novo módulo `optimizer.rs` e tela gráfica `gui/optimizer.rs` com reconhecimento detalhado de hardware (CPU núcleos/threads, RAM total/livre, GPU e versão de driver, Discos locais com espaço e percentual de uso), inventário pesquisável de programas instalados no Windows (registro nativo sem cmd.exe), aplicativos de inicialização e ações de otimização em 1 clique (Plano de Energia de Alto Desempenho / Ultimate Performance via `powercfg /setactive`, Modo Jogo do Windows no registro, Limpeza segura de arquivos temporários `%TEMP%` e recomendações para Rust Dev com Defender).
- **Internacionalização e Qualidade**: paridade completa PT-BR e EN em `i18n.rs` (55 novos termos adicionados, zero sobras), 103 testes automatizados passando (100% de sucesso) e binários de release `RRS - Riva Rust Store.exe` e `Instalador - RRS Riva Rust Store.exe` atualizados na raiz.

Rodada Logo Oficial & Aprender Rust (25/10, madrugada — Antigravity / Gemini):
- **Logo oficial**: ícone `.ico` embutido no `.exe` via `winresource` (visível no Explorer e atalhos), `brand::logo_badge` e `brand::app_icon` integrados na janela principal, instalador, barra lateral, Modo TV e Sobre.
- **Espaço Aprender & Programar em Rust**: opção `📖  Aprender & Programar` na barra lateral (abaixo de Tradutor C/C++), módulo `learn.rs` com 10 lições práticas do zero, editor de código interativo, execução local com o compilador `rustc` em segundo plano, diagnóstico didático de erros comuns de iniciantes, exportação de projeto Cargo e conquista `aluno_rustaceo` (50 XP).
- Executáveis `RRS - Riva Rust Store.exe` e `Instalador - RRS Riva Rust Store.exe` gerados em release (versão 0.3.0 com ícone embutido e 99 testes aprovados). Cópias anteriores salvas em `Obsoleto\gitstore-exe-antes-logo-aprender-24.10.26`.

Como ligar o login (o Fabricio faz uma vez):
1. GitHub › Settings › Developer settings › OAuth Apps › New OAuth App; nome "RRS - Riva Rust Store", Homepage = perfil dele, callback `http://127.0.0.1` (não é usado); marcar **Enable Device Flow**.
2. Copiar o **Client ID** (não é segredo) para Configurações › Conta do GitHub › Opção 1. Fica no banco (`settings.github_oauth_client_id`). Proposta (pendente): depois de testado, virar constante no código para os amigos não precisarem registrar nada.
3. Alternativa sem OAuth App: token clássico com `read:user` e `gist` na Opção 2.
4. Comunidade geral: criar um repositório público, ligar Discussions (Settings › Features) e informar `dono/repo` em Configurações › Central de bate-papo.

[Não verificado] na prática:
- controle físico no Modo TV (a navegação por teclado foi conferida; nenhum controle estava conectado);
- `gh attestation verify` (o GitHub CLI não está instalado neste PC);
- tradução com IA chamando a API de verdade (precisa da chave do Fabricio; o laço foi testado com respostas simuladas e `rustc` real);
- Windows Sandbox (o PC é Windows Home);
- instalação pelo winget (a busca foi testada com o winget real);
- jogos da Epic e do GOG em instalações reais (só a Steam tinha jogos);
- catálogo online: o mecanismo funciona (testado com chave descartável), mas a chave oficial ainda não existe e não há onde hospedar o catálogo;
- login pelo Device Flow de ponta a ponta (falta o Client ID do OAuth App; um Client ID inválido devolve `{"error":"Not Found"}`, conferido);
- chamadas reais de perfil (Gist), amigos, biblioteca por link e Discussions com conta logada (testadas só com respostas de exemplo); gravar e ler o token no Gerenciador de Credenciais.

Próximos passos — Proposta (pendente), em ordem:
1. O Fabricio registra o OAuth App e testa login, publicar/despublicar o perfil e a comunidade (passos acima).
2. O Fabricio gera a chave oficial (`C:\cargo-target\rrs\release\gitstore.exe catalog-keygen --out <arquivo fora do vault>`); a chave pública vai para `catalog_sign::TRUSTED_KEYS`; escolher onde hospedar `catalog.json` e `catalog.json.sig`.
3. Mover `target\` para fora do vault (`target-dir`).
4. Testes para `pick_best_asset`, `platform_of` e `normalize_repo_name` (funções puras ainda sem teste).
5. Traduzir para o inglês as notas da auditoria (geradas em português em `verify.rs`) e as mensagens da barra de status.
6. Rodar o Claude Security (scan) e o `cargo audit` (não instalado).

## 7. Gerar os .exe da raiz

1. Confirme que a RRS está fechada: `Get-Process | Where-Object { $_.Path -like 'F:\NAS\Cods\Rust\RRS\*' }` deve vir vazio.
2. Copie os dois `.exe` atuais para `F:\NAS\Obsoleto\gitstore-exe-antes-<assunto>-DD.MM.AA\` e registre no CSV.
3. `cargo build --release` e copie `C:\cargo-target\rrs\release\gitstore.exe` para `RRS - Riva Rust Store.exe` **e** `Instalador - RRS Riva Rust Store.exe` (o mesmo binário: o nome decide se abre a loja ou o instalador).
4. Teste os dois pela linha de comando (`--db` e `--catalog` temporários, comando `search`) e abra a janela uma vez.
5. A cópia instalada em `%LOCALAPPDATA%\Programs\RRS-RivaRustStore` só o Fabricio atualiza (pelo instalador).

## 8. Registrar ao terminar

1. Commit local da rodada (seção 2, sem push).
1. Entrada no topo de `F:\NAS\SaaS\Base-de-Conhecimento\05-Sessoes\DIARIO-DE-SESSOES.md` (Feito / Decidido / Pendente; diga qual IA trabalhou).
2. Nota do dia em linguagem simples: `F:\NAS\SaaS\Days Runs\Evolucao DD.MM.AA.md` (criar ou atualizar).
3. Atualizar a seção 6 deste arquivo.
4. Rodar `python F:\NAS\.claude\scripts\verificar-vault.py`.
