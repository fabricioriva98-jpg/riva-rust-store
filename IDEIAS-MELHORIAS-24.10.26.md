---
tipo: pesquisa
projeto: RRS - Riva Rust Store
data: 2026-10-24
autor: Claude (Opus 5.5)
status: Proposta (pendente)
---
# Ideias de melhoria para a RRS — pesquisa de 24/10/2026

Pedido do Fabricio: "busque na internet ideias ou melhorias para o projeto [...] Quero melhorar ela porem no momento estou sem mais ideia".
Base: pesquisa na web (fontes no fim) cruzada com o código atual da RRS e com a [[AVALIACAO-24.10.26|avaliação de hoje]].

## Fase 1 — implementada em 24/10/2026 (noite)

Pedido: "Pode começar a fase 1 das ideias (selo Feito em Rust, exportar biblioteca, modo portátil, versão anterior, coleções, interface e selo de atestado)". Cópia do código anterior em `Obsoleto/gitstore-src-antes-fase1-24.10.26/`.

| # | O que ficou pronto | Conferido |
|---|---|---|
| 1 | Campo `language` no catálogo (82 de 82 preenchidos pela busca do GitHub), seção e selo "🦀 Feito em Rust". Entraram 7 apps Rust com instalador para Windows: Alacritty, WezTerm, Lapce, Neovide, Espanso, Gyroflow e A/B Street. Antes eram 3 apps Rust, agora são 10. | tela e teste |
| 2 | Exportar/importar biblioteca (`rrs-biblioteca.json`): nas configurações da interface e por `rrs library-export` / `rrs library-import`. Apps importados vão para Biblioteca › "Para reinstalar"; ao baixar, o tempo de jogo volta. | tela, linha de comando e testes |
| 3 | Modo portátil: o caminho do app instalado é gravado relativo à pasta da RRS. Bancos antigos são convertidos ao abrir, inclusive quando a letra da unidade mudou. | testes |
| 4 | Versões anteriores: botão no painel de detalhes que lista as releases do GitHub, cada uma com "⬇ Baixar", e `rrs download dono/repo --tag vX`. | tela (Ship of Harkinian) e download real do Alacritty v0.16.1 com SHA-256 conferido |
| 5 | Vitrine com "Feito em Rust", "Lançados recentemente" e 4 coleções curadas. Enquanto houver poucas datas de release, a linha usa a data do último commit e mostra o título "Atualizados recentemente". | tela e testes |
| 6 | Interface: fontes de símbolos do Windows como reserva (o "☐" sumiu), cards da grade empilhados direito, destaque e barra superior que quebram linha, editores do tradutor com altura cheia. Também: "Licença: não informada" no lugar de "Aberta" e o texto "sem vírus" retirado dos ports. | tela; um teste falha se algum caractere da interface ficar sem glifo |
| 7 | Selo de atestado de build do GitHub (presença, sem conferir a assinatura): consultado no `verify` e em "Revalidar no GitHub". | `verify cli/cli` = presente; 6 apps Rust = ausente |

Testes: de 11 para 26, todos passando. `cargo clippy` mostra os mesmos 8 avisos de antes. Os `.exe` da raiz ainda são da versão anterior à fase 1.

## Fases 2 e 3 — implementadas em 24/10/2026 (noite)

Pedido: "faça a fase 2 a 4 e depois gere o arquivo .exe". Não existe fase 4 neste documento: foram feitas as fases 2 e 3 inteiras. Cópia do código anterior em `Obsoleto/gitstore-src-antes-fase2-3-24.10.26/`. Versão 0.3.0.

| # | O que ficou pronto | Conferido |
|---|---|---|
| 8 | Catálogo online assinado (ed25519): `catalog-keygen`, `catalog-sign`, `catalog-update`, `catalog-refresh` e campo "Catálogo online" nas configurações (confere 1 vez por dia ao abrir). Recusa chave desconhecida, catálogo alterado, versão mais antiga e assinatura vencida. | 8 testes; ponta a ponta com chave descartável |
| 9 | Fila de downloads com 2 ao mesmo tempo; cada download registra o app no banco ao terminar (corrige o M3 da avaliação). Também: `.part` apagado se o download cair e nome de arquivo sem caminho (M4). | 3 testes com servidor HTTP local |
| 10 | Codeberg/Forgejo/Gitea e GitLab (`codeberg.org/dono/repo`, `gitlab.com/grupo/projeto`). | 3 testes; `verify` real no Forgejo e no Inkscape |
| 11 | Programas do Windows comparados com o catálogo (Biblioteca › "Instalados no Windows fora da RRS"). | 3 testes; na tela achou o ripgrep 15.1.0 instalado, com aviso da 15.2.0 |
| 12 | Backup de saves: pasta por jogo, backup manual e automático quando o jogo fecha, restauração com cópia "antes-de-restaurar". | 3 testes com compactação real |
| 13 | Busca no winget ("Também no winget" nos resultados da busca) e instalação pelo winget com confirmação. | 3 testes; busca real conferida |
| 14 | Inglês: seletor de idioma; mais de 260 textos traduzidos; no inglês os resumos vêm da descrição do GitHub. As notas da auditoria e parte das mensagens da barra de status continuam em português. | teste cobra tradução de todo texto marcado; conferido na tela |
| 15 | Acessibilidade: cards e botões só de ícone com nome para o leitor de tela (AccessKit, já ligado no eframe). | 1 teste lendo a árvore do AccessKit |
| 16 | "🔏 Conferir atestado (gh)": chama `gh attestation verify` (assinatura Sigstore e repositório). | 2 testes; [Não verificado] na prática: o GitHub CLI não está instalado neste PC |
| 17 | Tradutor: aviso por linha não reconhecida e rótulo experimental; "🤖 Traduzir com IA (Claude)" (`claude-opus-5-5`, laço do `rustc` até 4 tentativas, chave do usuário) e `translate --ai`. | 8 testes (laço com respostas simuladas e `rustc` real); [Não verificado] chamada real à API (precisa da chave) |
| 18 | Jogos da Steam, Epic e GOG na Biblioteca e no Modo TV; comando `games`. | 5 testes; na tela listou os 3 itens reais da Steam (o GRIS Soundtrack é trilha sonora, não jogo) |
| 19 | Modo TV: tela cheia, blocos grandes, teclado e controle (gilrs). | 2 testes; teclado conferido na tela; [Não verificado] com controle físico |
| 20 | Seção "⌨ Terminal (Rust)" com 6 ferramentas (ripgrep, bat, fd, hyperfine, zoxide, Starship), campo "nome do crate" (crates.io → release) e "Adicionar ao terminal" (pasta `bin\` no PATH); comando `crate`. | 3 testes; `crate ripgrep` real baixou e o `rg.exe` rodou |
| 21 | "🧪 Abrir no Windows Sandbox" (pasta só leitura), escondido quando o Sandbox não existe. | 2 testes; [Não verificado] na prática: o PC é Windows Home |

Extras: "Desinstalar" de cada app pede dois cliques (a pasta pode ter saves); validação pelo `rustc` com arquivo temporário único. Catálogo: 88 itens, 16 em Rust. Testes: de 26 para 75, todos passando; `cargo clippy` com os mesmos 8 avisos antigos. Instruções para outra IA em `CONTINUIDADE-IA.md`.

## O que a pesquisa mostrou

- **A concorrente mais parecida é a Komi Store** (antes chamada "GitHub Store"). Ela é aberta, feita em Kotlin/Compose, roda em Android, Windows, macOS e Linux, e instala releases do GitHub, Codeberg, Forgejo e Gitea. O próprio aviso do projeto diz que ele não revisa, não valida e não garante os instaladores.
- **Por isso, o diferencial natural da RRS é verificar.** Ela já tem nota de confiança, SHA-256, Defender, Authenticode e VirusTotal. O caminho é aprofundar a verificação e torná-la honesta (ver M1 e M2 da avaliação).
- **A segunda identidade possível é o Rust.** Nenhuma loja pesquisada tem uma seção "feito em Rust" nem instala ferramentas Rust pré-compiladas.
- **No tradutor C/C++**, o estado da arte combina um tradutor por regras (c2rust) com IA e correção pelos erros do compilador. Tradução por padrões linha a linha, como a da RRS hoje, não tem como competir com isso.

## Ideias priorizadas

Esforço: **P** = até 1 sessão; **M** = 2 a 4 sessões; **G** = mais que isso. Ordem dentro de cada fase = valor ÷ esforço.

### Fase 1 — rápidas e de alto valor

| # | Ideia | Inspiração | Esforço | Onde mexe na RRS |
|---|---|---|---|---|
| 1 | **Selo "🦀 Feito em Rust"** e seção própria (filtro pela linguagem do repositório) | identidade da loja | P | `github.rs` (campo `language` do repositório), `db.rs`, `gui.rs` |
| 2 | **Exportar/importar a biblioteca** (lista de apps instalados) para reinstalar em outro PC | UniGetUI, Obtainium | P | `db.rs` + botão nas configurações |
| 3 | **Modo portátil de verdade**: guardar o caminho dos apps relativo à pasta da RRS. Hoje ele é absoluto e quebra se a letra do pendrive ou do disco mudar | PortableApps.com | P | `db::mark_installed`, `downloader.rs` |
| 4 | **Instalar uma versão anterior** (lista de releases do repositório) | UniGetUI | P | `github.rs` (endpoint de releases), painel de detalhes |
| 5 | **Coleções e "Lançados recentemente"** na vitrine | Komi Store | P | `gui.rs` (ordenação por data já existe) |
| 6 | **Interface sem glifos quebrados e sem sobreposição** | avaliação de hoje | P | `gui.rs` (fontes e layout) |
| 7 | **Selo "Tem atestado de build do GitHub"**: indica que o arquivo foi gerado pelo GitHub Actions a partir de um commit | ObtainX (fork do Obtainium), `gh release verify-asset` | P | `github.rs`, `verify.rs` |

Sobre a #7: no começo é só presença do atestado, não verificação criptográfica. A tela precisa dizer isso. A verificação completa é a #16.

### Fase 2 — médias, mudam o tamanho da loja

| # | Ideia | Inspiração | Esforço | Onde mexe na RRS |
|---|---|---|---|---|
| 8 | **Catálogo atualizado e assinado**: um repositório do catálogo atualiza versões e hashes sozinho (como o `checkver`/`autoupdate` do Scoop) e publica o `catalog.json` assinado; a RRS baixa e confere a assinatura, a versão e a validade | Scoop, TUF | M | `db::import_catalog`, novo módulo de assinatura |
| 9 | **Fila de downloads** com 2 ao mesmo tempo, já corrigindo o M3 da avaliação | Heroic (fila de downloads), roadmap do ARCHITECTURE | M | `downloader.rs`, `gui.rs` |
| 10 | **Mais forjas além do GitHub**: Codeberg, Forgejo, Gitea, GitLab | Komi Store, Obtainium | M | `github.rs` vira um módulo de forjas |
| 11 | **Detectar apps já instalados no Windows** (lista de programas instalados do sistema) e avisar quando houver versão nova no catálogo | UniGetUI | M | novo módulo Windows |
| 12 | **Backup de saves**: o usuário aponta a pasta de saves de cada jogo e a RRS faz cópia compactada antes de atualizar ou quando o jogo fecha | Heroic, Playnite | M | `downloader.rs` (já mede quando o jogo fecha) |
| 13 | **Usar o catálogo do winget como fonte extra**: os manifestos do `winget-pkgs` têm link do instalador e SHA-256 para milhares de apps | winget-pkgs, UniGetUI | M | importador novo; a nota de confiança mostra a origem "winget" |
| 14 | **Inglês além do português** | Komi Store (14 idiomas) | M | extrair os textos de `gui.rs` |
| 15 | **Acessibilidade** (leitor de tela) pelo AccessKit, que o egui integra; nome acessível nos botões só com ícone | AccessKit | M | `Cargo.toml`, `gui.rs` |

[Não verificado] na #15: o nome exato da opção do AccessKit no egui 0.31 precisa ser conferido na documentação oficial.

### Fase 3 — grandes

| # | Ideia | Inspiração | Esforço |
|---|---|---|---|
| 16 | **Verificação criptográfica do atestado** (assinatura Sigstore e identidade do workflow) | GitHub artifact attestations | G |
| 17 | **Tradutor C/C++ com IA e laço do `rustc`** (detalhe abaixo) | DARPA TRACTOR, C2SaferRust, EvoC2Rust | G |
| 18 | **Biblioteca unificada**: importar os jogos de Steam, Epic e GOG para iniciar tudo pela RRS | Playnite | G |
| 19 | **Modo tela cheia com controle** (sofá e TV) | Playnite | G |
| 20 | **Ferramentas de terminal Rust pré-compiladas** (ripgrep, bat, fd...): metadados do crates.io → artefato da release → sem compilar | cargo-binstall | G |
| 21 | **Abrir o instalador no Windows Sandbox** (arquivo `.wsb` com a pasta do download só leitura e o comando de início) | Windows Sandbox | M |

Limites da #21: o Windows Sandbox não existe no Windows Home (só Pro, Enterprise e Education), e o seu PC é Home. A RRS teria de detectar se ele existe e esconder o botão quando não existir.

## Tradutor C/C++ → Rust: o que fazer com ele

Estado da arte (fontes abaixo):
- **c2rust** traduz C para Rust *unsafe* que espelha o C (versão 0.21.0). A parte de transformar em Rust seguro ainda está "em estágio inicial", e a verificação cruzada só roda em Linux.
- **C2SaferRust (2025)** usa o c2rust e depois IA. O artigo relata até 38% menos ponteiros brutos e até 28% menos código *unsafe*.
- **EvoC2Rust (ICSE-SEIP 2026)** usa IA e corrige pelos erros do compilador. Relata taxa de compilação de 17,24% a 50,84% melhor que a das abordagens só com IA.
- O **DARPA TRACTOR** financia exatamente essa combinação de análise de programa e IA.

Opções para a RRS:

| Opção | Como | Prós | Contras |
|---|---|---|---|
| A. Manter por padrões | o tradutor atual, mais avisos por linha não reconhecida e o rótulo "conversão assistida (experimental)" | sem custo, funciona offline | cobertura baixa (falhou num C comum no teste de hoje) |
| B. **IA + `rustc` em laço** (recomendada) | manda o C para um modelo de IA, compila com o `rustc` (a RRS já faz isso), devolve os erros ao modelo e repete até compilar ou até N tentativas | é o que a pesquisa mostra funcionar; reaproveita o `verify_rust_syntax` | precisa de chave de API do usuário (como a do VirusTotal) e tem custo por uso |
| C. Usar o c2rust | chamar o c2rust instalado | tradutor de verdade | gera Rust *unsafe*, exige `compile_commands.json`; [Não verificado] o suporte a Windows |
| D. Virar app separado | publicar o tradutor como um app dentro da própria RRS | a loja fica focada; o tradutor ganha vida própria | dois produtos para manter |

Recomendação: **A agora** (é a #6 da avaliação, rápida) e **B depois**, mostrando sempre se compilou e quantos blocos `unsafe` sobraram.
[Inferência] A opção D faz sentido se o tradutor crescer.

## O que eu não faria agora

- **Contas, avaliações e comentários de usuários**: exigem servidor, moderação e LGPD. Os sinais públicos do GitHub (estrelas, issues, atividade) cobrem parte disso sem servidor.
- **Sistema de plugins** (como o do Playnite): só vale depois que o núcleo estabilizar.

## Fontes

- Komi Store — repositório e descrição no F-Droid: https://github.com/kurikomi-labs/komi-store · https://f-droid.org/packages/zed.rainxch.githubstore/
- Obtainium (GIGAZINE) e ObtainX (F-Droid): https://gigazine.net/gsc_news/en/20260509-obtainium · https://f-droid.org/ja/packages/dev.bikram.obtainx/
- UniGetUI (Neowin): https://www.neowin.net/software/unigetui-310-final-wingetui-major-update/
- Playnite: https://www.techspot.com/downloads/7747-playnite.html · https://github.com/HerrKnarz/Playnite-Extensions
- Heroic Games Launcher: https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher · https://flathub.org/apps/com.heroicgameslauncher.hgl
- Flathub, selo de verificado: https://flathub.org/about · https://gamingonlinux.com/2024/04/flathub-now-prominently-notes-unverified-flatpak-apps
- GitHub, verificação de assets: https://cli.github.com/manual/gh_release_verify-asset · https://cli.github.com/manual/gh_release_verify
- TUF: https://theupdateframework.io/specification
- winget-pkgs, manifesto: https://learn.microsoft.com/windows/package-manager/package/manifest
- Scoop, autoupdate: https://github.com/lukesampson/scoop/wiki/App-Manifest-Autoupdate
- PortableApps.com: https://portableapps.com/node/30091
- cargo-binstall: https://github.com/cargo-bins/cargo-binstall
- Windows Sandbox, `.wsb`: https://learn.microsoft.com/en-us/windows/security/application-security/application-isolation/windows-sandbox/windows-sandbox-configure-using-wsb-file
- AccessKit: https://github.com/AccessKit/accesskit
- c2rust: https://github.com/immunant/c2rust · https://docs.rs/c2rust
- C2SaferRust: https://arxiv.org/abs/2501.14257
- DARPA TRACTOR: https://www.darpa.mil/program/translating-all-c-to-rust
- EvoC2Rust e panorama: https://www.softwareseni.com/ai-assisted-code-translation-from-c-to-rust-how-it-works-and-when-it-will-be-available/
