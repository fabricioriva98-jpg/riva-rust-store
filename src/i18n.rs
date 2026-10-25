//! Idioma da interface (português ou inglês). Os textos ficam em português no código e passam por
//! `tr`; a tabela abaixo dá a versão em inglês. Um teste confere que todo `tr("...")` do código tem
//! tradução e que a tabela não tem entradas sobrando.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Pt,
    En,
}

pub const SETTING_LANG: &str = "idioma";

impl Lang {
    pub fn from_setting(value: Option<&str>) -> Lang {
        match value.map(|v| v.trim().to_lowercase()) {
            Some(v) if v == "en" => Lang::En,
            _ => Lang::Pt,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Lang::Pt => "pt",
            Lang::En => "en",
        }
    }
}

pub fn tr(lang: Lang, pt: &str) -> String {
    if lang == Lang::En {
        if let Some((_, en)) = TABLE.iter().find(|(k, _)| *k == pt) {
            return (*en).to_string();
        }
    }
    pt.to_string()
}

/// (português, inglês)
pub static TABLE: &[(&str, &str)] = &[
    // Barra lateral
    ("🛡 CATÁLOGO VERIFICADO", "🛡 VERIFIED CATALOG"),
    ("EXPLORAR LOJA", "EXPLORE"),
    ("Vitrine & Destaques", "Storefront & Featured"),
    ("Feito em Rust", "Made in Rust"),
    ("Engenharia Reversa", "Reverse Engineering"),
    ("Jogos & Ports", "Games & Ports"),
    ("Emuladores & Retrô", "Emulators & Retro"),
    ("IA Local & Apps", "Local AI & Apps"),
    ("Ferramentas & RE", "Tools & RE"),
    ("SEU PC", "YOUR PC"),
    ("Biblioteca", "Library"),
    ("Minha Biblioteca", "My Library"),
    ("ESTÚDIO RUST", "RUST STUDIO"),
    ("🦀  Tradutor C/C++ ➔ Rust", "🦀  C/C++ ➔ Rust Translator"),
    ("🛡 Só Verificados (70+)", "🛡 Trusted only (70+)"),
    ("🔄 Checar Atualizações", "🔄 Check for Updates"),
    ("📌 Instalar RRS no PC", "📌 Install RRS on this PC"),
    (
        "Instala a RRS no Windows e cria atalhos na Área de Trabalho, Menu Iniciar e terminal (comando 'rrs').",
        "Installs RRS on Windows and creates Desktop, Start Menu and terminal ('rrs' command) shortcuts.",
    ),
    // Barra superior
    ("Buscar jogos, ports de engenharia reversa, emuladores, IA, apps...", "Search games, reverse-engineering ports, emulators, AI, apps..."),
    ("Plataforma:", "Platform:"),
    ("Ordenar:", "Sort:"),
    ("Mais Populares", "Most Popular"),
    ("Maior Confiança", "Highest Trust"),
    ("Nome (A-Z)", "Name (A-Z)"),
    ("Mais Recentes", "Most Recent"),
    ("⊞ Grade", "⊞ Grid"),
    ("☰ Lista", "☰ List"),
    ("Limpar busca", "Clear search"),
    ("Adicionar dono/repo do GitHub", "Add owner/repo from GitHub"),
    ("+ Verificar Repo", "+ Verify Repo"),
    // Configurações
    ("🔑 GITHUB_TOKEN:", "🔑 GITHUB_TOKEN:"),
    ("Opcional (5000 req/h no GitHub)", "Optional (5000 req/h on GitHub)"),
    ("Opcional (consulta VirusTotal)", "Optional (VirusTotal lookups)"),
    ("💾 Salvar Chaves", "💾 Save Keys"),
    ("https://.../catalog.json (com catalog.json.sig ao lado)", "https://.../catalog.json (with catalog.json.sig next to it)"),
    ("🔄 Salvar e atualizar", "🔄 Save and update"),
    ("Arquivo rrs-biblioteca.json", "rrs-biblioteca.json file"),
    ("📤 Exportar biblioteca", "📤 Export library"),
    (
        "Salva a lista de apps instalados e o tempo de jogo, para reinstalar em outro PC",
        "Saves the list of installed apps and playtime, to reinstall on another PC",
    ),
    ("📥 Importar biblioteca", "📥 Import library"),
    (
        "Lê um rrs-biblioteca.json de outro PC: os apps entram em Biblioteca › Para reinstalar",
        "Reads an rrs-biblioteca.json from another PC: the apps go to Library › To reinstall",
    ),
    ("📦 Catálogo Local:", "📦 Local catalog:"),
    ("itens carregados", "items loaded"),
    ("📤 Exportar Catálogo JSON", "📤 Export catalog JSON"),
    ("📥 Reimportar catalog.json", "📥 Reimport catalog.json"),
    ("📁 Abrir Pasta do Catálogo", "📁 Open catalog folder"),
    // Rodapé
    ("na fila", "queued"),
    ("projetos verificados no catálogo", "verified projects in the catalog"),
    // Vitrine
    ("★ DESTAQUE", "★ FEATURED"),
    ("🦀 Rust", "🦀 Rust"),
    ("estrelas", "stars"),
    ("Versão", "Version"),
    ("Plataformas:", "Platforms:"),
    ("📦 Ver na Loja & Baixar com SHA-256", "📦 View & Download with SHA-256"),
    ("◀ Anterior", "◀ Previous"),
    ("Próximo ▶", "Next ▶"),
    ("Ver todos →", "See all →"),
    ("🦀 Feito em Rust", "🦀 Made in Rust"),
    ("Apps e jogos escritos em Rust, a linguagem que dá nome à loja.", "Apps and games written in Rust, the language behind the store's name."),
    ("🆕 Lançados recentemente", "🆕 Recently released"),
    ("Versões publicadas há menos tempo no GitHub.", "Most recently published versions on GitHub."),
    ("🆕 Atualizados recentemente", "🆕 Recently updated"),
    (
        "Projetos com código atualizado há menos tempo (data do último commit no GitHub).",
        "Projects with the most recent code changes (date of the last commit on GitHub).",
    ),
    ("🏛 Clássicos dos anos 90, recriados", "🏛 '90s classics, recreated"),
    ("Motores refeitos pela comunidade para jogar os originais em PCs atuais.", "Community-rebuilt engines to play the originals on modern PCs."),
    ("🧠 IA no seu PC, sem nuvem", "🧠 AI on your PC, no cloud"),
    ("Modelos e ferramentas de IA que rodam localmente.", "AI models and tools that run locally."),
    ("🎬 Kit do criador de conteúdo", "🎬 Content creator kit"),
    ("Gravar, editar, converter e publicar vídeo e áudio.", "Record, edit, convert and publish video and audio."),
    ("🔐 Privacidade e controle dos seus dados", "🔐 Privacy and control of your data"),
    (
        "Senhas, sincronização e acesso remoto sem depender de serviços fechados.",
        "Passwords, sync and remote access without depending on closed services.",
    ),
    ("🧬 Jogos de Engenharia Reversa — Ports Nativos, Decompilações & Recompilações", "🧬 Reverse-Engineered Games — Native Ports, Decompilations & Recompilations"),
    (
        "Motores nativos de PC reconstruídos pela comunidade (60+ FPS, widescreen, mods) — 100% verificados.",
        "Native PC engines rebuilt by the community (60+ FPS, widescreen, mods).",
    ),
    ("🔬 Aplicativos de Engenharia Reversa & Análise Binária", "🔬 Reverse Engineering & Binary Analysis Apps"),
    ("Descompiladores, debuggers e analisadores hexadecimais oficiais de código aberto.", "Official open-source decompilers, debuggers and hex analyzers."),
    ("🕹 Emuladores & Plataformas Retrô Verificadas", "🕹 Emulators & Retro Platforms"),
    ("Emuladores oficiais de alta performance com código aberto e hashes verificados.", "Official high-performance open-source emulators."),
    ("🤖 IA Local & Aplicativos Essenciais para PC", "🤖 Local AI & Essential PC Apps"),
    (
        "Modelos de IA offline, upscalers de imagem, gravação de vídeo e utilitários consagrados.",
        "Offline AI models, image upscalers, video recording and well-known utilities.",
    ),
    ("🎮 Jogos Completos 100% Gratuitos (Prontos para Jogar)", "🎮 Complete Free Games (Ready to Play)"),
    ("Jogos completos de código aberto que não exigem arquivos externos.", "Complete open-source games that need no external files."),
    // Seções
    ("🔎 Resultados da Busca", "🔎 Search Results"),
    ("Projetos verificados encontrados no catálogo.", "Verified projects found in the catalog."),
    (
        "Apps e jogos cuja linguagem principal no GitHub é Rust (segurança de memória sem coletor de lixo).",
        "Apps and games whose main language on GitHub is Rust (memory safety without a garbage collector).",
    ),
    ("🧬 Central de Engenharia Reversa (Jogos & Aplicativos Verificados)", "🧬 Reverse Engineering Hub (Games & Apps)"),
    (
        "Ports nativos de PC, recompilações estáticas, decompilações e ferramentas de engenharia reversa.",
        "Native PC ports, static recompilations, decompilations and reverse-engineering tools.",
    ),
    ("🎮 Catálogo de Jogos & Ports Nativos", "🎮 Games & Native Ports"),
    ("Jogos gratuitos completos e motores reconstruídos por engenharia reversa.", "Complete free games and engines rebuilt through reverse engineering."),
    ("🕹 Emuladores & Plataformas Retrô", "🕹 Emulators & Retro Platforms"),
    ("Emuladores verificados diretamente dos repositórios oficiais dos desenvolvedores.", "Emulators straight from the developers' official repositories."),
    ("🤖 IA Local & Aplicativos Desktop", "🤖 Local AI & Desktop Apps"),
    ("Ferramentas de Inteligência Artificial offline, mídia e produtividade.", "Offline AI, media and productivity tools."),
    ("🛠 Motores de Jogos, Ferramentas & Engenharia Reversa", "🛠 Game Engines, Tools & Reverse Engineering"),
    ("Engines, editores de nível, descompiladores e analisadores binários.", "Engines, level editors, decompilers and binary analyzers."),
    ("📦 Minha Biblioteca (Instalados neste PC)", "📦 My Library (Installed on this PC)"),
    (
        "Gerencie seus jogos e aplicativos baixados, confira integridade SHA-256 e atualizações.",
        "Manage your downloaded games and apps, check SHA-256 integrity and updates.",
    ),
    ("🦀 Tradutor C/C++ ➔ Rust", "🦀 C/C++ ➔ Rust Translator"),
    ("Mostrar:", "Show:"),
    ("Todos de Eng. Reversa", "All reverse engineering"),
    ("🧬 Jogos (Ports, Decompilações & Recompilações)", "🧬 Games (Ports, Decompilations & Recompilations)"),
    ("🔬 Aplicativos & Ferramentas (Ghidra, ImHex, x64dbg)", "🔬 Apps & Tools (Ghidra, ImHex, x64dbg)"),
    ("⬆ Atualizar Todos os Aplicativos", "⬆ Update All Apps"),
    ("Filtro:", "Filter:"),
    ("Todos", "All"),
    ("🎮 Jogos", "🎮 Games"),
    ("🧬 Ports RE", "🧬 RE Ports"),
    ("🔬 Ferramentas RE", "🔬 RE Tools"),
    ("🤖 IA Local", "🤖 Local AI"),
    ("🕹 Emuladores", "🕹 Emulators"),
    ("📱 Apps", "📱 Apps"),
    ("🛠 Engines", "🛠 Engines"),
    ("Nenhum item encontrado nesta seção com os filtros atuais.", "No items in this section with the current filters."),
    ("🔁 Para reinstalar", "🔁 To reinstall"),
    (
        "Vindos de uma biblioteca importada. Abra cada um e baixe; o tempo de jogo volta junto.",
        "From an imported library. Open each one and download it; the playtime comes back too.",
    ),
    ("Fora do catálogo local (adicione pelo campo \"Adicionar dono/repo\"):", "Not in the local catalog (add them with the \"Add owner/repo\" field):"),
    // Cards
    ("PORT / RECOMP", "PORT / RECOMP"),
    ("ENG. REVERSA", "REVERSE ENG."),
    ("EMULADOR", "EMULATOR"),
    ("IA LOCAL", "LOCAL AI"),
    ("JOGO GRÁTIS", "FREE GAME"),
    ("APLICATIVO", "APP"),
    ("FERRAMENTA", "TOOL"),
    ("TERMINAL", "TERMINAL"),
    ("ITEM", "ITEM"),
    ("⬆ Atualizar", "⬆ Update"),
    ("✔ Instalado", "✔ Installed"),
    ("Obter", "Get"),
    ("🏗 Atestado", "🏗 Attested"),
    ("⬆ ATUALIZAÇÃO", "⬆ UPDATE"),
    ("Verificado", "Trusted"),
    ("Atenção", "Caution"),
    ("Alto Risco", "High Risk"),
    // Painel de detalhes
    ("Licença:", "License:"),
    ("não informada", "not stated"),
    ("GitHub Oficial ↗", "Official GitHub ↗"),
    ("Linguagem:", "Language:"),
    ("🏗 Atestado de build: presente", "🏗 Build attestation: present"),
    ("🏗 Sem atestado de build", "🏗 No build attestation"),
    ("🏗 Atestado de build: não consultado (use \"Revalidar no GitHub\")", "🏗 Build attestation: not checked (use \"Recheck on GitHub\")"),
    ("🔁 Na lista para reinstalar (biblioteca importada)", "🔁 On the reinstall list (imported library)"),
    ("Ao baixar, o tempo de jogo do outro PC volta junto.", "When you download it, the playtime from the other PC comes back."),
    ("Tirar da lista", "Remove from list"),
    ("🧬 Port / Recompilação por Engenharia Reversa", "🧬 Reverse-Engineered Port / Recompilation"),
    (
        "Este projeto recria o motor do jogo para PC a partir de engenharia reversa e não inclui os arquivos do jogo. Para jogos originalmente comerciais, o programa pede a sua cópia/ROM original na primeira abertura.",
        "This project rebuilds the game engine for PC through reverse engineering and does not include the game files. For originally commercial games, it asks for your own copy/ROM on first launch.",
    ),
    ("📦 Obter Aplicativo — Versão", "📦 Get App — Version"),
    ("Nenhum instalador encontrado nesta Release.", "No installer found in this release."),
    ("Instalador selecionado:", "Selected installer:"),
    ("★ Recomendado p/ seu PC: ", "★ Recommended for your PC: "),
    ("✔ SHA-256 Oficial do GitHub:", "✔ Official GitHub SHA-256:"),
    ("• O hash SHA-256 será calculado e registrado durante o download.", "• The SHA-256 hash will be computed and recorded during the download."),
    ("⏳ Na fila de downloads...", "⏳ In the download queue..."),
    ("⬆ Atualizar para", "⬆ Update to"),
    ("Verificar SHA-256", "Verify SHA-256"),
    ("⬇ Baixar & Verificar SHA-256 Automaticamente", "⬇ Download & Verify SHA-256 Automatically"),
    ("Instalado:", "Installed:"),
    ("Nova versão disponível:", "New version available:"),
    ("✔ Pronto no PC — Versão", "✔ Ready on this PC — Version"),
    ("confirmado c/ GitHub", "confirmed with GitHub"),
    ("calculado", "computed"),
    ("Baixado em:", "Downloaded on:"),
    ("⏱ Tempo de Jogo:", "⏱ Playtime:"),
    ("📅 Última vez:", "📅 Last played:"),
    ("▶ JOGAR AGORA", "▶ PLAY NOW"),
    ("▶ ABRIR APLICATIVO", "▶ OPEN APP"),
    ("📂 Abrir Pasta", "📂 Open Folder"),
    ("🛡️ Windows Defender", "🛡️ Windows Defender"),
    ("Executa uma varredura nativa do Windows Defender no arquivo baixado sem custos", "Runs a free native Windows Defender scan on the downloaded file"),
    ("⚠ Confirmar remoção", "⚠ Confirm removal"),
    ("🗑 Desinstalar", "🗑 Uninstall"),
    ("Apaga a pasta do app em downloads\\ e tira da biblioteca.", "Deletes the app's folder in downloads\\ and removes it from the library."),
    ("removido do computador.", "removed from this computer."),
    ("Instale uma versão anterior se a mais nova der problema no seu PC.", "Install an older version if the newest one has problems on your PC."),
    ("Carregar versões do GitHub", "Load versions from GitHub"),
    ("Nenhuma release publicada.", "No published releases."),
    ("pré-lançamento", "pre-release"),
    ("mais recente", "latest"),
    ("instalada", "installed"),
    ("SHA-256 conferido com o GitHub", "SHA-256 checked against GitHub"),
    ("release sem SHA-256 do GitHub (só calculado)", "release without a GitHub SHA-256 (computed only)"),
    ("⬇ Baixar", "⬇ Download"),
    ("sem instalador", "no installer"),
    ("🛡 Auditoria de Precedentes", "🛡 Track Record Audit"),
    ("Índice de Confiança:", "Trust Score:"),
    ("🔄 Revalidar no GitHub", "🔄 Recheck on GitHub"),
    ("🔍 Checar no VirusTotal", "🔍 Check on VirusTotal"),
    ("Consulta os hashes oficiais da Release no VirusTotal sem baixar o arquivo", "Looks up the release's official hashes on VirusTotal without downloading the file"),
    // Tradutor
    ("🦀 Tradutor de C / C++ para Rust — RRS Studio", "🦀 C / C++ to Rust Translator — RRS Studio"),
    ("OFFLINE & INSTANTÂNEO", "OFFLINE & INSTANT"),
    ("VALIDADOR RUSTC INTEGRADO", "BUILT-IN RUSTC CHECK"),
    (
        "Converta estruturas, classes, headers de engenharia reversa, ponteiros, loops e containers STL (std::vector, std::string) de C/C++ para Rust.",
        "Convert structs, classes, reverse-engineering headers, pointers, loops and STL containers (std::vector, std::string) from C/C++ to Rust.",
    ),
    ("Modo de Conversão:", "Conversion mode:"),
    ("🦀 Rust Idiomático & Seguro (Vec, String, &mut)", "🦀 Idiomatic & Safe Rust (Vec, String, &mut)"),
    ("🔗 FFI / ABI C (#[repr(C)], *mut T, extern \"C\")", "🔗 FFI / C ABI (#[repr(C)], *mut T, extern \"C\")"),
    ("Exemplos:", "Examples:"),
    ("Exemplo", "Example"),
    ("⚡ Traduzir", "⚡ Translate"),
    ("✔ Validar Sintaxe no rustc", "✔ Check with rustc"),
    ("Compila o código Rust gerado usando o rustc local para garantir que não há erros", "Compiles the generated Rust code with the local rustc to make sure there are no errors"),
    ("📋 Copiar Rust", "📋 Copy Rust"),
    ("Arquivo (.c/.cpp/.h/.rs):", "File (.c/.cpp/.h/.rs):"),
    ("Caminho de arquivo para carregar (.cpp/.h) ou salvar (.rs)", "File path to load (.cpp/.h) or save (.rs)"),
    ("📂 Carregar C/C++", "📂 Load C/C++"),
    ("💾 Salvar .rs", "💾 Save .rs"),
    ("📦 Gerar Projeto Cargo", "📦 Create Cargo Project"),
    ("Cria uma pasta completa com Cargo.toml e src/main.rs pronta para 'cargo run'", "Creates a full folder with Cargo.toml and src/main.rs ready for 'cargo run'"),
    ("ENTRADA C / C++", "C / C++ INPUT"),
    ("Cole seu código .c, .cpp, .h ou .hpp abaixo (tradução em tempo real)", "Paste your .c, .cpp, .h or .hpp code below (live translation)"),
    ("SAÍDA RUST (.RS)", "RUST OUTPUT (.RS)"),
    ("Você também pode editar o código Rust gerado antes de validar/salvar", "You can also edit the generated Rust code before checking/saving"),
    ("📋 Notas de Conversão & Verificação do Compilador", "📋 Conversion Notes & Compiler Check"),
    ("Diagnóstico do rustc:", "rustc diagnostics:"),
    // Fase 2: Windows, saves, winget
    ("apps do catálogo encontrados no Windows", "catalog apps found on Windows"),
    ("🖥 Instalados no Windows fora da RRS", "🖥 Installed on Windows outside RRS"),
    ("🔄 Procurar de novo", "🔄 Scan again"),
    ("🔎 Procurar apps instalados no Windows", "🔎 Scan apps installed on Windows"),
    (
        "Lê a lista de programas do Windows e mostra os que também estão no catálogo, com aviso de versão nova.",
        "Reads the Windows program list and shows the ones that are also in the catalog, flagging new versions.",
    ),
    ("programas no Windows", "programs on Windows"),
    ("reconhecidos no catálogo", "recognized in the catalog"),
    ("no PC:", "on PC:"),
    ("catálogo:", "catalog:"),
    ("⬆ versão nova", "⬆ new version"),
    ("✔ em dia", "✔ up to date"),
    ("Ver na loja", "View in store"),
    ("💾 Saves e configurações", "💾 Saves & settings"),
    (
        "Aponte a pasta onde o jogo guarda os saves. A RRS faz uma cópia compactada quando o jogo fecha.",
        "Point to the folder where the game keeps its saves. RRS makes a zipped copy when the game closes.",
    ),
    ("Salvar pasta", "Save folder"),
    ("✔ Pasta de saves guardada.", "✔ Save folder stored."),
    ("Fazer backup agora", "Back up now"),
    ("backup(s); os mais recentes:", "backup(s); the most recent:"),
    ("⚠ Confirmar", "⚠ Confirm"),
    ("Restaurar", "Restore"),
    (
        "Substitui os saves atuais pelos do backup. Antes, guarda os atuais num backup \"antes-de-restaurar\".",
        "Replaces the current saves with the backup. First it stores the current ones in a \"before-restore\" backup.",
    ),
    ("📦 Também no winget", "📦 Also on winget"),
    ("Buscar no winget", "Search winget"),
    ("Procura o termo no catálogo do winget, o gerenciador de pacotes do Windows.", "Searches the term in the catalog of winget, the Windows package manager."),
    (
        "O winget baixa e confere o SHA-256 do manifesto da Microsoft. Esses pacotes não passam pela auditoria de precedentes da RRS.",
        "winget downloads and checks the SHA-256 from Microsoft's manifest. These packages do not go through the RRS track-record audit.",
    ),
    ("Nada encontrado no winget.", "Nothing found on winget."),
    ("⚠ Confirmar instalação", "⚠ Confirm install"),
    ("Instalar via winget", "Install with winget"),
    // Fase 3: outras lojas, modo TV, terminal, atestado, sandbox, IA
    ("Terminal (Rust)", "Terminal (Rust)"),
    ("⌨ Terminal (Rust)", "⌨ Terminal (Rust)"),
    (
        "Ferramentas de linha de comando já compiladas, baixadas da release oficial e conferidas pelo SHA-256.",
        "Prebuilt command-line tools, downloaded from the official release and checked by SHA-256.",
    ),
    ("Adicionar ferramenta pelo nome do crate (crates.io)", "Add a tool by its crate name (crates.io)"),
    ("Buscar no crates.io", "Look up on crates.io"),
    (
        "Descobre o repositório do crate e verifica a release como qualquer app da loja; baixa o binário pronto, sem compilar.",
        "Finds the crate's repository and checks the release like any store app; downloads the prebuilt binary, no compiling.",
    ),
    ("⌨ Adicionar ao terminal", "⌨ Add to terminal"),
    ("Copia o executável para a pasta bin da RRS e coloca essa pasta no PATH do usuário.", "Copies the executable into the RRS bin folder and adds that folder to the user PATH."),
    ("pronto; abra um terminal novo para usar.", "ready; open a new terminal to use it."),
    ("🎮 Modo TV (tela cheia)", "🎮 TV Mode (full screen)"),
    ("Biblioteca em tela cheia para jogar com controle ou teclado (inclui Steam, Epic e GOG).", "Full-screen library to play with a gamepad or keyboard (includes Steam, Epic and GOG)."),
    ("🎮 Modo TV", "🎮 TV Mode"),
    ("controle(s) conectado(s)", "gamepad(s) connected"),
    ("✖ Sair do modo TV", "✖ Exit TV mode"),
    ("Setas ou direcional: mover · Enter ou A: abrir · Esc ou B: sair", "Arrows or D-pad: move · Enter or A: open · Esc or B: exit"),
    ("Nada instalado ainda. Baixe um jogo na loja ou instale pela Steam, Epic ou GOG.", "Nothing installed yet. Download a game from the store or install one through Steam, Epic or GOG."),
    ("🎮 Outras lojas no PC (Steam, Epic, GOG)", "🎮 Other stores on this PC (Steam, Epic, GOG)"),
    ("🔎 Procurar jogos instalados", "🔎 Scan installed games"),
    ("Lista os jogos das outras lojas para abrir tudo pela RRS (também no Modo TV).", "Lists the other stores' games so you can open everything from RRS (also in TV Mode)."),
    ("Nenhum jogo da Steam, Epic ou GOG encontrado.", "No Steam, Epic or GOG games found."),
    ("▶ Jogar", "▶ Play"),
    ("jogos encontrados em outras lojas", "games found in other stores"),
    ("🔏 Conferir atestado (gh)", "🔏 Verify attestation (gh)"),
    (
        "Usa o GitHub CLI para conferir a assinatura do atestado de build deste arquivo e o repositório de origem.",
        "Uses the GitHub CLI to verify this file's build attestation signature and its source repository.",
    ),
    (
        "✔ Atestado conferido pelo GitHub CLI: assinatura válida e arquivo gerado por este repositório.",
        "✔ Attestation verified by the GitHub CLI: valid signature and file built by this repository.",
    ),
    ("O GitHub CLI recusou o atestado:", "The GitHub CLI rejected the attestation:"),
    ("Não conferido:", "Not verified:"),
    ("🧪 Abrir no Windows Sandbox", "🧪 Open in Windows Sandbox"),
    (
        "Abre o instalador numa máquina descartável do Windows; ao fechar o Sandbox, tudo o que rodou lá some.",
        "Opens the installer in a disposable Windows machine; when you close the Sandbox, everything that ran there is gone.",
    ),
    ("🧪 Windows Sandbox aberto com o arquivo (pasta só para leitura).", "🧪 Windows Sandbox opened with the file (read-only folder)."),
    ("Opcional (tradutor com IA)", "Optional (AI translator)"),
    ("⏳ Traduzindo com IA...", "⏳ Translating with AI..."),
    ("🤖 Traduzir com IA (Claude)", "🤖 Translate with AI (Claude)"),
    (
        "Envia o código para a API do Claude com a sua chave (cada tradução é cobrada), compila com o rustc e devolve os erros ao modelo até compilar (até 4 tentativas).",
        "Sends the code to the Claude API with your key (each translation is billed), compiles it with rustc and sends the errors back to the model until it compiles (up to 4 attempts).",
    ),
    ("Para usar, salve a chave da API do Claude em Configurações.", "To use it, save your Claude API key in Settings."),
    ("Traduzido com IA:", "Translated with AI:"),
    ("rodada(s)", "round(s)"),
    ("✔ Compila no rustc (verificado pela RRS).", "✔ Compiles with rustc (checked by RRS)."),
    ("✘ Não compilou depois de todas as tentativas; veja o diagnóstico abaixo.", "✘ Did not compile after all attempts; see the diagnostics below."),
    ("Ocorrências de unsafe:", "unsafe occurrences:"),
    ("✔ Código Rust validado pelo rustc.", "✔ Rust code checked by rustc."),
    ("Tradução com IA concluída.", "AI translation finished."),
    ("Tradução com IA falhou:", "AI translation failed:"),
    // Configurações, conta do GitHub, perfil, conquistas e comunidade
    ("A nota de confiança é um indicador de precedentes, não uma garantia de segurança.", "The trust score is an indicator of track record, not a guarantee of safety."),
    ("Abrir Issues ↗", "Open Issues ↗"),
    ("Abrir no GitHub ↗", "Open on GitHub ↗"),
    (
        "Ainda não há repositório da comunidade RRS. Crie um repositório público no GitHub, ligue as Discussions e informe em Configurações › Central de bate-papo. As conversas de cada app já funcionam pela aba do app selecionado.",
        "There is no RRS community repository yet. Create a public repository on GitHub, turn on Discussions and enter it in Settings › Chat hub. Each app's conversations already work from the selected app's tab.",
    ),
    ("Apaga o Gist do perfil na sua conta; os amigos deixam de ver o seu nível e conquistas.", "Deletes the profile Gist from your account; friends will no longer see your level and achievements."),
    ("Apps instalados", "Installed apps"),
    ("Cancelar", "Cancel"),
    ("Carregar amigos", "Load friends"),
    ("Chaves de API salvas no banco local.", "API keys saved in the local database."),
    ("Client ID do OAuth App:", "OAuth App Client ID:"),
    ("Como registrar o OAuth App (uma vez, uns 5 minutos)", "How to register the OAuth App (once, about 5 minutes)"),
    ("1. No GitHub: Settings › Developer settings › OAuth Apps › New OAuth App.", "1. On GitHub: Settings › Developer settings › OAuth Apps › New OAuth App."),
    (
        "2. Nome: RRS - Riva Rust Store. Homepage URL: o seu perfil (https://github.com/seu-usuario). Authorization callback URL: http://127.0.0.1 (não é usado).",
        "2. Name: RRS - Riva Rust Store. Homepage URL: your profile (https://github.com/your-user). Authorization callback URL: http://127.0.0.1 (not used).",
    ),
    ("3. Marque \"Enable Device Flow\" e clique em Register application.", "3. Tick \"Enable Device Flow\" and click Register application."),
    (
        "4. Copie o Client ID (não é segredo) e cole no campo acima. Não é preciso gerar Client secret.",
        "4. Copy the Client ID (it is not a secret) and paste it in the field above. No Client secret is needed.",
    ),
    ("Compartilhar com amigos por link", "Share with friends by link"),
    ("Conectado ao GitHub como", "Connected to GitHub as"),
    ("Conquista:", "Achievement:"),
    ("Conquistas", "Achievements"),
    ("conquistas", "achievements"),
    (
        "Conversas no GitHub Discussions. Ler é aqui; responder e abrir conversa nova é no site do GitHub.",
        "Conversations on GitHub Discussions. You read them here; replying and starting a new one happen on the GitHub site.",
    ),
    ("Copia um texto com o nome, a descrição e o link do app para mandar a um amigo.", "Copies a text with the app's name, description and link to send to a friend."),
    ("Cria um Gist secreto na sua conta (só quem tiver o link vê) e copia o link.", "Creates a secret Gist in your account (only people with the link can see it) and copies the link."),
    ("Criar link exige entrar com o GitHub; importar de um link funciona sem conta.", "Creating a link requires signing in with GitHub; importing from a link works without an account."),
    ("Crie em github.com/settings/tokens (token clássico) marcando read:user e gist.", "Create it at github.com/settings/tokens (classic token) with read:user and gist checked."),
    (
        "Desligado, nada sai deste PC. Ligado, a RRS publica num Gist público da sua conta: nível, título, conquistas e os 10 apps mais usados com as horas. Não publica caminhos, hashes nem chaves.",
        "When off, nothing leaves this PC. When on, RRS publishes a public Gist in your account: level, title, achievements and the 10 most used apps with their hours. It never publishes paths, hashes or keys.",
    ),
    ("Despublicar", "Unpublish"),
    ("Digite este código no GitHub:", "Enter this code on GitHub:"),
    ("Downloads", "Downloads"),
    ("Entrar com o GitHub", "Sign in with GitHub"),
    (
        "Entrar com o GitHub libera o perfil, os amigos, o compartilhamento por link e a central de bate-papo. A RRS pede só leitura do perfil e permissão de criar gists.",
        "Signing in with GitHub unlocks the profile, friends, sharing by link and the chat hub. RRS only asks to read your profile and to create gists.",
    ),
    ("Entrar com token", "Sign in with token"),
    ("Entre com o GitHub (acima) para publicar.", "Sign in with GitHub (above) to publish."),
    (
        "Entre com o GitHub para ler as conversas aqui (a API de conversas do GitHub exige login).",
        "Sign in with GitHub to read the conversations here (GitHub's discussions API requires sign-in).",
    ),
    ("Entre com o GitHub para ter nome, foto, amigos e perfil público.", "Sign in with GitHub to get a name, photo, friends and a public profile."),
    ("Esperando você autorizar no navegador...", "Waiting for you to authorize in the browser..."),
    ("Este repositório não usa o GitHub Discussions. As conversas dele ficam nas Issues.", "This repository does not use GitHub Discussions. Its conversations are in the Issues."),
    (
        "Ficam no banco local da RRS (texto puro). Com a conta do GitHub conectada, o token do GitHub abaixo não é necessário.",
        "They stay in the local RRS database (plain text). With the GitHub account connected, the GitHub token below is not needed.",
    ),
    ("GitHub ↗", "GitHub ↗"),
    ("Horas jogadas", "Hours played"),
    ("Jogador local", "Local player"),
    ("Link copiado:", "Link copied:"),
    ("Link recebido de um amigo (gist.github.com/...)", "Link received from a friend (gist.github.com/...)"),
    ("Nenhuma conversa ainda. Que tal abrir a primeira?", "No conversations yet. How about starting the first one?"),
    ("Nível", "Level"),
    ("O token fica no Gerenciador de Credenciais do Windows, não no banco da RRS.", "The token is kept in Windows Credential Manager, not in the RRS database."),
    ("Opção 1 — código no navegador (recomendado)", "Option 1 — code in the browser (recommended)"),
    ("Opção 2 — token pessoal", "Option 2 — personal token"),
    ("Pasta de dados:", "Data folder:"),
    ("Quem você segue no GitHub. Aparece o nível de quem também publicou o perfil da RRS.", "People you follow on GitHub. The level shows for those who also published their RRS profile."),
    ("Repositório da comunidade:", "Community repository:"),
    ("Sair da conta", "Sign out"),
    ("Salvar", "Save"),
    (
        "Só importa se a assinatura for de uma chave confiável, a versão não for mais antiga que a atual e a validade não tiver vencido. Confere uma vez por dia ao abrir.",
        "It only imports when the signature comes from a trusted key, the version is not older than the current one and it has not expired. Checks once a day on startup.",
    ),
    ("Token com os escopos read:user e gist", "Token with the read:user and gist scopes"),
    ("Traduções", "Translations"),
    (
        "Usa o GitHub Discussions: as conversas de cada app ficam no repositório dele; a comunidade geral da RRS precisa de um repositório público com Discussions ligadas.",
        "Uses GitHub Discussions: each app's conversations live in its own repository; the general RRS community needs a public repository with Discussions turned on.",
    ),
    ("Verificações", "Checks"),
    ("Você ainda não segue ninguém no GitHub.", "You don't follow anyone on GitHub yet."),
    ("Você saiu da conta do GitHub (o token foi removido do Gerenciador de Credenciais).", "You signed out of GitHub (the token was removed from Credential Manager)."),
    ("em comum", "in common"),
    ("não publicou perfil da RRS", "has not published an RRS profile"),
    ("para o nível", "to level"),
    ("respostas", "replies"),
    ("ℹ Sobre", "ℹ About"),
    ("← Voltar para a lista", "← Back to the list"),
    ("↩ Responder no GitHub ↗", "↩ Reply on GitHub ↗"),
    ("⏳ Carregando conversas...", "⏳ Loading conversations..."),
    ("⏳ Entrando...", "⏳ Signing in..."),
    ("⚙  Configurações", "⚙  Settings"),
    ("⚙ Abrir Configurações", "⚙ Open Settings"),
    ("⚙ Configurações", "⚙ Settings"),
    ("⚠ Confirmar despublicação", "⚠ Confirm unpublishing"),
    ("✏ Nova conversa no GitHub ↗", "✏ New conversation on GitHub ↗"),
    ("✔ Perfil despublicado (o Gist foi apagado da sua conta).", "✔ Profile unpublished (the Gist was deleted from your account)."),
    ("✔ Perfil publicado", "✔ Profile published"),
    ("✔ Perfil publicado no seu GitHub (Gist público).", "✔ Profile published on your GitHub (public Gist)."),
    ("✔ Repositório da comunidade salvo.", "✔ Community repository saved."),
    ("🌍 Idioma", "🌍 Language"),
    ("🌐 Abrir github.com/login/device", "🌐 Open github.com/login/device"),
    ("🌐 Catálogo online assinado", "🌐 Signed online catalog"),
    ("🌐 Perfil público (opcional)", "🌐 Public profile (optional)"),
    ("🌐 Publicar meu perfil", "🌐 Publish my profile"),
    ("🏆  Perfil & Conquistas", "🏆  Profile & Achievements"),
    ("🏠 Comunidade RRS", "🏠 RRS Community"),
    ("👤 Conta do GitHub", "👤 GitHub account"),
    ("👤 Ver meu perfil", "👤 View my profile"),
    ("👥 Amigos", "👥 Friends"),
    ("💬  Comunidade", "💬  Community"),
    ("💬 Central de bate-papo", "💬 Chat hub"),
    ("💬 Conversas sobre este app", "💬 Conversations about this app"),
    ("💬 Sobre", "💬 About"),
    ("📋 Copiar código", "📋 Copy code"),
    ("📚 Biblioteca", "📚 Library"),
    ("📥 Importar do link", "📥 Import from link"),
    ("🔄 Atualizar", "🔄 Refresh"),
    ("🔄 Atualizar agora", "🔄 Update now"),
    ("🔑 Chaves de API (opcionais)", "🔑 API keys (optional)"),
    ("🔗 Compartilhar", "🔗 Share"),
    ("🔗 Criar link da minha biblioteca", "🔗 Create a link to my library"),
    ("🔗 Texto do app copiado; cole na conversa com o seu amigo.", "🔗 App text copied; paste it in the chat with your friend."),
    // Títulos de nível e conquistas (o texto vem de achievements.rs)
    ("Novato", "Rookie"),
    ("Explorador", "Explorer"),
    ("Colecionador", "Collector"),
    ("Veterano", "Veteran"),
    ("Lenda da RRS", "RRS Legend"),
    ("Primeiro download", "First download"),
    ("Baixe e instale o primeiro app pela RRS.", "Download and install your first app with RRS."),
    ("Tenha 10 apps instalados.", "Have 10 apps installed."),
    ("Rustáceo", "Rustacean"),
    ("Instale 3 apps feitos em Rust.", "Install 3 apps made in Rust."),
    ("Desconfiado", "Skeptic"),
    ("Revalide 10 repositórios no GitHub.", "Recheck 10 repositories on GitHub."),
    ("Segunda opinião", "Second opinion"),
    ("Consulte um arquivo no VirusTotal.", "Look up a file on VirusTotal."),
    ("Vigilante", "Watchful"),
    ("Faça 5 varreduras com o Windows Defender.", "Run 5 Windows Defender scans."),
    ("Atestado em dia", "Attested"),
    ("Instale um app com atestado de build do GitHub.", "Install an app with a GitHub build attestation."),
    ("Arqueólogo", "Archaeologist"),
    ("Instale uma versão anterior de um app.", "Install an older version of an app."),
    ("Maratonista", "Marathoner"),
    ("Jogue 10 horas somando todos os jogos.", "Play 10 hours across all games."),
    ("Retrô", "Retro"),
    ("Instale um emulador.", "Install an emulator."),
    ("Engenheiro reverso", "Reverse engineer"),
    ("Instale um port feito por engenharia reversa.", "Install a reverse-engineered port."),
    ("Tradutor", "Translator"),
    ("Traduza 5 códigos de C/C++ para Rust.", "Translate 5 C/C++ snippets to Rust."),
    ("Parceria com IA", "AI partnership"),
    ("Faça uma tradução com IA que compile no rustc.", "Make an AI translation that compiles with rustc."),
    ("Terminal raiz", "Terminal native"),
    ("Instale uma ferramenta de terminal em Rust.", "Install a Rust terminal tool."),
    ("Precavido", "Prepared"),
    ("Faça um backup de saves.", "Back up your saves."),
    ("Mudança de casa", "Moving house"),
    ("Exporte ou compartilhe a sua biblioteca.", "Export or share your library."),
    ("Modo sofá", "Couch mode"),
    ("Abra o Modo TV.", "Open TV Mode."),
    ("Social", "Social"),
    ("Publique o seu perfil para os amigos.", "Publish your profile for your friends."),
    ("Boa conversa", "Good talk"),
    ("Abra a central de bate-papo da comunidade.", "Open the community chat hub."),
    ("Indicação", "Referral"),
    ("Compartilhe um app com alguém.", "Share an app with someone."),
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Textos literais passados para `tr(self.lang, "...")`, `tr(lang, "...")` ou `self.tr("...")`,
    /// inclusive quando o texto começa na linha seguinte.
    fn tr_calls(src: &str) -> Vec<String> {
        let mut out = Vec::new();
        for prefix in ["tr(self.lang,", "tr(lang,", "self.tr("] {
            out.extend(literals_after(src, prefix));
        }
        out
    }

    fn literals_after(src: &str, prefix: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut rest = src;
        while let Some(i) = rest.find(prefix) {
            let after_prefix = rest[i + prefix.len()..].trim_start();
            let Some(after) = after_prefix.strip_prefix('"') else {
                rest = &rest[i + prefix.len()..];
                continue;
            };
            let mut text = String::new();
            let mut escaped = false;
            let mut end = after.len();
            for (idx, c) in after.char_indices() {
                if escaped {
                    text.push(match c {
                        'n' => '\n',
                        other => other,
                    });
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    end = idx + 1;
                    break;
                } else {
                    text.push(c);
                }
            }
            out.push(text);
            rest = &after[end..];
        }
        out
    }

    fn sources() -> Vec<&'static str> {
        vec![
            include_str!("gui.rs"),
            include_str!("gui/extras.rs"),
            include_str!("gui/phase3.rs"),
            include_str!("gui/tv.rs"),
            include_str!("gui/settings.rs"),
            include_str!("gui/social.rs"),
            include_str!("achievements.rs"),
        ]
    }

    #[test]
    fn todo_texto_traduzivel_tem_ingles() {
        let mut missing: Vec<String> = sources()
            .iter()
            .flat_map(|s| tr_calls(s))
            .filter(|t| !TABLE.iter().any(|(k, _)| k == t))
            .collect();
        missing.sort();
        missing.dedup();
        assert!(missing.is_empty(), "sem tradução: {missing:#?}");
    }

    #[test]
    fn tabela_sem_repeticao_nem_sobra() {
        // Também vale texto que chega ao `tr` por variável (títulos de seção, coleções, selos)
        let all: String = sources().concat();
        let mut unused = Vec::new();
        for (i, (k, en)) in TABLE.iter().enumerate() {
            assert!(!en.trim().is_empty(), "tradução vazia para {k:?}");
            assert!(!TABLE[..i].iter().any(|(k2, _)| k2 == k), "repetida: {k:?}");
            let literal = format!("\"{}\"", k.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n"));
            if !all.contains(&literal) {
                unused.push(*k);
            }
        }
        assert!(unused.is_empty(), "entradas sem uso no código: {unused:#?}");
    }

    #[test]
    fn troca_de_idioma() {
        assert_eq!(Lang::from_setting(Some("EN")), Lang::En);
        assert_eq!(Lang::from_setting(Some("pt")), Lang::Pt);
        assert_eq!(Lang::from_setting(None), Lang::Pt);
        assert_eq!(tr(Lang::Pt, "qualquer texto"), "qualquer texto");
        assert_eq!(tr(Lang::En, "texto sem tradução"), "texto sem tradução");
    }
}
