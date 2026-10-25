---
tipo: avaliacao
projeto: RRS - Riva Rust Store (gitstore 0.2.0)
data: 2026-10-24
autor: Claude (Opus 5.5)
---
# Avaliação da RRS — 24/10/2026

## Resumo

- **Compila sem avisos do compilador** (`cargo build`, rustc 1.99.1, MSVC). `cargo clippy`: 8 avisos de estilo. `cargo test`: **0 testes**.
- **3 problemas graves** (corrigidos na mesma noite; ver §Correções aplicadas): o "Desinstalar" do instalador apaga qualquer pasta digitada; o Windows Defender aparece como "limpo" quando falha; `cmd /C start` recebe texto do catálogo (risco de injeção de comando).
- **Tradutor C/C++**: passa nos 3 exemplos embutidos (validados pelo `rustc`); num arquivo C comum gera código inválido e não avisa.
- **Interface**: glifos que viram "☐", botões sobrepostos, editores do tradutor com 4 linhas visíveis, textos cortados.
- **Nota de confiança** não separa nada no catálogo atual: os 75 itens têm nota ≥ 70, então o filtro "Só Verificados (70+)" não esconde nenhum.

## Correções aplicadas (24/10/2026, noite)

Pedido do Fabricio: "Pode corrigir os 3 problemas graves (G1, G2, G3) da RRS e adicionar testes para eles." Cópia do código anterior em `Obsoleto/gitstore-src-antes-correcoes-24.10.26/`.

| Achado | Correção | Testes |
|---|---|---|
| G1 | A instalação grava `rrs-install.json`. O "Desinstalar" só aparece em pasta com esse marcador ou com o `RRS - Riva Rust Store.exe`, pede um segundo clique ("⚠ Confirmar") e apaga só os arquivos da RRS. A pasta `downloads/` e qualquer outro arquivo ficam; a pasta só é removida se ficar vazia. | 5 em `installer.rs`: pasta alheia recusada e intacta; só arquivos da RRS apagados; pasta vazia removida; instalação antiga sem marcador reconhecida; marcador válido |
| G2 | Três estados (`Clean`, `Threat`, `NotVerified`). Código ≠ 0 e 2, processo sem código ou falha ao chamar o Defender = "⚠ Não verificado", em amarelo. | 3 em `downloader.rs` |
| G3 | `cmd /C start` substituído por `ShellExecuteW` (Shell do Windows, sem interpretação de `& \| ^ %`). `open_url` só aceita `https://` sem espaços, aspas ou caracteres de controle; o erro aparece na barra de status. | 3 em `downloader.rs`, incluindo um teste que falha se algum módulo voltar a chamar `cmd` |

Verificação: `cargo build` sem avisos; `cargo test` com 11 de 11 testes passando; `cargo clippy` com os mesmos 8 avisos de estilo de antes (nenhum novo).
[Não verificado] na interface: abrir o "GitHub Oficial ↗" e iniciar um app pelo `ShellExecuteW`; o fluxo de confirmação do instalador.
Build release (`cargo build --release`, sem avisos, 14,7 MB, sha256 `9fcaf68d…7d63`): copiado para os dois `.exe` da raiz (hashes iguais). Testes: `search`, `show` e `translate --check` pela linha de comando; a janela abre com o título esperado. A cópia instalada em `%LOCALAPPDATA%` continua na versão anterior (o Fabricio atualiza pelo instalador). Os `.exe` anteriores estão em `Obsoleto/gitstore-exe-antes-24.10.26/` (registro e DESFAZER em `Obsoleto/*-24.10.26-gitstore-exe.*`).
Fica de fora: a desinstalação não tira a pasta da RRS do PATH do usuário (já era assim).

## O que foi usado nesta avaliação

| Ferramenta | Uso | Resultado |
|---|---|---|
| `cargo build` / `cargo test` | build de desenvolvimento e testes | OK, 0 testes |
| `cargo clippy --all-targets` | lint (componente instalado nesta sessão com `rustup component add clippy`) | 8 avisos |
| CLI da RRS em banco isolado (pasta temporária) | `search`, `show`, `translate --check` em 4 arquivos | ver §Tradutor |
| Computer use (capturas de tela) | versão instalada em `%LOCALAPPDATA%\Programs\RRS-RivaRustStore` (hash igual ao `.exe` do projeto) | ver §Interface |
| Leitura do código | `main.rs`, `github.rs`, `db.rs`, `verify.rs`, `downloader.rs`, `installer.rs` inteiros; `gui.rs` (cerca de 900 de 2.645 linhas) e `translator.rs` (trechos) | achados abaixo |
| SQLite (somente leitura) | `gitstore.db`: 75 apps, 0 instalados, tabela `settings` vazia (nenhum token salvo) | — |
| `verificar-vault.py` | início de sessão | sem alertas |
| Mod `guarda-do-vault` | ativo; protege as ações do Claude no vault (não é usado pela RRS) | nenhuma ação bloqueada |

**Não executados** (aplicáveis, ficam como próximo passo): `claude-security` (scan multiagente), `code-review` em nível alto (multiagente), `cargo audit` (não instalado; dependências **não auditadas**).
**Fora de escopo**: skills de finanças, vendas, jurídico, bio, marketing, Shopify, Gmail/Agenda e afins.

## Achados graves

**G1. "Desinstalar" do instalador apaga qualquer pasta** — `src/installer.rs:414-440` e `:110-116`.
O botão aparece para qualquer caminho existente no campo "Pasta de Instalação" e chama `remove_dir_all` nele, sem confirmação. Digitar `F:\NAS` e clicar em Desinstalar apaga o vault inteiro.
Correção: gravar um marcador na instalação (ex.: `rrs-install.json`) e só desinstalar uma pasta que o tenha; apagar só os arquivos conhecidos (`RRS - Riva Rust Store.exe`, `rrs.exe`, `catalog.json`, `gitstore.db`); pedir confirmação.

**G2. Defender "limpo" quando não verificou** — `src/downloader.rs:350-356` e `src/gui.rs:1990-1996`.
Código de saída diferente de 0 e 2 vira `is_clean: true` ("concluída sem incidentes"). Se o `MpCmdRun.exe` não puder ser chamado, a tela mostra em verde "Não foi possível acionar o Windows Defender".
Correção: três estados (Limpo / Ameaça / Não verificado), com "Não verificado" em amarelo.

**G3. `cmd /C start "" <texto>`** — `src/downloader.rs:264-290`, `:441-448`, `:486-491`.
O `cmd.exe` interpreta `&`, `|`, `^` e `%`. A URL vem do catálogo (`app.url`) e o caminho, do banco. Hoje as 75 URLs começam com `https://github.com/` e nenhuma tem `&` (conferido), então o risco é latente: aparece se o catálogo passar a vir de fora ("atualizado semanalmente").
Correção: abrir com `ShellExecuteW` (crates `open` ou `opener`) e aceitar só `https://`.

## Achados médios

- **M1. Alcance da verificação**: o digest SHA-256 vem da mesma API que o link de download. Ele detecta corrupção ou troca no transporte, mas não uma release maliciosa publicada pelo próprio dono. 19 dos 75 itens têm pelo menos um instalador sem digest e são baixados sem conferência (só aviso amarelo). A tela de detalhes do Ship of Harkinian diz "sem vírus", o que nenhuma verificação da RRS garante.
- **M2. Nota de confiança**: os 75 itens têm nota ≥ 70. No Ship of Harkinian, o cabeçalho mostra "Licença: Aberta" e a auditoria logo abaixo diz "Sem licença open-source padronizada"; mesmo sem licença e sem digest, sai com 78 "Verificado".
- **M3. "Atualizar Todos"** — `src/gui.rs:589-618` com `:313-328`. Os downloads em sequência dividem um único `download_state`, e o `mark_installed` só roda quando a tela vê o estado `Finished`. O download seguinte sobrescreve esse estado. [Inferência] Atualizações intermediárias podem não ser registradas, e o app continua mostrando "atualização disponível". Correção: chamar `mark_installed` dentro da própria tarefa.
- **M4. Nome do arquivo sem sanitização** — `src/downloader.rs:52` usa `asset.name` direto no caminho. [Não verificado] o GitHub normaliza nomes de assets, mas o catálogo local também alimenta esse campo. Correção: usar só `Path::new(&asset.name).file_name()`.
- **M5. Zero testes**. Há funções puras fáceis de cobrir: `platform_of`, `pick_best_asset`, `normalize_repo_name`, `last_page`, `urlencode`, `evaluate_metadata`, os 3 exemplos do tradutor e o banco com `Connection::open_in_memory()`.
- **M6. Tradutor** (detalhes abaixo): converte linha a linha por padrões e não avisa quando não reconhece uma construção.
- **M7. `target/` dentro do vault**: 5.279 arquivos e 4,9 GB em `Cods/Rust/RRS/target`. O Obsidian carrega tudo que não está em pasta com ponto, e o filtro do cofre `Cods` só esconde da busca (CLAUDE.md §4). Proposta: `target-dir` num disco local (ex.: `C:\cargo-target\gitstore`) no `.cargo/config.toml`. Isso tira os 4,9 GB do vault e [Inferência] acelera o build, porque o cache incremental volta a funcionar fora do F:.
- **M8. Git**: repositório próprio sem nenhum commit. Os dois `.exe` de 27 MB na raiz não estão no `.gitignore` e entrariam no histórico.

## Achados baixos

- **Clippy**: `&PathBuf` em vez de `&Path` (`main.rs:262, 387, 554`); `return` desnecessário (`downloader.rs:290`); `match` desnecessário (`github.rs:246`); variantes de tamanho muito diferente (`gui.rs:32`); `sort_by_key` (`gui.rs:293`); bloco reescrevível com `?` (`translator.rs:593`).
- Cerca de 20 `.lock().unwrap()`: um pânico numa thread envenena o mutex e derruba a interface.
- Arquivo `.part` não é removido se o download cair no meio (`downloader.rs:84`).
- Arquivo temporário fixo na validação pelo `rustc` (`translator.rs:361`): duas validações simultâneas colidem.
- Tokens do GitHub e do VirusTotal ficariam em texto puro no SQLite (hoje vazio). Proposta: Gerenciador de Credenciais do Windows (crate `keyring`).
- O texto do instalador diz "48 títulos" (`installer.rs:284`); o catálogo tem 75. O roadmap do `ARCHITECTURE.md` ainda lista como "Fase 1" o `egui_extras`, que já está no `Cargo.toml`.

## Tradutor C/C++ → Rust

| Entrada | Modo | Resultado no `rustc` |
|---|---|---|
| Exemplo 1 (struct, for, printf) | Seguro | OK |
| Exemplo 2 (std::vector, std::string, cout) | Seguro | OK |
| Exemplo 3 (enum, struct binária) | Seguro e FFI | OK |
| C comum: `typedef struct`, função de uma linha, `int arr[10]`, `malloc`, `switch`, `for` com `++i` | Seguro | **erro** (`typedef struct { ... }` copiado sem conversão) |

No 4º arquivo a saída mantém `typedef`, `switch`, `int arr[10]` e `for (int ...)` quase sem mudança, e troca `malloc(64)` por `String`. A única nota emitida é sobre o `free`. Proposta: avisar a cada linha não reconhecida e apresentar o recurso como "conversão assistida (experimental)". Para cobertura real, seria preciso um parser de C (ex.: `lang-c` ou `tree-sitter-c`) [Especulação sobre o esforço].

## Interface (capturas de tela, janela com cerca de 955×615 px)

- **Glifos ausentes**: "➔" e alguns emojis aparecem como "☐" (menu "Tradutor C/C++ ☐ Rust", barra de status, títulos de seção, "☐ Port / Recompilação"). A fonte padrão do egui não tem esses caracteres. Correção: trocar "➔" por "→" ou carregar uma fonte com cobertura via `FontDefinitions`.
- **Sobreposições**: o selo "✔" fica sobre "◀ Anterior" no destaque, e "Traduzir" sobre "Gerar Projeto Cargo".
- **Textos cortados**: "Plataformas: linux,macos,w", "Adicionar dor", "+ Verificar Repo" e os títulos das linhas da lista sob o painel de detalhes.
- **Tradutor**: os editores de entrada e saída mostram cerca de 4 linhas, com área vazia embaixo.
- [Não verificado] o layout no tamanho padrão de 1260×780.

## Pontos fortes

- 8 módulos com papéis claros; o `ARCHITECTURE.md` documenta as peculiaridades do ambiente (NAS, MSVC, BOM).
- SQL sempre parametrizado (sem injeção de SQL), TLS por `rustls`, hash calculado durante o download, gravação em `.part` com `rename` no fim.
- Caminhos passados ao PowerShell entre aspas simples, com `'` duplicado (escape correto).
- Um único binário que vira instalador pelo nome do arquivo, com catálogo embutido (funciona offline).

## Próximos passos — Proposta (pendente)

1. ~~Corrigir G1, G2 e G3~~ (feito em 24/10, com os `.exe` da raiz já trocados).
2. Testes das demais funções puras; `cargo clippy -- -D warnings` como verificação.
3. Mover o `target/` para fora do vault.
4. Rodar o `claude-security` e o `cargo audit`.
5. Fontes, glifos e layout da interface; textos de segurança sem promessas ("sem vírus", "Verificado").
