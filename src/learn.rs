//! Módulo de aprendizado interativo de Rust: lições práticas do zero, guia básico para iniciantes,
//! execução segura de código via `rustc`, diagnóstico amigável de erros e exportação de projetos Cargo.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Uma lição prática guiada de Rust para iniciantes.
#[derive(Debug, Clone)]
pub struct Lesson {
    pub id: &'static str,
    pub title: &'static str,
    pub category: &'static str,
    pub summary: &'static str,
    pub explanation: &'static str,
    pub initial_code: &'static str,
    pub challenge: &'static str,
}

/// Catálogo de lições ordenadas progressivamente do absoluto zero ao primeiro projeto.
pub const LESSONS: &[Lesson] = &[
    Lesson {
        id: "ola_mundo",
        title: "1. Olá, Mundo & Primeiro Programa",
        category: "Básico",
        summary: "Conheça a função main(), macros de impressão e como Rust exibe mensagens.",
        explanation: "\
Em Rust, todo programa executável começa na função `fn main()`.
A linha `println!(\"...\");` imprime um texto no terminal. O ponto de exclamação `!` indica que `println!` é uma macro, uma ferramenta poderosa de Rust que confere os formatos e tipos em tempo de compilação sem custo de velocidade.

Pontos-chave:
• `fn` declara uma nova função.
• `main()` é o ponto de partida oficial.
• Todo comando em Rust termina com ponto e vírgula `;`.",
        initial_code: "\
fn main() {
    println!(\"Olá, Rustáceo!\");
    println!(\"Bem-vindo ao espaço de aprendizado da RRS - Riva Rust Store.\");
    println!(\"Rust é moderna, ultra veloz e não tem vazamentos de memória!\");
}
",
        challenge: "Modifique o texto para colocar o seu nome e imprimir uma saudação personalizada.",
    },
    Lesson {
        id: "variaveis_mut",
        title: "2. Variáveis & Imutabilidade",
        category: "Básico",
        summary: "Descubra por que variáveis em Rust são imutáveis por padrão e quando usar `mut`.",
        explanation: "\
Em muitas linguagens, qualquer variável pode ser alterada a qualquer momento, o que causa bugs difíceis de rastrear.
Em Rust, variáveis declaradas com `let` são imutáveis por padrão. Se você tentar alterar o valor, o compilador recusa antes mesmo do programa rodar!

Para permitir que o valor mude, declare explicitamente com `let mut` (de mutável).
Isso torna o código autoexplicativo: quem lê sabe exatamente o que pode e o que não pode mudar.",
        initial_code: "\
fn main() {
    let mut pontuacao = 10;
    println!(\"Pontuação inicial: {}\", pontuacao);

    pontuacao += 15;
    println!(\"Pontuação após vencer o desafio: {}\", pontuacao);

    let vidas = 3;
    println!(\"Vidas fixas restantes: {}\", vidas);
}
",
        challenge: "Crie uma variável mutável `let mut ouro = 100;`, compre um item diminuindo 35 moedas e exiba o resultado.",
    },
    Lesson {
        id: "tipos_basicos",
        title: "3. Tipos de Dados & Cálculos",
        category: "Básico",
        summary: "Números inteiros, decimais, booleanos e conversão explícita segura.",
        explanation: "\
Rust é fortemente tipada:
• Inteiros: `i32` (número com sinal de 32 bits, padrão), `u32` (sem sinal, só positivo), `i64`, `u64`.
• Decimais: `f64` (padrão de precisão dupla) e `f32`.
• Booleanos: `bool` (`true` ou `false`).
• Texto: `&str` (fatia de texto) e `String` (texto dinâmico mutável).

Em Rust, nunca ocorrem conversões automáticas acidentais de tipo. Para converter números, use a palavra-chave `as` (ex.: `num as f64`).",
        initial_code: "\
fn main() {
    let vidas: i32 = 5;
    let ouro: u32 = 250;
    let bonus: f64 = 1.25;

    let ouro_final = (ouro as f64) * bonus;

    println!(\"Vidas: {}\", vidas);
    println!(\"Ouro acumulado: {}\", ouro);
    println!(\"Ouro após bônus de 25%: {:.2}\", ouro_final);
}
",
        challenge: "Calcule a média de 3 notas de um jogador: nota1 = 8.5, nota2 = 9.0, nota3 = 7.5 e imprima a média formatada.",
    },
    Lesson {
        id: "funcoes",
        title: "4. Funções & Retorno Elegante",
        category: "Lógica",
        summary: "Aprenda a criar funções limpas com retorno de expressão sem a palavra `return`.",
        explanation: "\
Funções são definidas com `fn nome(parametro: Tipo) -> TipoDeRetorno`.
Em Rust há uma diferença especial entre declaração (termina com `;`) e expressão (sem `;`).
A última linha de uma função, se não tiver ponto e vírgula, é automaticamente o valor retornado! Você ainda pode usar `return` explícito quando quiser sair mais cedo, mas a forma idiomática é usar expressões.",
        initial_code: "\
fn calcular_dano(forca: i32, armadura: i32) -> i32 {
    if forca > armadura {
        forca - armadura
    } else {
        1 // Dano mínimo garantido
    }
}

fn saudar(jogador: &str) {
    println!(\"Salve, nobre {}! Prepare-se para a aventura.\", jogador);
}

fn main() {
    saudar(\"Guerreiro\");
    let dano = calcular_dano(50, 20);
    println!(\"Dano infligido no monstro: {} pontos\", dano);
}
",
        challenge: "Crie uma função `fn calcular_area_retangulo(largura: f64, altura: f64) -> f64` e use-a dentro da `main()`.",
    },
    Lesson {
        id: "controle_fluxo",
        title: "5. Condições & O Poder do `match`",
        category: "Lógica",
        summary: "Decisões com `if/else` e o casador de padrões exaustivo `match`.",
        explanation: "\
O `if/else` em Rust pode ser usado tanto para controlar o fluxo quanto como expressão para atribuir valores:
`let status = if vida > 0 { \"vivo\" } else { \"derrotado\" };`

E o `match` é muito mais seguro que os switch/case de outras linguagens: o compilador confere se você cobriu todos os casos possíveis! Se esquecer de um caso, o código nem compila.",
        initial_code: "\
fn main() {
    let nivel = 4;

    let cargo = match nivel {
        1 => \"Novato da Guilda\",
        2..=3 => \"Aventureiro Experiente\",
        4..=6 => \"Veterano de Batalha\",
        _ => \"Lenda Viva\",
    };

    println!(\"Nível {} => Cargo: {}\", nivel, cargo);

    let xp = 1200;
    if xp >= 1000 {
        println!(\"Parabéns! Você tem XP suficiente para subir de rank.\");
    } else {
        println!(\"Faltam {} de XP para a próxima patente.\", 1000 - xp);
    }
}
",
        challenge: "Adicione mais um caso no `match` para o nível 10 com o título 'Grão-Mestre' e teste com nível 10.",
    },
    Lesson {
        id: "lacos_repeticao",
        title: "6. Laços de Repetição (`for`, `while`)",
        category: "Lógica",
        summary: "Iteração segura e expressiva sobre intervalos e coleções.",
        explanation: "\
Em Rust, raramente usamos laços com índices manuais que podem causar falhas de limite de vetor.
O `for` itera diretamente sobre intervalos como `1..=5` ou coleções de itens.
O `while` repete enquanto uma condição for verdadeira.
E o laço `loop` repete infinitamente até você executar um `break`.",
        initial_code: "\
fn main() {
    println!(\"--- Contagem Regressiva ---\");
    for segundo in (1..=5).rev() {
        println!(\"T-menos {} segundo(s)...\", segundo);
    }
    println!(\"🚀 Lançamento autorizado!\");

    println!(\"\\n--- Simulando Batalha por Turnos ---\");
    let mut chefe_vida = 30;
    let mut turno = 1;
    while chefe_vida > 0 {
        chefe_vida -= 10;
        println!(\"Turno {}: golpe acertado! Vida do chefe: {}\", turno, chefe_vida);
        turno += 1;
    }
    println!(\"Chefe derrotado com sucesso!\");
}
",
        challenge: "Crie um loop que imprima a tabuada do 7 de 1 até 10 usando `for`.",
    },
    Lesson {
        id: "structs_e_metodos",
        title: "7. Estruturas (`struct`) & Métodos (`impl`)",
        category: "Modelagem",
        summary: "Crie seus próprios tipos de dados com campos e comportamentos associados.",
        explanation: "\
As `structs` permitem agrupar dados relacionados.
Para adicionar métodos (comportamentos) à sua struct, usamos o bloco `impl`.
O primeiro parâmetro de um método é `&self` (leitura dos dados) ou `&mut self` (quando o método altera os campos da struct).",
        initial_code: "\
struct ItemLoja {
    nome: String,
    preco: u32,
    em_estoque: bool,
}

impl ItemLoja {
    fn novo(nome: &str, preco: u32) -> Self {
        Self {
            nome: nome.to_string(),
            preco,
            em_estoque: true,
        }
    }

    fn aplicar_desconto(&mut self, porcentagem: u32) {
        let desconto = (self.preco * porcentagem) / 100;
        self.preco = self.preco.saturating_sub(desconto);
    }
}

fn main() {
    let mut jogo = ItemLoja::novo(\"Super Rust Odyssey\", 120);
    println!(\"Item: {} | Preço original: R$ {}\", jogo.nome, jogo.preco);

    jogo.aplicar_desconto(20);
    println!(\"Item com 20% OFF: R$ {}\", jogo.preco);
}
",
        challenge: "Adicione um campo `quantidade: u32` na struct e um método `fn vender(&mut self) -> bool` que decrementa o estoque.",
    },
    Lesson {
        id: "vetores_colecoes",
        title: "8. Vetores & Coleções Dinâmicas",
        category: "Coleções",
        summary: "Listas dinâmicas que crescem e diminuem com segurança de memória.",
        explanation: "\
Um `Vec<T>` é uma lista que armazena elementos do mesmo tipo na memória heap.
Criamos com `vec![1, 2, 3]` ou `Vec::new()`.
Você pode adicionar elementos com `.push()`, consultar o tamanho com `.len()` e iterar com `.iter()` de forma rápida e segura.",
        initial_code: "\
fn main() {
    let mut inventario = vec![\"Espada de Ferro\", \"Poção de Vida\", \"Escudo de Madeira\"];

    inventario.push(\"Mapa da Dungeon\");

    println!(\"Itens na mochila (total {}):\", inventario.len());
    for (indice, item) in inventario.iter().enumerate() {
        println!(\"  [{}] {}\", indice + 1, item);
    }

    if inventario.contains(&\"Poção de Vida\") {
        println!(\"\\nVocê está preparado para a batalha com itens de cura!\");
    }
}
",
        challenge: "Crie um vetor com 5 números de pontuação, calcule a soma total e mostre a maior pontuação.",
    },
    Lesson {
        id: "option_e_result",
        title: "9. Sem Null: `Option` e `Result`",
        category: "Segurança",
        summary: "O fim dos crashes por NullPointerException: aprenda a tratar ausências e erros.",
        explanation: "\
O criador do ponteiro nulo (null) chamou-o de seu 'erro de um bilhão de dólares'.
Em Rust, não existe null!
• Para valores que podem estar ausentes: `Option<T>` com `Some(valor)` ou `None`.
• Para operações que podem falhar: `Result<T, E>` com `Ok(resultado)` ou `Err(motivo)`.

O compilador obriga você a tratar o caso de erro, garantindo que o programa nunca quebre sem aviso.",
        initial_code: "\
fn dividir(dividendo: f64, divisor: f64) -> Result<f64, String> {
    if divisor == 0.0 {
        Err(\"Divisão por zero não é permitida!\".to_string())
    } else {
        Ok(dividendo / divisor)
    }
}

fn buscar_usuario(id: u32) -> Option<&'static str> {
    match id {
        1 => Some(\"Fabricio\"),
        2 => Some(\"Riva\"),
        _ => None,
    }
}

fn main() {
    match dividir(100.0, 4.0) {
        Ok(resultado) => println!(\"100 / 4 = {}\", resultado),
        Err(e) => println!(\"Erro: {}\", e),
    }

    match buscar_usuario(1) {
        Some(nome) => println!(\"Usuário encontrado: {}\", nome),
        None => println!(\"Usuário não cadastrado!\"),
    }
}
",
        challenge: "Modifique a chamada para dividir por 0.0 e veja como o Rust trata o erro graciosamente sem travar.",
    },
    Lesson {
        id: "mini_jogo",
        title: "10. Mini-Projeto: Jogo de Adivinhação",
        category: "Projeto",
        summary: "Um jogo interativo completo reunindo variáveis, condições, loops e lógica.",
        explanation: "\
Parabéns por chegar até aqui!
Neste mini-projeto, combinamos tudo o que você aprendeu: funções, variáveis mutáveis, laços, condições e formatação para criar um jogo de adivinhação completo.

Você pode editar as regras, aumentar as tentativas e adicionar mais desafios ao código!",
        initial_code: "\
fn main() {
    println!(\"========================================\");
    println!(\"   🎯 MINI-JOGO DE ADIVINHAÇÃO RRS     \");
    println!(\"========================================\");

    let numero_secreto = 42;
    let palpites = [15, 60, 38, 42];
    let max_tentativas = 5;

    println!(\"O computador pensou em um número entre 1 e 100.\");
    println!(\"Simulando palpites do jogador...\\n\");

    let mut acertou = false;

    for (indice, &palpite) in palpites.iter().enumerate() {
        let tentativa = indice + 1;
        print!(\"Tentativa {}/{}: Palpite [{}] => \", tentativa, max_tentativas, palpite);

        if palpite < numero_secreto {
            println!(\"📈 Muito baixo!\");
        } else if palpite > numero_secreto {
            println!(\"📉 Muito alto!\");
        } else {
            println!(\"🎉 ACERTOU! O número secreto era {}!\", numero_secreto);
            acertou = true;
            break;
        }
    }

    if !acertou {
        println!(\"\\nFim de jogo! Tente novamente na próxima rodada.\");
    } else {
        println!(\"\\nVitória! Você completou o mini-projeto com sucesso!\");
    }
}
",
        challenge: "Mude o `numero_secreto` para outro número e teste com novos palpites no vetor.",
    },
];

/// Resultado da execução ou compilação de código Rust.
#[derive(Debug, Clone)]
pub struct RunResult {
    pub success: bool,
    pub output: String,
    pub elapsed_ms: u64,
    pub beginner_tip: Option<String>,
}

/// Diagnóstico pedagógico em português para erros comuns cometidos por iniciantes no rustc.
pub fn diagnose_beginner_error(stderr: &str) -> Option<String> {
    if stderr.contains("cannot borrow as mutable") || stderr.contains("cannot borrow `*self` as mutable") {
        Some("💡 Dica para Iniciantes: Em Rust, variáveis são imutáveis por padrão. Adicione `mut` na declaração (exemplo: `let mut x = ...` ou `&mut self`).".into())
    } else if stderr.contains("cannot assign twice to immutable variable") || stderr.contains("cannot assign to immutable variable") {
        Some("💡 Dica para Iniciantes: Você tentou alterar uma variável imutável. Adicione `mut` na declaração dela: `let mut ...`".into())
    } else if stderr.contains("mismatched types") {
        Some("💡 Dica para Iniciantes: Os tipos de dados não combinam. Em Rust, conversões numéricas são explícitas; use `as i32`, `as f64`, etc.".into())
    } else if stderr.contains("cannot find value") || stderr.contains("cannot find function") {
        Some("💡 Dica para Iniciantes: O compilador não encontrou esse nome. Verifique se digitou o nome correto ou se declarou a variável antes de usá-la.".into())
    } else if stderr.contains("expected `;`") || stderr.contains("expected one of `;`") {
        Some("💡 Dica para Iniciantes: Parece que faltou um ponto e vírgula `;` no final da linha anterior.".into())
    } else if stderr.contains("unused variable") {
        Some("💡 Dica para Iniciantes: A variável foi criada mas nunca usada. Se foi intencional, coloque um sublinhado antes do nome (ex: `_x`) para silenciar o aviso.".into())
    } else if stderr.contains("aborting due to") {
        Some("💡 Dica para Iniciantes: O compilador `rustc` encontrou erros de sintaxe ou tipos. Leia a primeira linha de erro indicada acima.".into())
    } else {
        None
    }
}

/// Compila e executa código Rust localmente de forma segura com o compilador `rustc` do sistema.
pub fn compile_and_run(code: &str) -> RunResult {
    let start = Instant::now();
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let temp_dir = std::env::temp_dir();
    let stem = format!("rrs_learn_{}_{n}", std::process::id());
    let src_path = temp_dir.join(format!("{stem}.rs"));
    let exe_path = temp_dir.join(if cfg!(target_os = "windows") {
        format!("{stem}.exe")
    } else {
        stem
    });

    if let Err(e) = std::fs::write(&src_path, code) {
        return RunResult {
            success: false,
            output: format!("Erro ao criar arquivo temporário: {e}"),
            elapsed_ms: start.elapsed().as_millis() as u64,
            beginner_tip: None,
        };
    }

    // Compilação com rustc
    let compile_output = std::process::Command::new("rustc")
        .arg("--edition=2021")
        .arg("-O")
        .arg(&src_path)
        .arg("-o")
        .arg(&exe_path)
        .output();

    let _ = std::fs::remove_file(&src_path);

    let compile_res = match compile_output {
        Ok(out) => out,
        Err(e) => {
            let _ = std::fs::remove_file(&exe_path);
            return RunResult {
                success: false,
                output: format!("Não foi possível executar o `rustc`: {e}\nCertifique-se de que o Rust está instalado no seu computador."),
                elapsed_ms: start.elapsed().as_millis() as u64,
                beginner_tip: Some("💡 Instalação do Rust: Você pode instalar o Rust oficial pelo site https://rustup.rs.".into()),
            };
        }
    };

    if !compile_res.status.success() {
        let stderr = String::from_utf8_lossy(&compile_res.stderr).to_string();
        let tip = diagnose_beginner_error(&stderr);
        let _ = std::fs::remove_file(&exe_path);
        return RunResult {
            success: false,
            output: format!("❌ Erro de Compilação:\n{}", stderr.trim()),
            elapsed_ms: start.elapsed().as_millis() as u64,
            beginner_tip: tip,
        };
    }

    // Execução do executável compilado
    let run_res = std::process::Command::new(&exe_path).output();

    let _ = std::fs::remove_file(&exe_path);

    match run_res {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let elapsed_ms = start.elapsed().as_millis() as u64;

            if out.status.success() {
                let display_out = if stdout.is_empty() && stderr.is_empty() {
                    "✔ Programa executado com sucesso (nenhuma saída de texto gerada).".into()
                } else if !stderr.is_empty() {
                    format!("{}\n[Saída de Erro]:\n{}", stdout, stderr)
                } else {
                    stdout
                };

                RunResult {
                    success: true,
                    output: display_out,
                    elapsed_ms,
                    beginner_tip: None,
                }
            } else {
                RunResult {
                    success: false,
                    output: format!("❌ O programa encerrou com falha:\n{}\n{}", stdout, stderr),
                    elapsed_ms,
                    beginner_tip: Some("💡 Dica: Verifique se houve algum `panic!` no código (ex.: índice fora dos limites do vetor ou divisão inesperada).".into()),
                }
            }
        }
        Err(e) => RunResult {
            success: false,
            output: format!("Falha ao iniciar o programa compilado: {e}"),
            elapsed_ms: start.elapsed().as_millis() as u64,
            beginner_tip: None,
        },
    }
}

/// Gera uma estrutura de projeto Cargo pronta para abrir em qualquer editor.
pub fn generate_beginner_cargo_project(dest_dir: &Path, project_name: &str, code: &str) -> Result<PathBuf> {
    let clean_name = project_name
        .trim()
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_");
    let clean_name = if clean_name.is_empty() {
        "meu_primeiro_rust".to_string()
    } else {
        clean_name
    };

    let target_dir = dest_dir.join(&clean_name);
    let src_dir = target_dir.join("src");
    std::fs::create_dir_all(&src_dir)
        .with_context(|| format!("falha ao criar pasta do projeto em {}", target_dir.display()))?;

    let cargo_toml = format!(
        r#"[package]
name = "{clean_name}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#
    );

    let gitignore = r#"/target
**/*.rs.bk
"#;

    let readme = format!(
        r#"# {clean_name}

Projeto Rust criado automaticamente pela **RRS - Riva Rust Store**.

## Como Executar

Abra o terminal nesta pasta e execute:
```bash
cargo run
```

Para compilar em modo otimizado:
```bash
cargo build --release
```
"#
    );

    std::fs::write(target_dir.join("Cargo.toml"), cargo_toml)?;
    std::fs::write(target_dir.join(".gitignore"), gitignore)?;
    std::fs::write(target_dir.join("README.md"), readme)?;
    std::fs::write(src_dir.join("main.rs"), code)?;

    Ok(target_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn todas_as_licoes_tem_id_titulo_e_codigo() {
        assert_eq!(LESSONS.len(), 10, "devem existir 10 lições para iniciantes");
        let mut ids = std::collections::HashSet::new();
        for l in LESSONS {
            assert!(ids.insert(l.id), "id de lição repetido: {}", l.id);
            assert!(!l.title.is_empty());
            assert!(!l.explanation.is_empty());
            assert!(l.initial_code.contains("fn main()"), "a lição {} deve ter uma main() executável", l.id);
        }
    }

    #[test]
    fn diagnostico_de_erros_identifica_erros_comuns() {
        assert!(diagnose_beginner_error("cannot borrow as mutable").unwrap().contains("mut"));
        assert!(diagnose_beginner_error("mismatched types").unwrap().contains("tipos de dados"));
        assert!(diagnose_beginner_error("cannot find value `x`").unwrap().contains("não encontrou"));
        assert!(diagnose_beginner_error("expected `;`").unwrap().contains("ponto e vírgula"));
    }

    #[test]
    fn geracao_de_projeto_cargo_cria_arquivos_corretos() {
        let temp = std::env::temp_dir().join(format!("rrs_test_cargo_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp);

        let code = "fn main() { println!(\"teste\"); }";
        let proj = generate_beginner_cargo_project(&temp, "Meu Teste", code).unwrap();

        assert!(proj.join("Cargo.toml").exists());
        assert!(proj.join("src/main.rs").exists());
        assert!(proj.join("README.md").exists());

        let saved = std::fs::read_to_string(proj.join("src/main.rs")).unwrap();
        assert_eq!(saved, code);

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn compilacao_e_execucao_de_codigo_valido_funciona() {
        let code = r#"
            fn main() {
                println!("RRS_TESTE_OK_123");
            }
        "#;
        let res = compile_and_run(code);
        if res.output.contains("Não foi possível executar o `rustc`") {
            // Em ambiente sem rustc no PATH de testes, ignora
            return;
        }
        assert!(res.success, "falha na execução: {}", res.output);
        assert!(res.output.contains("RRS_TESTE_OK_123"));
    }
}
