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
    pub level: &'static str,
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
        level: "Básico",
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
        level: "Básico",
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
        level: "Básico",
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
        level: "Básico",
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
        level: "Básico",
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
        level: "Básico",
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
        level: "Básico",
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
        level: "Básico",
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
        level: "Básico",
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
        level: "Básico",
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
    Lesson {
        id: "posse_e_emprestimo",
        title: "11. Posse (Ownership) & Empréstimo (&)",
        category: "Memória",
        level: "Intermediário",
        summary: "O coração de Rust: como o compilador gerencia memória sem Garbage Collector.",
        explanation: "\
Em Rust, cada valor tem um único 'dono' (owner). Quando o dono sai de escopo, a memória é liberada automaticamente (Drop).
Se você passar uma variável do tipo String para outra função ou variável, a posse é 'movida' (Move) e a original deixa de existir!
Para reutilizar o valor sem mover, usamos 'empréstimo' (Borrowing) com referências imutáveis `&T` ou mutáveis `&mut T`.
Regra de ouro: Você pode ter infinitas referências de leitura `&` OU exatamente uma referência de escrita `&mut`, nunca ambas ao mesmo tempo.",
        initial_code: "\
fn calcular_tamanho(texto: &String) -> usize {
    texto.len()
}

fn adicionar_saudacao(texto: &mut String) {
    texto.push_str(\" — Seja bem-vindo à RRS!\");
}

fn main() {
    let mut mensagem = String::from(\"Olá, Rustáceo\");

    // Passamos por referência & (empréstimo), a mensagem continua nossa
    let tamanho = calcular_tamanho(&mensagem);
    println!(\"Texto: '{}' (tamanho: {} bytes)\", mensagem, tamanho);

    // Empréstimo mutável &mut para alterar o conteúdo
    adicionar_saudacao(&mut mensagem);
    println!(\"Mensagem atualizada: {}\", mensagem);
}
",
        challenge: "Crie uma função `fn duplicar_texto(s: &mut String)` que adicione um espaço e repita o conteúdo de `s`.",
    },
    Lesson {
        id: "traits_e_interfaces",
        title: "12. Traits: Contratos e Polimorfismo",
        category: "Abstração",
        level: "Intermediário",
        summary: "Defina comportamentos compartilhados entre tipos diferentes sem herança clássica.",
        explanation: "\
Em Rust não há herança de classes (`class Dog extends Animal`). Em vez disso, usamos `traits`!
Uma trait define um conjunto de métodos que um tipo deve implementar. É o equivalente a interfaces em outras linguagens, mas muito mais poderoso e seguro.
Podemos usar traits para polimorfismo com `impl Trait` ou dynamic dispatch `&dyn Trait`.",
        initial_code: "\
trait Descrevivel {
    fn descrever(&self) -> String;

    // Método com implementação padrão opcional
    fn resumo(&self) -> String {
        format!(\"[Resumo]: {}\", self.descrever())
    }
}

struct Jogo {
    titulo: String,
    preco: f64,
}

struct Aplicativo {
    nome: String,
    versao: String,
}

impl Descrevivel for Jogo {
    fn descrever(&self) -> String {
        format!(\"Jogo '{}' por R$ {:.2}\", self.titulo, self.preco)
    }
}

impl Descrevivel for Aplicativo {
    fn descrever(&self) -> String {
        format!(\"App '{}' (v{})\", self.nome, self.versao)
    }
}

fn exibir_item(item: &impl Descrevivel) {
    println!(\"{}\", item.resumo());
}

fn main() {
    let g = Jogo { titulo: \"CyberRust 2077\".into(), preco: 149.90 };
    let a = Aplicativo { nome: \"RRS Launcher\".into(), versao: \"2.5\".into() };

    exibir_item(&g);
    exibir_item(&a);
}
",
        challenge: "Crie uma struct `Livro { titulo: String, paginas: u32 }`, implemente `Descrevivel` para ela e teste com `exibir_item`.",
    },
    Lesson {
        id: "tipos_genericos",
        title: "13. Tipos Genéricos (`<T>`)",
        category: "Abstração",
        level: "Intermediário",
        summary: "Escreva código flexível e reutilizável para múltiplos tipos com custo zero de runtime.",
        explanation: "\
Generics permitem escrever funções, structs e enums que operam sobre qualquer tipo de dado sem duplicar código.
Em Rust, generics usam 'monomorfização': o compilador gera uma cópia otimizada em código de máquina para cada tipo concreto usado, ou seja, zero perda de desempenho em comparação com código escrito à mão!
Podemos restringir generics usando 'trait bounds': `<T: Display>` ou `<T: PartialOrd>`.",
        initial_code: "\
struct Par<T> {
    primeiro: T,
    segundo: T,
}

impl<T: std::fmt::Display> Par<T> {
    fn exibir(&self) {
        println!(\"Par: ({}, {})\", self.primeiro, self.segundo);
    }
}

fn maior<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

fn main() {
    let par_inteiros = Par { primeiro: 10, segundo: 20 };
    let par_textos = Par { primeiro: \"Alpha\", segundo: \"Omega\" };

    par_inteiros.exibir();
    par_textos.exibir();

    println!(\"Maior número: {}\", maior(42, 100));
    println!(\"Maior texto: {}\", maior(\"Maçã\", \"Banana\"));
}
",
        challenge: "Adicione um método `fn trocar(&mut self)` na struct `Par<T>` para inverter a ordem dos dois elementos.",
    },
    Lesson {
        id: "closures_lambdas",
        title: "14. Closures (Funções Anônimas & Lambdas)",
        category: "Funcional",
        level: "Intermediário",
        summary: "Passe lógica como argumento, capture variáveis do ambiente e crie pipelines elegantes.",
        explanation: "\
Closures são funções anônimas que você pode salvar em variáveis ou passar como argumentos para outras funções.
A sintaxe usa barras verticais: `|param1, param2| { corpo }`.
O grande diferencial das closures é que elas podem capturar variáveis do escopo onde foram criadas (por referência imutável, mutável ou por movimento `move`).",
        initial_code: "\
fn main() {
    let multiplicador = 3;

    // Closure que captura 'multiplicador' do escopo externo
    let triplicar = |x: i32| x * multiplicador;

    println!(\"3 x 5 = {}\", triplicar(5));
    println!(\"3 x 12 = {}\", triplicar(12));

    let mut contador = 0;
    let mut incrementar = || {
        contador += 1;
        println!(\"Contador interno: {}\", contador);
    };

    incrementar();
    incrementar();
    incrementar();
}
",
        challenge: "Crie uma closure `let aplicar_desconto = |preco: f64| preco * 0.85;` e calcule o preço com 15% de desconto de R$ 200,00.",
    },
    Lesson {
        id: "iteradores_avancados",
        title: "15. Iteradores Funcionais (`map`, `filter`, `fold`)",
        category: "Funcional",
        level: "Intermediário",
        summary: "Processe coleções de dados de forma declarativa, expressiva e sem loops manuais.",
        explanation: "\
Em Rust, iteradores são 'preguiçosos' (lazy): nada é calculado até você consumir o iterador com `.collect()`, `.sum()` ou `.for_each()`.
Métodos como `.map()`, `.filter()`, `.take()` e `.fold()` transformam streams de dados com eficiência máxima, muitas vezes superando loops manuais em velocidade graças à vetorização do LLVM.",
        initial_code: "\
fn main() {
    let numeros = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    // Pipeline funcional: filtra pares, eleva ao quadrado e coleta
    let pares_quadrados: Vec<i32> = numeros
        .iter()
        .filter(|&&n| n % 2 == 0)
        .map(|&n| n * n)
        .collect();

    println!(\"Números originais: {:?}\", numeros);
    println!(\"Pares ao quadrado: {:?}\", pares_quadrados);

    // Soma acumulada usando fold
    let soma_total: i32 = numeros.iter().fold(0, |acc, &n| acc + n);
    println!(\"Soma total de 1 a 10: {}\", soma_total);
}
",
        challenge: "Adicione um filtro para selecionar apenas números ímpares maiores que 3 e multiplique todos eles.",
    },
    Lesson {
        id: "concorrencia_threads",
        title: "16. Concorrência Segura com Threads",
        category: "Concorrência",
        level: "Intermediário",
        summary: "Execute tarefas em paralelo em múltiplos núcleos da CPU sem risco de 'data race'.",
        explanation: "\
Rust garante 'Fearless Concurrency' (concorrência sem medo): o compilador impede em tempo de compilação condições de corrida de dados!
Com `std::thread::spawn` iniciamos uma nova thread do sistema operacional.
O método `.join()` aguarda a conclusão da thread e coleta seu resultado.",
        initial_code: "\
use std::thread;
use std::time::Duration;

fn main() {
    println!(\"Thread principal iniciando tarefas paralelas...\");

    let handle1 = thread::spawn(|| {
        for i in 1..=3 {
            println!(\"  [Worker 1] Processando lote {}...\", i);
            thread::sleep(Duration::from_millis(50));
        }
        \"Lote 1 Concluído!\"
    });

    let handle2 = thread::spawn(|| {
        for i in 1..=3 {
            println!(\"  [Worker 2] Indexando arquivos {}...\", i);
            thread::sleep(Duration::from_millis(70));
        }
        \"Lote 2 Concluído!\"
    });

    // Aguarda o término das threads
    let res1 = handle1.join().unwrap();
    let res2 = handle2.join().unwrap();

    println!(\"Resultados: {}, {}\", res1, res2);
    println!(\"Todas as tarefas concorrentes foram finalizadas!\");
}
",
        challenge: "Crie uma terceira thread que faça a contagem regressiva de 3 a 1 e aguarde todas com `.join()`.",
    },
    Lesson {
        id: "canais_mpsc",
        title: "17. Mensageria entre Threads (`mpsc`)",
        category: "Concorrência",
        level: "Intermediário",
        summary: "Comunicação entre threads por passagem de mensagens sem travas manuais.",
        explanation: "\
Em vez de usar travas complexas (mutex) que podem causar deadlocks, o canal `mpsc` (Multiple Producer, Single Consumer) permite enviar mensagens entre threads de forma limpa.
Você cria um par `(sender, receiver)` com `std::sync::mpsc::channel()`.
O transmissor envia dados com `tx.send(dado)` e o receptor lê com `rx.recv()` ou iterando diretamente sobre `rx`.",
        initial_code: "\
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    let (tx, rx) = mpsc::channel();

    let tx1 = tx.clone();
    thread::spawn(move || {
        let msgs = vec![\"Download iniciado\", \"50% baixado\", \"Download finalizado\"];
        for msg in msgs {
            tx1.send(format!(\"[Rede] {}\", msg)).unwrap();
            thread::sleep(Duration::from_millis(40));
        }
    });

    thread::spawn(move || {
        let msgs = vec![\"Verificando SHA256\", \"Assinatura digital OK\"];
        for msg in msgs {
            tx.send(format!(\"[Segurança] {}\", msg)).unwrap();
            thread::sleep(Duration::from_millis(60));
        }
    });

    // A thread principal recebe todas as mensagens até os transmissores fecharem
    while let Ok(msg) = rx.recv() {
        println!(\"Mensagem recebida: {}\", msg);
    }

    println!(\"Canal fechado. Todas as mensagens foram processadas!\");
}
",
        challenge: "Crie um canal onde uma thread envia o quadrado de números de 1 a 5 e a thread principal imprime cada um.",
    },
    Lesson {
        id: "smart_pointers_box",
        title: "18. Smart Pointers: Alocação na Heap com `Box<T>`",
        category: "Ponteiros",
        level: "Intermediário",
        summary: "Armazene dados diretamente na Heap e crie estruturas recursivas como listas encadeadas.",
        explanation: "\
Em Rust, a maioria dos tipos primitivos vive na Stack (rápida, tamanho fixo conhecido em compilação).
O `Box<T>` é o ponteiro inteligente mais simples: ele move o valor para a memória Heap e mantém apenas um ponteiro de 8 bytes na Stack.
É essencial para:
1. Estruturas com tamanho não conhecido em tempo de compilação (recursivas como árvores e listas encadeadas).
2. Transferir posse de grandes blocos de dados sem copiar bytes na memória.",
        initial_code: "\
// Estrutura recursiva: uma lista encadeada simples precisa de Box
enum ListaNode {
    Elemento(i32, Box<ListaNode>),
    Vazio,
}

use ListaNode::*;

fn imprimir_lista(mut no: &ListaNode) {
    print!(\"Lista: \");
    while let Elemento(valor, proximo) = no {
        print!(\"{} -> \", valor);
        no = proximo;
    }
    println!(\"FIM\");
}

fn main() {
    // Alocando um valor simples na Heap
    let valor_heap = Box::new(2026);
    println!(\"Valor dentro da Box na Heap: {}\", *valor_heap);

    // Construindo lista encadeada: 10 -> 20 -> 30 -> Vazio
    let lista = Elemento(10, Box::new(Elemento(20, Box::new(Elemento(30, Box::new(Vazio))))));
    imprimir_lista(&lista);
}
",
        challenge: "Crie uma lista encadeada com 4 números (100, 200, 300, 400) e exiba na tela.",
    },
    Lesson {
        id: "arquivos_e_io",
        title: "19. Arquivos & Sistema de Arquivos (`std::fs`)",
        category: "I/O",
        level: "Intermediário",
        summary: "Crie, leia e manipule arquivos e pastas no disco com tratamento robusto de erros.",
        explanation: "\
O módulo `std::fs` contém tudo o que você precisa para interagir com o sistema de arquivos do sistema operacional.
Métodos convenientes:
• `fs::write(\"caminho.txt\", \"conteudo\")`: cria ou sobrescreve um arquivo.
• `fs::read_to_string(\"caminho.txt\")`: lê o arquivo inteiro como texto.
• `fs::create_dir_all(\"pasta/sub\")`: cria pastas recursivamente.
Todas essas funções retornam `Result<T, std::io::Error>`, obrigando o tratamento limpo de permissões ou arquivos inexistentes.",
        initial_code: "\
use std::fs;

fn main() -> std::io::Result<()> {
    let pasta = std::env::temp_dir().join(\"rrs_demo_io\");
    fs::create_dir_all(&pasta)?;

    let arquivo = pasta.join(\"config_exemplo.ini\");
    let conteudo = \"modo=desempenho\\nfps_limite=144\\nloja_favorita=Steam\";

    // Grava o arquivo
    fs::write(&arquivo, conteudo)?;
    println!(\"Arquivo gravado com sucesso em: {}\", arquivo.display());

    // Lê de volta
    let lido = fs::read_to_string(&arquivo)?;
    println!(\"\\n--- Conteúdo Lido do Disco ---\");
    for (i, linha) in lido.lines().enumerate() {
        println!(\"Linha {}: {}\", i + 1, linha);
    }

    // Limpeza
    let _ = fs::remove_file(&arquivo);
    let _ = fs::remove_dir(&pasta);
    println!(\"\\nLimpeza de teste concluída!\");

    Ok(())
}
",
        challenge: "Adicione mais uma linha ao arquivo com `tema=escuro` e confira se ela é exibida ao ler.",
    },
    Lesson {
        id: "mini_projeto_cli_tarefas",
        title: "20. Mini-Projeto: Gerenciador de Tarefas CLI",
        category: "Projeto",
        level: "Intermediário",
        summary: "Um aplicativo completo de linha de comando com structs, enums, métodos e formatação tabular.",
        explanation: "\
Parabéns por completar o nível Intermediário de Rust!
Neste projeto final, construímos um Gerenciador de Tarefas (To-Do List) completo no terminal.
Usamos structs com métodos `impl`, enums para representar o status da tarefa, vetores dinâmicos e formatação tabular profissional.
Você pode exportar este projeto como um projeto Cargo completo com o botão acima!",
        initial_code: "\
#[derive(Debug, Clone)]
enum Status {
    Pendente,
    EmProgresso,
    Concluida,
}

struct Tarefa {
    id: usize,
    descricao: String,
    status: Status,
}

struct GerenciadorTarefas {
    itens: Vec<Tarefa>,
    proximo_id: usize,
}

impl GerenciadorTarefas {
    fn novo() -> Self {
        Self { itens: Vec::new(), proximo_id: 1 }
    }

    fn adicionar(&mut self, descricao: &str) {
        self.itens.push(Tarefa {
            id: self.proximo_id,
            descricao: descricao.to_string(),
            status: Status::Pendente,
        });
        self.proximo_id += 1;
    }

    fn concluir(&mut self, id: usize) -> bool {
        if let Some(t) = self.itens.iter_mut().find(|t| t.id == id) {
            t.status = Status::Concluida;
            true
        } else {
            false
        }
    }

    fn listar(&self) {
        println!(\"{:-<50}\", \"\");
        println!(\"{:<5} | {:<25} | {:<12}\", \"ID\", \"DESCRIÇÃO\", \"STATUS\");
        println!(\"{:-<50}\", \"\");
        for t in &self.itens {
            let status_txt = match t.status {
                Status::Pendente => \"⏳ Pendente\",
                Status::EmProgresso => \"⚙ Em Andamento\",
                Status::Concluida => \"✔ Concluída\",
            };
            println!(\"{:<5} | {:<25} | {:<12}\", t.id, t.descricao, status_txt);
        }
        println!(\"{:-<50}\", \"\");
    }
}

fn main() {
    println!(\"=== RRS Task Manager CLI ===\");
    let mut app = GerenciadorTarefas::novo();

    app.adicionar(\"Instalar Rust e Cargo\");
    app.adicionar(\"Aprender Ownership e Borrowing\");
    app.adicionar(\"Construir o primeiro projeto em Rust\");

    app.concluir(1);
    app.concluir(2);

    app.listar();
    println!(\"Total de tarefas cadastradas: {}\", app.itens.len());
}
",
        challenge: "Adicione uma função que conte quantas tarefas estão concluídas e quantas ainda estão pendentes.",
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
    } else if stderr.contains("cannot move out of") {
        Some("💡 Dica Intermediária: O valor foi movido e não pode mais ser acessado. Use referência `&` ou `.clone()` se o tipo permitir.".into())
    } else if stderr.contains("doesn't implement") || stderr.contains("the trait bound") {
        Some("💡 Dica Intermediária: O tipo não implementa a Trait necessária. Verifique os bounds `<T: ...>` ou adicione `impl Trait for Tipo`.".into())
    } else if stderr.contains("borrowed value does not live long enough") {
        Some("💡 Dica Intermediária: A referência tenta viver mais que o valor apontado. Ajuste o escopo das variáveis ou transfira a posse do valor.".into())
    } else if stderr.contains("aborting due to") {
        Some("💡 Dica Pedagógica: O compilador `rustc` encontrou erros. Leia a primeira linha de erro indicada acima para identificar a causa.".into())
    } else {
        None
    }
}

/// Compila e executa código Rust localmente com o compilador `rustc` do sistema.
/// Compilação e execução têm tempo máximo e saída limitada ([`crate::subprocess`]); `cancel`
/// é o botão Parar. Um `loop {}` de iniciante para sozinho em 10 s.
pub fn compile_and_run(code: &str, cancel: &std::sync::atomic::AtomicBool) -> RunResult {
    use crate::subprocess;
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
    let elapsed = || start.elapsed().as_millis() as u64;

    if let Err(e) = std::fs::write(&src_path, code) {
        return RunResult {
            success: false,
            output: format!("Erro ao criar arquivo temporário: {e}"),
            elapsed_ms: elapsed(),
            beginner_tip: None,
        };
    }

    // Compilação com rustc
    let mut compile = std::process::Command::new("rustc");
    compile.arg("--edition=2021").arg("-O").arg(&src_path).arg("-o").arg(&exe_path);
    let compile_output = subprocess::run_limited(compile, subprocess::COMPILE, Some(cancel));
    let _ = std::fs::remove_file(&src_path);

    let compile_res = match compile_output {
        Ok(out) => out,
        Err(e) => {
            let _ = std::fs::remove_file(&exe_path);
            return RunResult {
                success: false,
                output: format!("Não foi possível executar o `rustc`: {e}\nCertifique-se de que o Rust está instalado no seu computador."),
                elapsed_ms: elapsed(),
                beginner_tip: Some("💡 Instalação do Rust: Você pode instalar o Rust oficial pelo site https://rustup.rs.".into()),
            };
        }
    };

    if !compile_res.success() {
        let tip = diagnose_beginner_error(&compile_res.stderr);
        let _ = std::fs::remove_file(&exe_path);
        let header = match compile_res.notice(subprocess::COMPILE) {
            Some(n) => format!("⏹ Compilação interrompida ({n})."),
            None => "❌ Erro de Compilação:".to_string(),
        };
        return RunResult {
            success: false,
            output: format!("{header}\n{}", compile_res.stderr.trim()),
            elapsed_ms: elapsed(),
            beginner_tip: tip,
        };
    }

    // Execução do executável compilado
    let run_res = subprocess::run_limited(std::process::Command::new(&exe_path), subprocess::RUN, Some(cancel));
    let _ = std::fs::remove_file(&exe_path);
    let _ = std::fs::remove_file(exe_path.with_extension("pdb"));

    match run_res {
        Ok(out) => {
            let elapsed_ms = elapsed();
            let notice = out.notice(subprocess::RUN);
            if out.success() {
                let mut display_out = if out.stdout.is_empty() && out.stderr.is_empty() {
                    "✔ Programa executado com sucesso (nenhuma saída de texto gerada).".into()
                } else if !out.stderr.is_empty() {
                    format!("{}\n[Saída de Erro]:\n{}", out.stdout, out.stderr)
                } else {
                    out.stdout
                };
                if let Some(n) = notice {
                    display_out.push_str(&format!("\n⚠ {n}"));
                }
                RunResult {
                    success: true,
                    output: display_out,
                    elapsed_ms,
                    beginner_tip: None,
                }
            } else if let Some(n) = notice.filter(|_| out.timed_out || out.cancelled) {
                RunResult {
                    success: false,
                    output: format!("⏹ Programa interrompido ({n}).\n{}\n{}", out.stdout, out.stderr),
                    elapsed_ms,
                    beginner_tip: Some("💡 Dica: Um `loop` sem `break` (ou um `while` cuja condição nunca fica falsa) roda para sempre. Confira a condição de parada.".into()),
                }
            } else {
                RunResult {
                    success: false,
                    output: format!("❌ O programa encerrou com falha:\n{}\n{}", out.stdout, out.stderr),
                    elapsed_ms,
                    beginner_tip: Some("💡 Dica: Verifique se houve algum `panic!` no código (ex.: índice fora dos limites do vetor ou divisão inesperada).".into()),
                }
            }
        }
        Err(e) => RunResult {
            success: false,
            output: format!("Falha ao iniciar o programa compilado: {e}"),
            elapsed_ms: elapsed(),
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
        assert_eq!(LESSONS.len(), 20, "devem existir 20 lições (10 básico + 10 intermediário)");
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
        let res = compile_and_run(code, &std::sync::atomic::AtomicBool::new(false));
        if res.output.contains("Não foi possível executar o `rustc`") {
            // Em ambiente sem rustc no PATH de testes, ignora
            return;
        }
        assert!(res.success, "falha na execução: {}", res.output);
        assert!(res.output.contains("RRS_TESTE_OK_123"));
    }

    #[test]
    fn licao_com_laco_infinito_para_sozinha_com_dica() {
        let code = "fn main() { let mut n: u64 = 0; loop { n = n.wrapping_add(1); std::hint::black_box(n); } }";
        let res = compile_and_run(code, &std::sync::atomic::AtomicBool::new(false));
        if res.output.contains("Não foi possível executar o `rustc`") {
            return;
        }
        assert!(!res.success);
        assert!(res.output.contains("tempo máximo"), "{}", res.output);
        assert!(res.beginner_tip.as_deref().is_some_and(|t| t.contains("loop")));
        assert!(res.elapsed_ms < 40_000, "{}", res.elapsed_ms);
    }
}
