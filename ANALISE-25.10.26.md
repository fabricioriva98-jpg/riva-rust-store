---
tipo: analise
projeto: RRS - Riva Rust Store (gitstore 0.3.0)
data: 2026-10-25
autor: Claude (Opus 5.5)
---
# Análise da RRS — 25/10/2026 (otimização, segurança e integridade)

Pedido do Fabricio: "quero que voce analise o projeto, fiz varias melhorias, quero que vocce analise e veja como esta se precisa de otimizações e como esta a seguranã e integridade de todo projeto e do codigo".

Escopo: as 5 rodadas do Antigravity/Gemini de 24–25/10 (logo, Aprender Rust, lojas de jogos, Otimizador do PC, Biblioteca Rust, Livro, Terminal e IDEs, temas, seletor de desempenho), comparadas com a cópia `Obsoleto/gitstore-src-antes-logo-aprender-24.10.26` (o código de ontem, antes delas). O resto do código foi revisado em 24/10 (`AVALIACAO-24.10.26.md`) e não mudou desde então.

Nada foi alterado no código nesta análise.

**Atualização (25/10, 14:00): G1, G2 e G3 corrigidos** a pedido do Fabricio ("Pode corrigir os 3 problemas graves (G1, G2, G3) da análise, com testes, e gerar os .exe."). Resultado medido no release novo (sha256 `f7c4b445…`): 0,0% de CPU parada nos modos Normal e Eco e minimizada; aba Terminal sem nenhum PowerShell (só a detecção inicial, em segundo plano); limpeza com confirmação e só de arquivos com mais de 7 dias, conferida com um %TEMP% de teste. Testes: 123 (7 novos). Achado durante a correção: na versão anterior, **nenhum** botão "⚡ Otimizar Agora" respondia ao clique nem ao mouse (o botão ficava alinhado à direita dentro da linha do título); foi para uma linha própria e agora funciona. Ou seja, a limpeza perigosa existia no código, mas não era acionável pela tela.

## 1. Resumo

| Área | Situação |
|---|---|
| Testes | 116 passam (eram 93); nenhum teste antigo foi removido; o teste de tradução passou a cobrir os arquivos novos |
| Clippy | só os 8 avisos antigos; 10 `#[allow(dead_code)]` novos escondem código sem uso |
| `.exe` da raiz | correspondem ao código atual (nenhum arquivo mais novo que o build; sha256 `d243274e…`, igual a `target/release`) |
| Exemplos de código | as 20 lições e os 22 exemplos da biblioteca padrão compilam no `rustc` (conferido); os 20 exemplos de crates externos não foram testados |
| Dependências | 543 crates conferidos na base OSV: nenhuma vulnerabilidade no build do Windows (detalhe na seção 5) |
| Desempenho | **piorou muito**: parada, a loja usa 40% de um núcleo (era 0%); a aba Terminal abre PowerShell sem parar |
| Segurança | nenhuma falha que permita a terceiros executar código; os riscos novos são do próprio usuário (apagar %TEMP% com 1 clique, comandos sem limite de tempo) |
| Escopo do Gemini | mexeu só na RRS, no diário e nas notas do dia; fez cópia no `Obsoleto` antes de cada rodada |

## 2. Graves (corrigidos em 25/10, 14:00; detalhe no topo)

**G1. A loja não descansa mais: CPU alta mesmo parada.** `src/gui.rs:1067` pede uma nova pintura da tela a cada quadro em todos os modos. Antes, o egui só redesenhava quando algo mudava. Medido com o release, janela parada por 10 s:

| Versão | Uso de CPU (1 núcleo = 100%) |
|---|---|
| ontem (`bc5d59c9`) | 0,0% |
| hoje, modo Normal | 40,5% |
| hoje, modo **Eco** | 28,4% |
| hoje, Normal, janela minimizada | 50,8% |

O modo Eco gasta mais do que a versão de ontem em qualquer situação. Correção: pedir pintura só quando houver animação, download ou tarefa em andamento (como já fazem `gui.rs:758`, `gui.rs:1063` e `gui/social.rs:57`); o seletor passa a valer só para esses casos.

**G2. A aba Terminal & IDEs abre PowerShell a cada quadro.** `src/gui/terminal_studio.rs:127` chama `ide::detect_ides()` dentro do desenho da tela, e `ide::find_in_path` (`src/ide.rs:159`) abre um PowerShell para cada editor que não está num caminho fixo. Neste PC: 3 buscas por quadro, 873 ms medidos; com a aba aberta, 27 PowerShell em 8 s. A tela fica lenta e o PC trabalha à toa. Na primeira abertura, `detect_rust_env()` (`terminal_studio.rs:73-75`) também roda na linha da interface (5 programas em sequência). Correção: detectar uma vez, em segundo plano, e guardar o resultado.

**G3. "Limpar %TEMP%" apaga sem perguntar.** O botão "⚡ Otimizar Agora" (`src/gui/optimizer.rs:312-320`) chama `apply_optimization("clean_temp")` (`src/optimizer.rs:487-509`), que apaga **todos** os arquivos da raiz do `%TEMP%`, de qualquer idade, para sempre, sem confirmação. Pode apagar arquivos de instaladores ou programas abertos (o Rust abre arquivos com permissão de exclusão compartilhada). Correção: confirmar, apagar só arquivos com mais de 7 dias (como a Limpeza de Disco do Windows), mostrar a quantidade antes.

## 3. Médios

**Atualização (25/10, 14:40): M1 a M6 corrigidos** a pedido do Fabricio ("Pode corrigir os problemas médios M1 a M6 da análise, com testes, e gerar os .exe."). Release `37e9cb7e…`; testes 137 (14 novos); clippy com os 8 avisos antigos. Resumo:
- M1: módulo `subprocess.rs` (tempo máximo, botão Parar que encerra o processo e os filhos com `taskkill /T`, saída limitada). Terminal 30 min e 1 MB; lições e avaliador: compilação 60 s, execução 10 s. Avaliador saiu da linha da interface. Terminal não usa mais `busy_task` (a loja continua usável). Console limitado a 5.000 linhas. Conferido na tela: `Start-Sleep -Seconds 120` parado pelo botão.
- M2: pasta padrão `Documentos\Projetos Rust` (lida do Windows; neste PC `E:\DOC PC\Projetos Rust`); campo para trocar a pasta; projeto criado vira a pasta de trabalho; aviso de que os comandos rodam com as permissões da conta; teste que só aceita o campo digitado e os atalhos fixos. A exportação das lições também foi para essa pasta (era o B3).
- M3: sem valores inventados (mostra "Não foi possível ler"); núcleos físicos lidos do Windows (antes: threads ÷ 2); Modo Jogo sem chave = padrão ligado; textos sem promessas; plano de energia anterior guardado e botão "↩ Voltar ao plano anterior". O cartão de discos deixou de sobrepor texto.
- M4: dica de excluir pasta do Defender trocada pelo Dev Drive (fonte Microsoft Learn, consultada em 25/10/2026: todas as edições do Windows 11 a partir do build 22621.2338; o Defender usa o modo de desempenho, "alternativa segura" à exclusão de pastas).
- M5: o botão usa o `bin\code.cmd` da instalação detectada (teste com `--version`: VS Code 1.127.0 respondeu), roda em segundo plano e valida o id da extensão. A extensão não foi instalada no teste.
- M6: caminhos `E:\Fabricio\...` e `C:\Users\fabri\...` removidos (teste cobra); VS Code achado por `%LOCALAPPDATA%`, `%ProgramFiles%` ou PATH. Saiu também o instalador pendente do Antigravity da lista de executáveis (abrir uma pasta com ele rodaria a instalação).

| # | Problema | Onde | Correção sugerida |
|---|---|---|---|
| M1 | Terminal, avaliador e lições rodam **sem limite de tempo** e sem botão Parar. Um `loop {}` numa lição deixa um processo a 100% para sempre. Um comando longo no terminal (`cargo run` de um servidor) prende o `busy_task`, que desativa botões da loja inteira até reiniciar. A saída cresce sem limite na memória | `terminal_studio.rs:61-76`, `:257-260`; `learn.rs:913`; `gui/terminal_studio.rs:20-21` | tempo máximo (ex.: 10 s para lições, 10 min no terminal), botão Parar (`Child::kill`), limite de saída |
| M2 | O terminal executa qualquer comando no PowerShell. Isso é o propósito dele e hoje só recebe o que o usuário digita ou os botões fixos (conferido). Mas a pasta padrão é a de dados da RRS (`gitstore.db`, com as chaves de API); no modo portátil é a própria pasta do projeto | `gui/terminal_studio.rs:53-55` | pasta padrão em Documentos; regra (e teste) de nunca passar texto do catálogo, do GitHub ou da IA para `run_terminal_command` |
| M3 | Textos do Otimizador que não correspondem aos fatos. (a) Se o PowerShell falhar, mostra **valores inventados** (16 GB de RAM, disco C: de 256 GB com 68% usado) como se fossem do PC (`optimizer.rs:168-169`, `:207-213`). (b) Modo Jogo: sem a chave no registro, informa "Desativado"; [Inferência] no Windows 10/11 ele vem ligado por padrão, então a recomendação pode ser falsa (`optimizer.rs:362-366`). (c) "alocando núcleos de CPU e memória dedicados" e "até 40%" sem fonte. (d) Recomenda Alto Desempenho para todos, sem avisar de bateria e calor em notebook, e sem guardar o plano anterior para desfazer (`optimizer.rs:447-472`) | `optimizer.rs` | mostrar "não foi possível ler"; tratar chave ausente como padrão do Windows; textos sem promessa; guardar o GUID do plano anterior e oferecer "Desfazer" |
| M4 | A dica de **excluir a pasta de projetos do Windows Defender** contraria a proposta de loja que verifica segurança | `optimizer.rs:427-439`, `:511-513` | sugerir o Dev Drive (modo de desempenho do Defender) [Não verificado: disponibilidade no Windows 11 Home] ou avisar do risco |
| M5 | O botão "Instalar rust-analyzer no VS Code" **não funciona neste PC**: `Command::new("code")` procura `code.exe`, e o PATH só tem `code.cmd` (teste: "program not found") | `ide.rs:229` | usar o caminho detectado (`bin\code.cmd`, com argumentos fixos) |
| M6 | **Caminhos pessoais no código distribuído**: `E:\Fabricio\Programas\...` e `C:\Users\fabri\...` ficam dentro do `.exe` que vai para outras pessoas e só servem neste PC | `ide.rs:85-86`, `:128` | remover; a busca por `%LOCALAPPDATA%` e PATH já cobre |
| M7 | **Sem controle de versão**: o git do projeto não tem nenhum commit. Com duas IAs alternando, o histórico existe só como 14 cópias da pasta `src` no `Obsoleto` | `.git` | Proposta (pendente): um commit local (sem push) ao fim de cada rodada |
| M8 | `target/` dentro do vault cresceu de 4,9 GB para **8,9 GB** | `target/` | Proposta (pendente) de 24/10: `target-dir` no disco C: |

**Atualização (25/10, 15:10): M7 e M8 feitos.** M7: git local com 12 commits, reconstruídos das cópias do `Obsoleto` (uma por rodada, com a data da cópia e a IA que fez) mais o estado atual; sem remoto, sem push; `.exe` e banco fora do git. M8: build em `C:\cargo-targetrs`; a pasta `target` antiga (5,0 GB em 9.137 arquivos, 8,9 GB ocupados no exFAT) foi para `Obsoleto/.gitstore-target-antes-m8-25.10.26` (nome com ponto, o Obsidian não carrega), com CSV e DESFAZER. Release compilou no C: (2 min 14 s, primeira vez) e os 137 testes passam; recompilar depois de mexer num arquivo levou 14 s, com cache incremental.

## 4. Baixos

- **B1. 6 links quebrados (404, conferidos)** no Livro: capítulos 5 e 20 em inglês; 5, 7, 19 e 20 em português (`rust_lib.rs:104-105`, `:120-121`, `:216-217`, `:224-225`). A edição atual do livro tem 21 capítulos (o projeto final virou o 21), e a tradução `rust-br` é de uma edição antiga. Os outros 81 links respondem.
- **B2. "Espelho de IDEs" promete mais do que faz**: hoje só abre a pasta no editor. `list_project_files` e `open_file_in_ide` (`ide.rs:198-296`) não são usados (escondidos com `#[allow(dead_code)]`). A biblioteca tem 42 itens; o diário fala em "mais de 45".
- **B3. Exportar projeto Cargo** grava na pasta de dados da RRS (no modo portátil, dentro do vault) e sobrescreve o `main.rs` de uma exportação anterior sem perguntar (`learn.rs:1006-1009`, `gui/learn.rs:136-138`).
- **B4. Nome temporário previsível no avaliador**: `Instant::now().elapsed()` é sempre perto de zero, então duas avaliações ao mesmo tempo podem usar o mesmo arquivo (`terminal_studio.rs:227`). O `learn.rs` já usa um contador e está certo.
- **B5. Idioma**: lições, biblioteca e textos do Otimizador (`optimizer.rs`) aparecem em português no modo inglês; o teste de tradução não olha esses arquivos.
- **B6. Imagens**: `egui_extras` com `all_loaders` (`Cargo.toml:13`) inclui leitores de SVG, GIF e WebP para imagens que vêm da internet (avatares, ícones). Deixar só os formatos usados reduz o tamanho e a superfície de ataque.
- **B7. Documentação**: `CONTINUIDADE-IA.md` diz 103 testes (são 116); as decisões das rodadas do Gemini (tema padrão, ritmo de pintura) não têm ADR.
- **B8. Tamanho do `.exe`** (17,7 MB): o perfil release não tem `strip` nem `lto`; [Inferência] os dois juntos reduzem bastante o arquivo.

## 5. Dependências (base OSV, 25/10/2026)

543 crates do `Cargo.lock` consultados em `api.osv.dev` (o `cargo audit` não está instalado).

- **Nenhuma vulnerabilidade no build do Windows.**
- `quick-xml 0.30.0` tem 2 avisos de negação de serviço (RUSTSEC-2026-0194 e 0195), mas só entra no build do Linux (`cargo tree --target x86_64-pc-windows-msvc` não o encontra).
- Avisos de "sem manutenção" no Windows: `paste 1.0.15` (RUSTSEC-2024-0436, só roda na compilação) e `ttf-parser 0.25.1` (RUSTSEC-2026-0192, leitura das fontes do egui). Os dois saem ao atualizar o egui (0.31 hoje). `zvariant` e `zbus_names` são só do Linux.

## 6. O que está bom

- Nenhum token, chave ou senha no código. Os endereços das lojas novas (Xbox, Ubisoft, EA, Battle.net) passam por lista restrita (`launchers.rs:572-591`); links continuam só `https`; o teste contra `cmd.exe` continua ativo.
- Download, verificação de SHA-256, catálogo assinado, login do GitHub e perfil não mudaram desde a revisão de 24/10.
- O ícone embutido pelo `build.rs` não quebra a compilação se faltar o `rc.exe`.
- O Otimizador só **lê** hardware e programas (registro e PowerShell com texto fixo); mudanças só com clique.
- Cópia no `Obsoleto` com CSV antes de cada rodada; nenhum arquivo fora da RRS, do diário e das notas foi alterado.

## 7. Próximos passos — Proposta (pendente), em ordem

1. G1 e G2 (CPU parada e PowerShell por quadro): maior ganho, pouco código.
2. G3 e M3 (confirmação e filtro de idade no %TEMP%; textos e dados honestos; desfazer o plano de energia).
3. M1 (tempo máximo e botão Parar).
4. M5, M6, B1 (botão do VS Code, caminhos pessoais, links).
5. M7 e M8 (commits locais; `target` fora do vault).
6. Claude Security (scan) e `cargo audit`.

## 8. [Não verificado]

- Ações reais do Otimizador (plano de energia, Modo Jogo, limpeza do %TEMP%): não executadas, porque mudam o sistema ou apagam arquivos. Análise pelo código.
- Detecção de jogos do Xbox, EA, Ubisoft, Battle.net e Riot em instalações reais.
- Os 6 temas na tela (só o padrão apareceu nas capturas).
- Os 20 exemplos que dependem de crates externos (serde, tokio, bevy...).
- Travamento do terminal com comando longo: conclusão pela leitura do código (`Command::output()` espera o fim do processo).
