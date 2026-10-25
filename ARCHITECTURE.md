# RRS - Riva Rust Store — Manual de Arquitetura & Guia de Desenvolvimento

> **Aviso para Desenvolvedores e Agentes de IA:**
> Este documento é o guia definitivo de arquitetura, funcionamento interno, decisões de design e regras operacionais do **RRS - Riva Rust Store**. Qualquer desenvolvedor humano ou agente autônomo de IA que for continuar o desenvolvimento deste projeto **deve ler e seguir estas diretrizes**.

---

## 1. Visão Geral & Ideia do Projeto

### O que é o RRS?
O **RRS (Riva Rust Store)** é uma central desktop moderna e ultrarrápida, desenvolvida 100% em **Rust**, com duas finalidades centrais:
1. **Loja & Launcher de Aplicativos e Jogos Gratuitos do GitHub**:
   - Uma vitrine visual moderna (inspirada na **Steam**, **Microsoft Store** e **Google Play**).
   - Catálogo curado de projetos de código aberto de alta qualidade, com foco especial em:
     - **Jogos Completos & Engines de Jogos**;
     - **Engenharia Reversa (REA)**: Ports nativos de PC, recompilações estáticas (*Ship of Harkinian*, *Zelda 64 Recomp*, *DevilutionX*), decompilações e motores recriados (*OpenRCT2*, *OpenRA*, *OpenMW*, *VCMI*, *Augustus*, *CorsixTH*);
     - **Ferramentas de Engenharia Reversa**: *Ghidra*, *ImHex*, *x64dbg*, *Cutter*, *radare2*, *dnSpy*;
     - **Emuladores Consagrados**: *PCSX2*, *mGBA*, *melonDS*, *PPSSPP*, *Cemu*, *Azahar*;
     - **IA Local Offline**: *Ollama*, *GPT4All*, *llamafile*, *KoboldCpp*, *Jan*, *Upscayl*;
     - **Aplicativos e Utilitários Consagrados**: *yt-dlp*, *qBittorrent*, *FreeCAD*, *OBS Studio*, *Notepad++*, *Telegram Desktop*, *LocalSend*, *Syncthing*, *UniGetUI*, *BleachBit*.
2. **Motor de Segurança & Auditoria de Precedentes**:
   - Auditoria automática antes de baixar (idade do projeto, atividade recente, estrelas, issues, licença).
   - Download com validação criptográfica do **SHA-256** oficial do GitHub Releases (arquivos corrompidos ou adulterados são descartados na hora).
   - Varredura de segurança local offline com **Windows Defender** nativo em 1 clique.
   - Verificação de assinatura digital **Authenticode** do Windows.
3. **Launcher de Jogos (Estilo Steam)**:
   - **Auto-extração inteligente de arquivos `.zip`** e localização automática do executável principal (`.exe`).
   - Botão **`▶ JOGAR AGORA`** / **`▶ ABRIR APLICATIVO`** que roda o jogo no diretório correto.
   - **Playtime Tracker**: Registro automático de tempo jogado (ex: *4.2 horas*) e última vez jogado no banco SQLite local.
4. **Estúdio Tradutor C / C++ ➔ Rust**:
   - Motor integrado de conversão de código C/C++ (`.c`, `.cpp`, `.h`, `.hpp`) para Rust (`.rs`).
   - Suporte a dois modos: **Rust Idiomático & Seguro** ou **Bindings FFI / ABI C** (`#[repr(C)]`).
   - Validação sintática em tempo real invocando o compilador oficial `rustc`.
   - **Gerador de Projetos Cargo**: Cria em 1 clique uma pasta completa com `Cargo.toml` e `src/main.rs`, pronta para `cargo run`.

---

## 2. Decisão Fundamental de Arquitetura: Catálogo Curado por IA

Uma decisão fundamental tomada pelo usuário é:
> **Não usar chaves de API pagas em tempo de execução dentro do aplicativo.**
> Em vez disso, a curadoria do catálogo é feita **offline/sob demanda pela IA**, gerando o arquivo `catalog.json`.

* **Arquivo de Catálogo**: [`catalog.json`](file:///f:/NAS/Cods/Rust/RRS/catalog.json) contém a lista de todos os repositórios com seus metadados traduzidos e curados em Português (`kind`, `category`, `tags`, `summary`, `assets`, `platforms`, `stars`, `trust_score`).
* **Catálogo Embutido no Binário**: O arquivo `catalog.json` é embutido no executável através de `include_str!("../catalog.json")` no módulo [`src/db.rs`](file:///f:/NAS/Cods/Rust/RRS/src/db.rs). Isso significa que:
  - O executável `.exe` gerado é **100% autossuficiente (standalone)**. Se o usuário copiar apenas o executável para outro PC, o catálogo de 75+ itens já funciona imediatamente!
  - Ao iniciar, o app sincroniza o catálogo embutido com o banco SQLite local (`gitstore.db`).
* **Periodicidade**: Uma vez por semana (ou quando solicitado), o usuário pede à IA para pesquisar novos repositórios ou atualizar releases. A IA executa a busca e atualiza o `catalog.json` e os executáveis.

---

## 3. Estrutura dos Módulos Rust (`src/`)

```
RRS/
├── .cargo/
│   └── config.toml          # target-dir = C:/cargo-target/rrs (build fora do vault)
├── Cargo.toml               # Dependências do projeto
├── catalog.json             # Catálogo mestre curado em formato JSON (ordenado por stars DESC)
├── gitstore.db              # Banco de dados relacional SQLite local
├── downloads/               # Pasta de downloads e jogos extraídos (downloads/<owner>_<repo>/app/)
├── projetos_traduzidos/     # Projetos Cargo gerados automaticamente pelo tradutor C++ -> Rust
└── src/
    ├── main.rs              # Ponto de entrada, CLI (clap), loop GUI (eframe) e controle de console
    ├── gui.rs               # Interface gráfica completa em egui (Steam Storefront, filtros, launcher)
    ├── db.rs                # Camada SQLite (rusqlite), catálogo embutido, apps e playtime
    ├── downloader.rs        # Download seguro, streaming SHA-256, auto-unzip, Defender e launcher
    ├── github.rs            # Cliente GitHub REST e parser de assets HTML sem limite de requisição
    ├── verify.rs            # Motor de reputação (0-100), precedentes e suporte a VirusTotal
    ├── installer.rs         # Assistente gráfico de instalação no Windows (atalhos Desktop e Menu Iniciar)
    └── translator.rs        # Tradutor C/C++ -> Rust, validador rustc e gerador de projeto Cargo
```

### Detalhamento por Módulo:

#### A. [`src/main.rs`](file:///f:/NAS/Cods/Rust/RRS/src/main.rs)
- **Subcomandos CLI**:
  - `gui` (padrão): Inicia a interface gráfica com `eframe::run_native`.
  - `install [--silent]`: Aciona o instalador gráfico ou executa instalação silenciosa.
  - `search`: Busca rápida no catálogo local por terminal.
  - `show <repo>`: Exibe ficha detalhada, segurança e arquivos disponíveis.
  - `verify <repo> [--vt]`: Audita um repositório remoto antes de baixar.
  - `download <repo>`: Baixa o melhor instalador para o PC e valida o hash.
  - `translate <arquivo> [--ffi] [--check]`: Traduz C/C++ pelo terminal.
  - `check-updates`: Verifica se há releases mais novas para o que está instalado.
  - `sync`, `import`, `export`: Utilitários de manutenção do catálogo.
- **Detecção de Clique Duplo no Windows**:
  - Usa a API Win32 `GetConsoleProcessList` e `FreeConsole()`. Se o usuário der clique duplo no arquivo `.exe` pelo Windows Explorer, a janela preta de comando do prompt é ocultada imediatamente, mantendo a experiência idêntica a um app nativo de Windows.
  - Se rodar de dentro do PowerShell/CMD com argumentos (ex: `rrs search`), a saída do terminal é preservada normalmente.
- **Detecção do Instalador**:
  - Se o executável for nomeado `Instalador - RRS Riva Rust Store.exe` (ou contiver `setup`/`installer`), ele abre direto no **Assistente de Instalação**.

#### B. [`src/gui.rs`](file:///f:/NAS/Cods/Rust/RRS/src/gui.rs)
- Construído com `eframe = "0.31"` / `egui`.
- **Navegação Lateral**:
  - `Vitrine & Destaques`: Carrossel Hero Banner com destaques curados + seções temáticas em grade.
  - `Engenharia Reversa`: Filtros rápidos para Jogos Recompilados e Ferramentas (Ghidra, ImHex, etc.).
  - `Jogos & Ports`: Jogos completos gratuitos.
  - `Emuladores & Retrô`: Emuladores de consoles de código aberto.
  - `IA Local & Apps`: Softwares de inteligência artificial offline e utilitários.
  - `Ferramentas & RE`: Game engines e descompiladores.
  - `Minha Biblioteca`: Painel estilo Steam exibindo jogos instalados, horas jogadas, data da última sessão, botão de jogar, pasta local e desinstalação.
  - `Tradutor C/C++ ➔ Rust`: Editor split-screen interativo lado a lado.
- **Painel Direito de Detalhes & Auditoria**:
  - Barra de notas de segurança (0 a 100) com alertas em cores (verde, amarelo, vermelho).
  - Seletor de arquivo de release para download.
  - Botão **`▶ JOGAR AGORA`** / **`▶ ABRIR APLICATIVO`**.
  - Botão **`🛡️ Windows Defender`**: Aciona varredura nativa do antivírus em segundo plano.
  - Botão **`🔏 Checar Assinatura`**: Checa assinatura digital Authenticode.
  - Botão **`🗑 Desinstalar`**: Remove a pasta do app e atualiza a biblioteca.

#### C. [`src/db.rs`](file:///f:/NAS/Cods/Rust/RRS/src/db.rs)
- SQLite embutido via `rusqlite` com feature `bundled`. Não requer DLLs externas.
- **Tabelas**:
  - `apps`: Armazena metadados de todos os repositórios do catálogo.
  - `installed`: Registra os aplicativos instalados, tag instalada, caminho local (`file_path`), hash `sha256`, se foi verificado contra o digest do GitHub (`verified`), data de instalação, `last_played` e `playtime_seconds`.
  - `settings`: Configurações de usuário (ex: tokens opcionais).
- **Catálogo Embutido (`EMBEDDED_CATALOG`)**:
  - Carrega `include_str!("../catalog.json")`.
  - Se o banco de dados for novo ou a pasta não tiver `catalog.json`, ele gera o arquivo e popula o banco automaticamente.
- **Funções de Playtime**:
  - `record_playtime(conn, full_name, elapsed_seconds)`: Incrementa os segundos jogados e registra o carimbo de data/hora atual.

#### D. [`src/downloader.rs`](file:///f:/NAS/Cods/Rust/RRS/src/downloader.rs)
- **Streaming Seguro**: Baixa o arquivo em pedaços usando `reqwest`, alimentando simultaneamente um hasher `sha2::Sha256`.
- **Integridade Obrigatória**: Se o GitHub fornecer um digest `sha256:` oficial na release e o hash baixado for diferente, o arquivo `.part` é **imediatamente excluído** e um erro de segurança é emitido.
- **Auto-Extração de `.zip`**:
  - Função `extract_archive_if_needed`: Se o arquivo baixado for um `.zip`, invoca PowerShell `Expand-Archive` para a pasta `app/`.
  - Função `find_best_executable`: Varre a pasta extraída, ignora instaladores/crashpads secundários e localiza o executável principal (`.exe`).
  - O caminho retornado para o banco passa a ser o executável pronto para rodar.
- **Launcher Inteligente (`launch_installed_app`)**:
  - Executa o aplicativo com seu `current_dir` setado para a pasta onde o `.exe` reside. Isso é crucial para que jogos carreguem suas pastas de assets, mapas e sons.
  - Retorna o `Child` process handle para que a GUI possa medir o tempo de execução e registrar no Playtime Tracker.
- **Varredura Windows Defender (`scan_with_windows_defender`)**:
  - Invoca `C:\Program Files\Windows Defender\MpCmdRun.exe -Scan -ScanType 3 -File "<path>" -DisableRemediation`.
  - Retorna status limpo (código 0) ou infectado (código 2).
- **Assinatura Digital (`check_authenticode_signature`)**:
  - Invoca `Get-AuthenticodeSignature` no Windows e retorna status e nome da certificadora.

#### E. [`src/github.rs`](file:///f:/NAS/Cods/Rust/RRS/src/github.rs)
- **Drible do Rate Limit**:
  - A API REST sem autenticação tem limite estrito de 60 requisições/hora.
  - Para obter os arquivos de uma release sem consumir cota da API, o módulo faz uma requisição `HEAD` para `https://github.com/<owner>/<repo>/releases/latest` (obtendo a tag pelo redirect 302) e depois obtém a lista de assets via `https://github.com/<owner>/<repo>/releases/expanded_assets/<tag>`. Essa rota é pública, não requer token e inclui os digests `sha256:` calculados pelo GitHub.
- **Detecção de Plataforma (`platform_of`)**:
  - Identifica se o arquivo é `windows`, `linux`, `macos`, `android` ou `java`.
  - Descarta arquivos de depuração (`.pdb`), assinaturas (`.sig`), checksums (`.sha256`) e códigos-fonte (`-src`, `source.tar.gz`).
- **Seletor Inteligente (`pick_best_asset`)**:
  - No Windows, dá prioridade máxima para executáveis `x86_64` / `x64` (`.msi`, `.exe`, `.zip`), penalizando versões de 32 bits (`x86`) ou ARM64 se houver binário x64 nativo.

#### F. [`src/installer.rs`](file:///f:/NAS/Cods/Rust/RRS/src/installer.rs)
- Assistente visual de instalação para Windows com `eframe`.
- Permite que qualquer pessoa instale o RRS no caminho `%LOCALAPPDATA%\Programs\RRS-RivaRustStore`.
- Cria atalhos oficiais do Windows (`.lnk`) na **Área de Trabalho** e no **Menu Iniciar** via WScript.Shell.
- Adiciona o diretório da aplicação ao `PATH` do usuário, permitindo rodar `rrs` no PowerShell de qualquer pasta.

#### G. [`src/translator.rs`](file:///f:/NAS/Cods/Rust/RRS/src/translator.rs)
- **Mapeamento de Tipos e Estruturas**:
  - Converte primitivos (`int`, `uint32_t`, `size_t`, `double`, `bool`) para tipos Rust (`i32`, `u32`, `usize`, `f64`, `bool`).
  - Mapeia ponteiros `const char*` ➔ `&str` ou `*const c_char`.
  - Converte `std::vector<T>` ➔ `Vec<T>`, `std::string` ➔ `String`.
  - Converte estruturas e classes C++ para `pub struct` em Rust com `#[derive(Debug, Clone)]`.
  - Converte loops `for (int i = 0; i < n; i++)` para `for i in 0..n`.
  - Converte `printf` e `std::cout << ...` para macros `println!`.
- **Validação com `rustc`**:
  - Função `verify_rust_syntax`: Escreve o código traduzido em um arquivo temporário e invoca `rustc --edition=2021 --crate-type=lib --emit=metadata`. Se houver erros, retorna os detalhes exatos da linha onde o compilador falhou.
- **Gerador de Projetos Cargo (`generate_cargo_project`)**:
  - Gera uma pasta completa com `Cargo.toml`, `.gitignore` e `src/main.rs`.

---

## 4. Cuidados Importantes & Peculiaridades do Ambiente

Ao modificar este projeto, fique muito atento aos seguintes pontos específicos:

1. **Compilação em Unidade de Rede (NAS `F:\`)**:
   - Em drives mapeados de rede ou SMB, o cache incremental do compilador Rust falha ao criar *hard links*.
   - **Solução implementada (25/10/2026)**: o [`.cargo/config.toml`](file:///f:/NAS/Cods/Rust/RRS/.cargo/config.toml) manda o build para `C:/cargo-target/rrs` (`target-dir`). Lá, em NTFS, o cache incremental funciona. Antes era `incremental = false` com o build dentro do vault.
2. **Toolchain MSVC Obrigatória no Windows**:
   - O projeto deve ser compilado com `stable-x86_64-pc-windows-msvc`. Se tentar usar `pc-windows-gnu` sem o GCC/MinGW instalado, o linker falhará com `dlltool.exe not found`.
3. **UTF-8 BOM no PowerShell 5.1**:
   - O Windows PowerShell 5.1 adiciona um byte-order mark (`\u{feff}`) quando grava arquivos UTF-8 com `Set-Content`.
   - Ao ler arquivos JSON ou código com `serde_json`, sempre use `.trim_start_matches('\u{feff}')` para evitar erros de caractere invisível na linha 1 coluna 1.
4. **Executável Aberto em Segundo Plano**:
   - Se tentar copiar o executável `C:\cargo-target\rrs\debug\gitstore.exe` para `RRS - Riva Rust Store.exe` enquanto a janela do aplicativo estiver aberta, o Windows bloqueará a operação com erro de compartilhamento (`IOException`). Sempre encerre o processo antes de substituir os arquivos na raiz.
5. **Acesso ao SQLite por Múltiplas Threads**:
   - `rusqlite::Connection` não é `Send` de forma síncrona simples entre threads do Tokio.
   - O padrão arquitetural adotado no RRS é: crie e abra a conexão SQLite dentro da thread ou use transações curtas (`db::open(&self.db_path)`), ou envie eventos via canal seguro (`bg_events: Arc<Mutex<Vec<BgEvent>>>`).

---

## 5. Roteiro de Melhorias Futuras (Roadmap)

Se você (ou outro agente de IA) for continuar evoluindo o projeto, aqui está a lista das próximas melhorias recomendadas:

### Fase 1: Cache Visual de Ícones dos Projetos
- Adicionar a dependência `egui_extras = { version = "0.31", features = ["all_loaders"] }`.
- Baixar e armazenar em cache os avatares dos repositórios (`https://github.com/<owner>.png?size=64`) na pasta `downloads/icons/`.
- Substituir os monogramas textuais dos cards pelos ícones reais dos jogos e ferramentas.

### Fase 2: Fila de Downloads Múltiplos
- Atualmente, a barra de progresso suporta 1 download por vez (`SharedDownloadState`).
- Evoluir para uma lista de tarefas `Vec<DownloadTask>` com semáforo de concorrência (`tokio::sync::Semaphore::new(2)`), permitindo enfileirar vários jogos simultaneamente.

### Fase 3: Botão "Atualizar Todos" na Biblioteca
- Na aba *Minha Biblioteca*, quando houver 2 ou mais apps com `app.has_update() == true`, exibir um botão no topo: **`⬆ Atualizar Todos os Aplicativos`** para atualizar a coleção em lote.

### Fase 4: Gerenciador de Mods e Savegames
- Para ports de jogos de engenharia reversa (*DevilutionX*, *Ship of Harkinian*, *GZDoom*, *fheroes2*), criar uma aba de "Arquivos de Dados / Mods" onde o usuário pode apontar a pasta dos arquivos originais do jogo (ex: `DIABDAT.MPQ`, `doom2.wad`, ROM de N64) ou instalar mods com 1 clique.

---

## 6. Comandos Essenciais para o Desenvolvedor / IA

```powershell
# Entrar na pasta do projeto
Set-Location f:\NAS\Cods\Rust\RRS

# Verificar integridade e compilar sem rodar
cargo check

# Compilar versão de desenvolvimento
cargo build

# Compilar executável final otimizado (Release)
cargo build --release

# Sincronizar catálogo SQLite -> catalog.json
C:\cargo-target\rrs\debug\gitstore.exe export

# Executar a loja gráfica diretamente
C:\cargo-target\rrs\debug\gitstore.exe gui

# Testar tradução de um arquivo C++ com checagem do rustc
C:\cargo-target\rrs\debug\gitstore.exe translate .\exemplo.cpp --check
```
