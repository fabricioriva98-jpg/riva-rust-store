//! Catálogo e Referência Completa da Biblioteca Rust (Standard Library `std` & Top Crates do Ecossistema).
//! Fornece documentação em português, assinatura de tipos essenciais, métodos principais, capítulos do Livro Oficial
//! e exemplos de código executáveis prontos para testar no editor da RRS e integrar a projetos Cargo.

/// Categoria de módulo ou biblioteca Rust.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibCategory {
    Collections,
    SyncConcurrency,
    FsIo,
    Net,
    SystemOs,
    MemoryTypes,
    Ecosystem,
    GameDev,
    Database,
}

impl LibCategory {
    pub fn label(self) -> &'static str {
        match self {
            LibCategory::Collections => "Coleções (std::collections)",
            LibCategory::SyncConcurrency => "Concorrência & Sincronização (std::sync, atomics)",
            LibCategory::FsIo => "Arquivos & Entrada/Saída (std::fs, std::io)",
            LibCategory::Net => "Redes & Web (std::net, axum, reqwest)",
            LibCategory::SystemOs => "Processos & Sistema (std::process, std::env, sysinfo)",
            LibCategory::MemoryTypes => "Tipos & Memória (std::cell, std::mem, Box)",
            LibCategory::Ecosystem => "Top Crates do Ecossistema (serde, tokio, clap, etc.)",
            LibCategory::GameDev => "Jogos & Gráficos (bevy, wgpu)",
            LibCategory::Database => "Bancos de Dados (sqlx, rusqlite)",
        }
    }
}

/// Representa um módulo da Standard Library ou Crate do ecossistema Rust.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustLibItem {
    pub id: &'static str,
    pub name: &'static str,
    pub category: LibCategory,
    pub summary: &'static str,
    pub description: &'static str,
    pub key_types: &'static [&'static str],
    pub key_methods: &'static [&'static str],
    pub example_code: &'static str,
    pub is_std: bool,
    pub cargo_dep: Option<&'static str>,
    pub docs_url: &'static str,
}

/// Capítulo do livro oficial "The Rust Programming Language" (O Livro de Rust).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustBookChapter {
    pub number: usize,
    pub title: &'static str,
    pub summary: &'static str,
    pub topics: &'static [&'static str],
    pub web_url_en: &'static str,
    pub web_url_pt: &'static str,
}

pub const RUST_BOOK_PDF_URL: &str = "https://doc.rust-lang.org/stable/book/print.html";
pub const RUST_BOOK_EN_URL: &str = "https://doc.rust-lang.org/book/";
pub const RUST_BOOK_PT_URL: &str = "https://rust-br.github.io/rust-book-pt-br/";

pub const RUST_BOOK_CHAPTERS: &[RustBookChapter] = &[
    RustBookChapter {
        number: 1,
        title: "Começando (Getting Started)",
        summary: "Instalação do rustup, primeiro Olá Mundo com rustc e introdução ao gerenciador de pacotes Cargo.",
        topics: &["rustup", "rustc", "cargo new", "cargo build", "cargo run"],
        web_url_en: "https://doc.rust-lang.org/book/ch01-00-getting-started.html",
        web_url_pt: "https://rust-br.github.io/rust-book-pt-br/ch01-00-getting-started.html",
    },
    RustBookChapter {
        number: 2,
        title: "Programando um Jogo de Adivinhação",
        summary: "Primeiro projeto prático: let, mut, match, io::stdin(), tratamento de erros e crate rand.",
        topics: &["std::io", "mut", "rand", "cmp::Ordering", "loop", "match"],
        web_url_en: "https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html",
        web_url_pt: "https://rust-br.github.io/rust-book-pt-br/ch02-00-guessing-game-tutorial.html",
    },
    RustBookChapter {
        number: 3,
        title: "Conceitos Comuns de Programação",
        summary: "Variáveis, imutabilidade por padrão, tipos primitivos, funções e controle de fluxo.",
        topics: &["variables", "shadowing", "types", "functions", "if/else", "loops"],
        web_url_en: "https://doc.rust-lang.org/book/ch03-00-common-programming-concepts.html",
        web_url_pt: "https://rust-br.github.io/rust-book-pt-br/ch03-00-common-programming-concepts.html",
    },
    RustBookChapter {
        number: 4,
        title: "Entendendo Ownership (Posse)",
        summary: "O conceito central de Rust: ownership, regras da pilha e heap, referências, borrowing e slices.",
        topics: &["ownership rules", "move semantics", "&references", "&mut", "slices"],
        web_url_en: "https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html",
        web_url_pt: "https://rust-br.github.io/rust-book-pt-br/ch04-00-understanding-ownership.html",
    },
    RustBookChapter {
        number: 5,
        title: "Usando Structs para Dados Relacionados",
        summary: "Definindo e instanciando structs, structs clássicas, tuple structs e blocos de métodos impl.",
        topics: &["struct", "impl", "self", "associated functions", "derive(Debug)"],
        web_url_en: "https://doc.rust-lang.org/book/ch05-00-using-structs-to-structure-related-data.html",
        web_url_pt: "https://rust-br.github.io/rust-book-pt-br/ch05-00-using-structs-to-structure-related-data.html",
    },
    RustBookChapter {
        number: 6,
        title: "Enums e Pattern Matching",
        summary: "Enums com dados acoplados, o operador match, Option<T> sem null e sintaxe if let.",
        topics: &["enum", "Option<T>", "match", "exhaustive patterns", "if let"],
        web_url_en: "https://doc.rust-lang.org/book/ch06-00-enums.html",
        web_url_pt: "https://rust-br.github.io/rust-book-pt-br/ch06-00-enums.html",
    },
    RustBookChapter {
        number: 7,
        title: "Gerenciando Projetos Crescentes com Pacotes, Crates e Módulos",
        summary: "Sistema de módulos, caminhos absolutos e relativos, pub, use e separação em arquivos.",
        topics: &["packages", "crates", "mod", "pub", "use", "super"],
        web_url_en: "https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html",
        web_url_pt: "https://rust-br.github.io/rust-book-pt-br/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html",
    },
    RustBookChapter {
        number: 8,
        title: "Coleções Comuns (Vector, String, HashMap)",
        summary: "Estruturas dinâmicas na heap: Vec<T>, strings UTF-8 e HashMap<K, V>.",
        topics: &["Vec<T>", "String vs &str", "UTF-8", "HashMap", "entry api"],
        web_url_en: "https://doc.rust-lang.org/book/ch08-00-common-collections.html",
        web_url_pt: "https://rust-br.github.io/rust-book-pt-br/ch08-00-common-collections.html",
    },
    RustBookChapter {
        number: 9,
        title: "Tratamento de Erros",
        summary: "Erros irrecuperáveis com panic! e erros recuperáveis com Result<T, E> e o operador ?.",
        topics: &["panic!", "Result<T, E>", "? operator", "unwrap / expect"],
        web_url_en: "https://doc.rust-lang.org/book/ch09-00-error-handling.html",
        web_url_pt: "https://rust-br.github.io/rust-book-pt-br/ch09-00-error-handling.html",
    },
    RustBookChapter {
        number: 10,
        title: "Tipos Genéricos, Traits e Lifetimes",
        summary: "Abstração com tipos genéricos, polimorfismo via traits e garantia de validade com lifetimes ('a).",
        topics: &["generics <T>", "traits", "trait bounds", "lifetimes 'a"],
        web_url_en: "https://doc.rust-lang.org/book/ch10-00-generics.html",
        web_url_pt: "https://rust-br.github.io/rust-book-pt-br/ch10-00-generics.html",
    },
    RustBookChapter {
        number: 11,
        title: "Escrevendo Testes Automatizados",
        summary: "Anatomia de testes unitários com #[test], assert!, assert_eq!, should_panic e testes de integração.",
        topics: &["#[test]", "assert_eq!", "cargo test", "integration tests"],
        web_url_en: "https://doc.rust-lang.org/book/ch11-00-testing.html",
        web_url_pt: "https://rust-br.github.io/rust-book-pt-br/ch11-00-testing.html",
    },
    RustBookChapter {
        number: 12,
        title: "Projeto de E/S: Construindo o Minigrep",
        summary: "Projeto completo construindo uma ferramenta CLI como o grep: argumentos, leitura de arquivo e TDD.",
        topics: &["CLI arguments", "std::fs", "minigrep", "TDD", "refactoring"],
        web_url_en: "https://doc.rust-lang.org/book/ch12-00-an-io-project.html",
        web_url_pt: "https://rust-br.github.io/rust-book-pt-br/ch12-00-an-io-project.html",
    },
    RustBookChapter {
        number: 13,
        title: "Recursos de Linguagens Funcionais: Iteradores e Closures",
        summary: "Closures capturando ambiente, a trait Iterator, adaptadores de iterador e custo zero de abstração.",
        topics: &["closures", "Fn/FnMut/FnOnce", "Iterator", "map/filter", "zero-cost"],
        web_url_en: "https://doc.rust-lang.org/book/ch13-00-functional-features.html",
        web_url_pt: "https://rust-br.github.io/rust-book-pt-br/ch13-00-functional-features.html",
    },
    RustBookChapter {
        number: 14,
        title: "Mais sobre o Cargo e o Crates.io",
        summary: "Perfis de compilação release/dev, documentação com rustdoc, workspaces e publicação no crates.io.",
        topics: &["cargo profiles", "/// rustdoc", "cargo workspaces", "crates.io"],
        web_url_en: "https://doc.rust-lang.org/book/ch14-00-more-about-cargo.html",
        web_url_pt: "https://rust-br.github.io/rust-book-pt-br/ch14-00-more-about-cargo.html",
    },
    RustBookChapter {
        number: 15,
        title: "Smart Pointers (Ponteiros Inteligentes)",
        summary: "Box<T> para heap, Rc<T> para contagem de referências, RefCell<T> para mutabilidade interior e drop.",
        topics: &["Box<T>", "Deref", "Drop", "Rc<T>", "RefCell<T>"],
        web_url_en: "https://doc.rust-lang.org/book/ch15-00-smart-pointers.html",
        web_url_pt: "https://rust-br.github.io/rust-book-pt-br/ch15-00-smart-pointers.html",
    },
    RustBookChapter {
        number: 16,
        title: "Concorrência Sem Medo (Fearless Concurrency)",
        summary: "Threads com std::thread, passagem de mensagens por canais mpsc e estado compartilhado com Arc<Mutex<T>>.",
        topics: &["std::thread", "mpsc::channel", "Arc<T>", "Mutex<T>", "Send / Sync"],
        web_url_en: "https://doc.rust-lang.org/book/ch16-00-concurrency.html",
        web_url_pt: "https://rust-br.github.io/rust-book-pt-br/ch16-00-concurrency.html",
    },
    RustBookChapter {
        number: 17,
        title: "Recursos de Orientação a Objetos em Rust",
        summary: "Encapsulamento, polimorfismo dinâmico com trait objects (dyn Trait) e implementação de State Pattern.",
        topics: &["OOP in Rust", "dyn Trait", "dispatch dynamic vs static", "state pattern"],
        web_url_en: "https://doc.rust-lang.org/book/ch17-00-oop.html",
        web_url_pt: "https://rust-br.github.io/rust-book-pt-br/ch17-00-oop.html",
    },
    RustBookChapter {
        number: 18,
        title: "Padrões e Matching",
        summary: "Onde padrões podem ser usados (match, if let, while let, for), padrões refutáveis e guards.",
        topics: &["pattern syntax", "destructuring", "guards @", "refutable patterns"],
        web_url_en: "https://doc.rust-lang.org/book/ch18-00-patterns.html",
        web_url_pt: "https://rust-br.github.io/rust-book-pt-br/ch18-00-patterns.html",
    },
    RustBookChapter {
        number: 19,
        title: "Recursos Avançados",
        summary: "Unsafe Rust, traits avançadas, tipos avançados, ponteiros brutos e macros declarativas/procedurais.",
        topics: &["unsafe", "raw pointers", "advanced traits", "macros", "procedural macros"],
        web_url_en: "https://doc.rust-lang.org/book/ch19-00-advanced-features.html",
        web_url_pt: "https://rust-br.github.io/rust-book-pt-br/ch19-00-advanced-features.html",
    },
    RustBookChapter {
        number: 20,
        title: "Projeto Final: Servidor Web Multithreaded",
        summary: "Projeto final: TCP listener, parsing de requisições HTTP e criação de um ThreadPool com canais.",
        topics: &["std::net::TcpListener", "HTTP parsing", "ThreadPool", "graceful shutdown"],
        web_url_en: "https://doc.rust-lang.org/book/ch20-00-final-project-a-multithreaded-web-server.html",
        web_url_pt: "https://rust-br.github.io/rust-book-pt-br/ch20-00-final-project-a-multithreaded-web-server.html",
    },
];

pub const RUST_LIBS: &[RustLibItem] = &[
// ─── COLEÇÕES ─────────────────────────────────────────────────────────────
    RustLibItem {
        id: "std_hashmap",
        name: "std::collections::HashMap",
        category: LibCategory::Collections,
        summary: "Tabela hash de chave-valor com complexidade média O(1) e proteção contra ataques HashDoS.",
        description: "\
O `HashMap<K, V>` armazena pares de chave e valor utilizando uma função de espalhamento (hasher criptograficamente seguro SipHash 1-3 por padrão).
É a estrutura de dados ideal para contadores, caches na memória, índices e mapeamentos rápidos.
As chaves precisam implementar as traits `Eq` e `Hash`.",
        key_types: &["HashMap<K, V>", "Entry<K, V>"],
        key_methods: &[
            ".insert(k, v) -> Option<V>",
            ".get(&k) -> Option<&V>",
            ".entry(k).or_insert(v)",
            ".remove(&k) -> Option<V>",
            ".contains_key(&k) -> bool",
        ],
        example_code: r#"use std::collections::HashMap;

fn main() {
    let mut inventario = HashMap::new();

    // Inserção simples
    inventario.insert("Espada de Rust", 3);
    inventario.insert("Poção de Vida", 10);
    inventario.insert("Escudo Élfico", 1);

    // Atualização com API de Entry (super idiomático em Rust)
    *inventario.entry("Poção de Vida").or_insert(0) += 5;

    println!("=== Inventário do Jogador ===");
    for (item, qtd) in &inventario {
        println!("  • {}: {} unidade(s)", item, qtd);
    }

    if let Some(qtd) = inventario.get("Poção de Vida") {
        println!("\nTotal de poções disponíveis: {}", qtd);
    }
}
"#,
        is_std: true,
        cargo_dep: None,
        docs_url: "https://doc.rust-lang.org/std/collections/struct.HashMap.html",
    },
    RustLibItem {
        id: "std_btreemap",
        name: "std::collections::BTreeMap",
        category: LibCategory::Collections,
        summary: "Mapa ordenado baseado em B-Tree com busca, inserção e remoção em O(log n).",
        description: "\
Diferente do `HashMap`, o `BTreeMap<K, V>` mantém todas as chaves ordenadas de acordo com a trait `Ord`.
É ideal quando você precisa iterar pelas chaves em ordem crescente, fazer buscas por faixa de valores (`.range()`) ou quando prefere consumo de memória compacto e previsível sem dispersão de buckets.",
        key_types: &["BTreeMap<K, V>", "Range<K, V>"],
        key_methods: &[
            ".insert(k, v)",
            ".range(min..max)",
            ".first_key_value()",
            ".last_key_value()",
        ],
        example_code: r#"use std::collections::BTreeMap;

fn main() {
    let mut pontuacoes = BTreeMap::new();
    pontuacoes.insert("Carlos", 450);
    pontuacoes.insert("Alice", 920);
    pontuacoes.insert("Beatriz", 780);

    println!("=== Ranking Ordenado Alfabeticamente ===");
    for (nome, pontos) in &pontuacoes {
        println!("  {:<10} => {} pts", nome, pontos);
    }

    // Busca por intervalo de chaves
    println!("\nJogadores de A a B:");
    for (nome, pontos) in pontuacoes.range("A"..="B\u{ffff}") {
        println!("  • {}: {}", nome, pontos);
    }
}
"#,
        is_std: true,
        cargo_dep: None,
        docs_url: "https://doc.rust-lang.org/std/collections/struct.BTreeMap.html",
    },
    RustLibItem {
        id: "std_vecdeque",
        name: "std::collections::VecDeque",
        category: LibCategory::Collections,
        summary: "Fila de duas pontas (Double-Ended Queue) baseada em buffer anular com inserção e remoção O(1) em ambas as extremidades.",
        description: "\
O `VecDeque<T>` é perfeito para implementar filas (FIFO), pilhas (LIFO), buffers circulares e filas de tarefas assíncronas.
Inserir ou remover itens no início (`.push_front()`, `.pop_front()`) ou no final (`.push_back()`, `.pop_back()`) custa apenas O(1) amortizado, ao contrário de um `Vec` tradicional que precisaria mover todos os elementos.",
        key_types: &["VecDeque<T>"],
        key_methods: &[
            ".push_back(val)",
            ".pop_front() -> Option<T>",
            ".push_front(val)",
            ".pop_back() -> Option<T>",
        ],
        example_code: r#"use std::collections::VecDeque;

fn main() {
    let mut fila_tarefas = VecDeque::new();

    // Enfileirando tarefas
    fila_tarefas.push_back("1. Verificar integridade SHA-256");
    fila_tarefas.push_back("2. Baixar instalador da release");
    fila_tarefas.push_back("3. Criar atalhos no Desktop");

    // Adiciona uma tarefa prioritária no início da fila
    fila_tarefas.push_front("0. [URGENTE] Conferir espaço em disco");

    println!("Processando fila FIFO:");
    while let Some(tarefa) = fila_tarefas.pop_front() {
        println!("  ▶ Executando: {}", tarefa);
    }
    println!("Todas as tarefas foram concluídas!");
}
"#,
        is_std: true,
        cargo_dep: None,
        docs_url: "https://doc.rust-lang.org/std/collections/struct.VecDeque.html",
    },
    RustLibItem {
        id: "std_binary_heap",
        name: "std::collections::BinaryHeap",
        category: LibCategory::Collections,
        summary: "Fila de prioridade (Priority Queue) baseada em Max-Heap com recuperação do maior elemento em O(1).",
        description: "\
O `BinaryHeap<T>` organiza seus elementos de forma que o maior item (de acordo com `Ord`) sempre fica no topo.
É amplamente utilizado no desenvolvimento de jogos (ordenação de eventos por tempo, algoritmo A* de pathfinding) e agendamento de tarefas prioritárias.",
        key_types: &["BinaryHeap<T>"],
        key_methods: &[
            ".push(val) -> O(log n)",
            ".pop() -> Option<T> -> O(log n)",
            ".peek() -> Option<&T> -> O(1)",
        ],
        example_code: r#"use std::collections::BinaryHeap;

#[derive(Eq, PartialEq)]
struct Tarefa {
    prioridade: u32,
    descricao: &'static str,
}

impl Ord for Tarefa {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.prioridade.cmp(&other.prioridade)
    }
}

impl PartialOrd for Tarefa {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn main() {
    let mut heap = BinaryHeap::new();
    heap.push(Tarefa { prioridade: 10, descricao: "Atualizar catálogo em segundo plano" });
    heap.push(Tarefa { prioridade: 99, descricao: "Responder clique do usuário na interface" });
    heap.push(Tarefa { prioridade: 50, descricao: "Verificar assinatura digital do executável" });

    println!("Executando tarefas por ordem de urgência (Max-Heap):");
    while let Some(t) = heap.pop() {
        println!("  [Prioridade {:>2}] {}", t.prioridade, t.descricao);
    }
}
"#,
        is_std: true,
        cargo_dep: None,
        docs_url: "https://doc.rust-lang.org/std/collections/struct.BinaryHeap.html",
    },

    // ─── CONCORRÊNCIA E SINCRONIZAÇÃO ──────────────────────────────────────────
    RustLibItem {
        id: "std_sync_arc_mutex",
        name: "std::sync::{Arc, Mutex}",
        category: LibCategory::SyncConcurrency,
        summary: "Ponteiro atômico compartilhado (Arc) com trava de exclusão mútua segura (Mutex).",
        description: "\
A combinação `Arc<Mutex<T>>` é a forma canônica em Rust de compartilhar dados mutáveis entre múltiplas threads de forma segura sem risco de corrupção ou data races.
O `Arc` (Atomically Reference Counted) gerencia a contagem de referências atômicas na memória Heap.
O `Mutex` garante que apenas uma thread acesse os dados internos por vez. Em Rust, o Mutex guarda os dados dentro de si e destrava automaticamente ao sair de escopo (Drop)!",
        key_types: &["Arc<T>", "Mutex<T>", "MutexGuard<'a, T>"],
        key_methods: &[
            "Arc::clone(&arc)",
            "mutex.lock().unwrap()",
            "mutex.try_lock()",
        ],
        example_code: r#"use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    // Contador compartilhado entre 5 threads
    let contador = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for id in 1..=5 {
        let contador_clone = Arc::clone(&contador);
        let h = thread::spawn(move || {
            for _ in 0..100 {
                let mut num = contador_clone.lock().unwrap();
                *num += 1;
            }
            println!("Thread #{} completou 100 incrementos.", id);
        });
        handles.push(h);
    }

    for h in handles {
        h.join().unwrap();
    }

    println!("\nValor final do contador: {}", *contador.lock().unwrap());
}
"#,
        is_std: true,
        cargo_dep: None,
        docs_url: "https://doc.rust-lang.org/std/sync/struct.Mutex.html",
    },
    RustLibItem {
        id: "std_sync_rwlock",
        name: "std::sync::RwLock",
        category: LibCategory::SyncConcurrency,
        summary: "Trava de leitura e escrita: múltiplas threads lendo simultaneamente ou exatamente uma gravando.",
        description: "\
O `RwLock<T>` (Reader-Writer Lock) maximiza a taxa de transferência em cenários onde leituras são muito mais frequentes que escritas (exemplo: catálogo de jogos em memória, lista de configurações).
Permite infinitos leitores concorrentes (`.read()`), mas exige acesso exclusivo para escrita (`.write()`).",
        key_types: &["RwLock<T>", "RwLockReadGuard", "RwLockWriteGuard"],
        key_methods: &[
            "lock.read().unwrap()",
            "lock.write().unwrap()",
        ],
        example_code: r#"use std::sync::RwLock;

fn main() {
    let config = RwLock::new(String::from("Modo Normal"));

    // Múltiplas leituras concorrentes
    {
        let r1 = config.read().unwrap();
        let r2 = config.read().unwrap();
        println!("Leitor 1 lê: {}", *r1);
        println!("Leitor 2 lê: {}", *r2);
    } // r1 e r2 saem de escopo e liberam a trava de leitura

    // Escrita exclusiva
    {
        let mut w = config.write().unwrap();
        *w = String::from("Modo Alto Desempenho RRS");
        println!("\nConfiguração atualizada com trava exclusiva!");
    }

    println!("Novo valor: {}", *config.read().unwrap());
}
"#,
        is_std: true,
        cargo_dep: None,
        docs_url: "https://doc.rust-lang.org/std/sync/struct.RwLock.html",
    },
    RustLibItem {
        id: "std_sync_mpsc",
        name: "std::sync::mpsc",
        category: LibCategory::SyncConcurrency,
        summary: "Canais de mensageria: Multi-Producer, Single-Consumer para envio de mensagens entre threads.",
        description: "\
Implementa o lema de concorrência de Rust: 'Não comunique compartilhando memória; compartilhe memória comunicando'.
Permite clonar múltiplos transmissores (`Sender`) que despacham dados de workers assíncronos para um único receptor central (`Receiver`).",
        key_types: &["Sender<T>", "Receiver<T>", "channel()"],
        key_methods: &[
            "tx.send(msg) -> Result<(), SendError<T>>",
            "rx.recv() -> Result<T, RecvError>",
            "rx.try_recv() -> Result<T, TryRecvError>",
        ],
        example_code: r#"use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    let (tx, rx) = mpsc::channel();

    // Spawn de dois produtores paralelos
    for id in 1..=2 {
        let tx_clone = tx.clone();
        thread::spawn(move || {
            let msg = format!("Pacote de telemetria da Thread #{}", id);
            thread::sleep(Duration::from_millis(id * 50));
            tx_clone.send(msg).unwrap();
        });
    }
    drop(tx); // Fecha o canal original para que o loop rx termine

    // Consumidor central
    for recebido in rx {
        println!("✔ Recebido no canal: {}", recebido);
    }
}
"#,
        is_std: true,
        cargo_dep: None,
        docs_url: "https://doc.rust-lang.org/std/sync/mpsc/index.html",
    },

    // ─── ARQUIVOS E ENTRADA/SAÍDA ──────────────────────────────────────────────
    RustLibItem {
        id: "std_fs",
        name: "std::fs",
        category: LibCategory::FsIo,
        summary: "Operações completas no sistema de arquivos: ler, gravar, criar diretórios, renomear e apagar.",
        description: "\
O módulo `std::fs` fornece a API nativa e segura para interagir com arquivos no disco.
Inclui atalhos convenientes como `fs::read_to_string` e `fs::write`, bem como a struct `File` com `OpenOptions` para controle granular de permissões, append e criação exclusiva.",
        key_types: &["File", "OpenOptions", "DirEntry", "Metadata"],
        key_methods: &[
            "fs::read_to_string(path) -> Result<String>",
            "fs::write(path, data) -> Result<()>",
            "fs::create_dir_all(path) -> Result<()>",
            "fs::read_dir(path) -> Result<ReadDir>",
        ],
        example_code: r#"use std::fs;

fn main() -> std::io::Result<()> {
    let demo_dir = std::env::temp_dir().join("rrs_lib_demo");
    fs::create_dir_all(&demo_dir)?;

    let arquivo = demo_dir.join("exemplo.txt");
    fs::write(&arquivo, "RRS - Riva Rust Store: Alto Desempenho e Segurança!")?;

    let texto = fs::read_to_string(&arquivo)?;
    println!("Conteúdo gravado e lido do arquivo:\n\"{}\"", texto);

    // Limpeza
    fs::remove_file(&arquivo)?;
    fs::remove_dir(&demo_dir)?;
    println!("\nArquivos de demonstração removidos com sucesso.");
    Ok(())
}
"#,
        is_std: true,
        cargo_dep: None,
        docs_url: "https://doc.rust-lang.org/std/fs/index.html",
    },
    RustLibItem {
        id: "std_io",
        name: "std::io",
        category: LibCategory::FsIo,
        summary: "Abstrações fundamentais de I/O: traits Read e Write, buffers otimizados e streams do sistema.",
        description: "\
O módulo `std::io` define as traits centrais `Read`, `Write` e `Seek`.
`BufReader` e `BufWriter` adicionam buffers na memória RAM (geralmente de 8 KB), reduzindo centenas de chamadas lentas ao sistema operacional (syscalls) para uma única leitura/escrita em bloco.",
        key_types: &["BufReader<R>", "BufWriter<W>", "Read", "Write", "Seek"],
        key_methods: &[
            "reader.lines()",
            "reader.read_to_end(&mut buf)",
            "writer.write_all(bytes)",
            "writer.flush()",
        ],
        example_code: r#"use std::io::{self, BufRead, BufReader, Write};

fn main() -> io::Result<()> {
    let dados_em_memoria = b"Linha 1: Rust e moderna\nLinha 2: Sem vazamentos de memoria\nLinha 3: 100% nativa";
    let reader = BufReader::new(&dados_em_memoria[..]);

    println!("Lendo linhas com BufReader:");
    for (num, linha) in reader.lines().enumerate() {
        println!("  [{}] {}", num + 1, linha?);
    }

    let mut saida = Vec::new();
    saida.write_all(b"Saida gerada via trait Write.")?;
    println!("\nBytes gravados no buffer: {}", saida.len());
    Ok(())
}
"#,
        is_std: true,
        cargo_dep: None,
        docs_url: "https://doc.rust-lang.org/std/io/index.html",
    },

    // ─── REDES E SOCKETS ───────────────────────────────────────────────────────
    RustLibItem {
        id: "std_net",
        name: "std::net",
        category: LibCategory::Net,
        summary: "Comunicação de rede TCP e UDP síncrona com controle total de conexões e endereçamento.",
        description: "\
Fornece conectividade nativa de baixo nível via sockets:
`TcpListener` para servidores que aceitam conexões de entrada;
`TcpStream` para clientes TCP confiáveis com troca de bytes;
`UdpSocket` para envio rápido de datagramas sem confirmação (muito usado em jogos e streaming).",
        key_types: &["TcpListener", "TcpStream", "UdpSocket", "SocketAddr", "IpAddr"],
        key_methods: &[
            "TcpListener::bind(addr)",
            "listener.accept()",
            "TcpStream::connect(addr)",
            "socket.send_to(buf, addr)",
        ],
        example_code: r#"use std::net::SocketAddr;

fn main() {
    let endereco: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    println!("IP configurado: {}", endereco.ip());
    println!("Porta configurada: {}", endereco.port());
    println!("É loopback (local)? {}", endereco.ip().is_loopback());

    println!("\nPara abrir um servidor TCP real:");
    println!("  let listener = std::net::TcpListener::bind(\"127.0.0.1:8080\")?;");
    println!("  for stream in listener.incoming() {{ ... }}");
}
"#,
        is_std: true,
        cargo_dep: None,
        docs_url: "https://doc.rust-lang.org/std/net/index.html",
    },

    // ─── PROCESSOS E SISTEMA OPERACIONAL ───────────────────────────────────────
    RustLibItem {
        id: "std_process",
        name: "std::process::Command",
        category: LibCategory::SystemOs,
        summary: "Execução segura de programas e utilitários do sistema com controle de argumentos, pipes e saídas.",
        description: "\
O builder `Command` cria e gerencia subprocessos do sistema operacional.
Permite redirecionar entrada/saída (`Stdio::piped()`), configurar variáveis de ambiente e diretórios de execução, evitando as falhas clássicas de injeção de comando encontradas em linguagens com chamadas via shell.",
        key_types: &["Command", "Output", "Child", "Stdio", "ExitStatus"],
        key_methods: &[
            "Command::new(program)",
            ".arg(arg)",
            ".args(slice)",
            ".output() -> Result<Output>",
            ".status() -> Result<ExitStatus>",
            ".spawn() -> Result<Child>",
        ],
        example_code: r#"use std::process::Command;

fn main() {
    // Executa rustc --version diretamente sem passar por interpretador shell
    let output = Command::new("rustc")
        .arg("--version")
        .output();

    match output {
        Ok(out) => {
            let texto = String::from_utf8_lossy(&out.stdout);
            println!("Compilador Rust instalado no seu PC:\n{}", texto.trim());
        }
        Err(e) => {
            println!("Não foi possível chamar o rustc diretamente: {}", e);
        }
    }
}
"#,
        is_std: true,
        cargo_dep: None,
        docs_url: "https://doc.rust-lang.org/std/process/struct.Command.html",
    },
    RustLibItem {
        id: "std_env",
        name: "std::env",
        category: LibCategory::SystemOs,
        summary: "Inspeção do ambiente de execução: variáveis de ambiente, argumentos CLI e diretório atual.",
        description: "\
Fornece funções para consultar argumentos passados na linha de comando (`env::args()`), ler e definir variáveis de ambiente (`env::var()`), consultar a pasta de arquivos temporários (`env::temp_dir()`) e o executável atual (`env::current_exe()`).",
        key_types: &["Args", "Vars", "VarError"],
        key_methods: &[
            "env::args() -> Args",
            "env::var(key) -> Result<String, VarError>",
            "env::current_dir() -> Result<PathBuf>",
            "env::temp_dir() -> PathBuf",
        ],
        example_code: r#"use std::env;

fn main() {
    println!("Diretório de trabalho atual: {:?}", env::current_dir().unwrap_or_default());
    println!("Pasta temporária do sistema: {:?}", env::temp_dir());

    if let Ok(user) = env::var("USERNAME") {
        println!("Usuário ativo no Windows: {}", user);
    }

    println!("\nArgumentos passados ao programa (total {}):", env::args().len());
    for (i, arg) in env::args().enumerate() {
        println!("  [{}] {}", i, arg);
    }
}
"#,
        is_std: true,
        cargo_dep: None,
        docs_url: "https://doc.rust-lang.org/std/env/index.html",
    },

    // ─── MEMÓRIA, TIPOS E UTILITÁRIOS ──────────────────────────────────────────
    RustLibItem {
        id: "std_path",
        name: "std::path::{Path, PathBuf}",
        category: LibCategory::MemoryTypes,
        summary: "Manipulação agnóstica de caminhos de arquivos e pastas no Windows e Linux.",
        description: "\
`Path` (fatia emprestada) e `PathBuf` (caminho próprio alocado na Heap) representam caminhos de arquivos de forma segura, tratando automaticamente barras normais (`/`) e invertidas (`\\`), extensões, prefixos de drive e caminhos relativos/absolutos.",
        key_types: &["Path", "PathBuf", "Component"],
        key_methods: &[
            "path.join(other)",
            "path.parent()",
            "path.file_name()",
            "path.extension()",
            "path.is_file()",
            "path.is_dir()",
        ],
        example_code: r#"use std::path::PathBuf;

fn main() {
    let mut caminho = PathBuf::from("C:\\Jogos");
    caminho.push("Steam");
    caminho.push("steamapps");
    caminho.push("appmanifest_105600.acf");

    println!("Caminho completo: {}", caminho.display());
    println!("Nome do arquivo: {:?}", caminho.file_name());
    println!("Extensão: {:?}", caminho.extension());
    println!("Pasta pai: {:?}", caminho.parent());
}
"#,
        is_std: true,
        cargo_dep: None,
        docs_url: "https://doc.rust-lang.org/std/path/index.html",
    },
    RustLibItem {
        id: "std_time",
        name: "std::time::{Instant, Duration}",
        category: LibCategory::MemoryTypes,
        summary: "Medição de tempo monótono de alta precisão (benchmarks e profiling) e representação de intervalos.",
        description: "\
`Instant` mede o tempo transcorrido a partir de um relógio monótono garantido pelo processador que nunca volta no tempo (imune a ajustes de fuso horário ou sincronização NTP).
Ideal para benchmarks de velocidade, cálculo de taxas de quadros (FPS) e timeouts de rede.",
        key_types: &["Instant", "Duration", "SystemTime"],
        key_methods: &[
            "Instant::now()",
            "start.elapsed() -> Duration",
            "Duration::from_millis(ms)",
            "Duration::as_secs_f64()",
        ],
        example_code: r#"use std::time::{Duration, Instant};

fn main() {
    let inicio = Instant::now();

    // Simula uma computação matemática
    let mut soma: u64 = 0;
    for i in 1..=1_000_000 {
        soma = soma.wrapping_add(i);
    }

    let decorrido = inicio.elapsed();
    println!("Soma acumulada: {}", soma);
    println!("Tempo de execução: {:?} ({:.3} ms)", decorrido, decorrido.as_secs_f64() * 1000.0);
}
"#,
        is_std: true,
        cargo_dep: None,
        docs_url: "https://doc.rust-lang.org/std/time/index.html",
    },

    // ─── TOP CRATES DO ECOSSISTEMA ─────────────────────────────────────────────
    RustLibItem {
        id: "crate_serde",
        name: "serde & serde_json",
        category: LibCategory::Ecosystem,
        summary: "O padrão da indústria para serialização e desserialização ultra veloz de dados com custo zero de abstração.",
        description: "\
`serde` permite transformar structs e enums de Rust em JSON, YAML, TOML, Bincode e MessagePack com simples anotações `#[derive(Serialize, Deserialize)]`.
Utiliza especialização do compilador para gerar código de parsing direto em linguagem de máquina sem uso de reflection em runtime.",
        key_types: &["Serialize", "Deserialize", "serde_json::Value"],
        key_methods: &[
            "serde_json::to_string(&data)",
            "serde_json::to_string_pretty(&data)",
            "serde_json::from_str::<T>(json_str)",
        ],
        example_code: r##"// Em projetos reais, adicione no Cargo.toml:
// serde = { version = "1.0", features = ["derive"] }
// serde_json = "1.0"

fn main() {
    let json_exemplo = r#"{
        "titulo": "Super Rust Bros",
        "estrelas": 5420,
        "plataformas": ["windows", "linux"]
    }"#;

    println!("JSON simulado recebido da API:\n{}", json_exemplo);
    println!("\nCom o serde, o código transforma isso direto em struct tipada com:");
    println!("  #[derive(serde::Deserialize)]");
    println!("  struct Jogo {{ titulo: String, estrelas: u32, plataformas: Vec<String> }}");
}
"##,
        is_std: false,
        cargo_dep: Some("serde = { version = \"1.0\", features = [\"derive\"] }\nserde_json = \"1.0\""),
        docs_url: "https://docs.rs/serde/latest/serde/",
    },
    RustLibItem {
        id: "crate_tokio",
        name: "tokio",
        category: LibCategory::Ecosystem,
        summary: "Runtime assíncrono para Rust com event loop multithreaded de altíssimo desempenho, timers e canais.",
        description: "\
O Tokio é o alicerce do ecossistema assíncrono em Rust.
Fornece o executor de tarefas assíncronas com work-stealing, I/O assíncrono não-bloqueante para sockets e arquivos, primitivas de sincronização (`tokio::sync::Mutex`) e timers assíncronos (`tokio::time::sleep`).",
        key_types: &["#[tokio::main]", "tokio::spawn", "tokio::sync::mpsc"],
        key_methods: &[
            "tokio::spawn(async move { ... })",
            "tokio::time::sleep(duration).await",
            "tokio::select! { ... }",
        ],
        example_code: r#"// Adicione no Cargo.toml:
// tokio = { version = "1.0", features = ["full"] }

fn main() {
    println!("Tokio é o motor assíncrono da RRS!");
    println!("Permite executar downloads simultâneos e consultas de rede em paralelo sem travar a interface.");
    println!("\nEstrutura típica de um app Tokio:");
    println!("  #[tokio::main]");
    println!("  async fn main() {{");
    println!("      let res = tokio::spawn(async {{ \"tarefa em background\" }}).await;");
    println!("  }}");
}
"#,
        is_std: false,
        cargo_dep: Some("tokio = { version = \"1\", features = [\"full\"] }"),
        docs_url: "https://docs.rs/tokio/latest/tokio/",
    },
    RustLibItem {
        id: "crate_reqwest",
        name: "reqwest",
        category: LibCategory::Ecosystem,
        summary: "Cliente HTTP e HTTPS ergonômico, seguro e moderno com suporte a JSON, streaming e TLS nativo.",
        description: "\
O `reqwest` simplifica requisições para APIs REST, downloads de arquivos e chamadas web com suporte a timeouts automáticos, pools de conexão reutilizáveis, cabeçalhos personalizados e proxies.",
        key_types: &["Client", "Response", "RequestBuilder"],
        key_methods: &[
            "Client::new()",
            "client.get(url).send().await",
            "response.json::<T>().await",
            "response.bytes().await",
        ],
        example_code: r#"// Adicione no Cargo.toml:
// reqwest = { version = "0.12", features = ["json"] }

fn main() {
    println!("O crate reqwest é usado pela RRS para:");
    println!("  • Consultar a API do GitHub Releases");
    println!("  • Baixar instaladores verificados com hash SHA-256");
    println!("  • Checar relatórios do VirusTotal");
    println!("\nExemplo de chamada assíncrona:");
    println!("  let body = reqwest::get(\"https://api.github.com\").await?.text().await?;");
}
"#,
        is_std: false,
        cargo_dep: Some("reqwest = { version = \"0.12\", features = [\"json\"] }"),
        docs_url: "https://docs.rs/reqwest/latest/reqwest/",
    },
    RustLibItem {
        id: "crate_anyhow",
        name: "anyhow & thiserror",
        category: LibCategory::Ecosystem,
        summary: "Gerenciamento idiomático de erros: `anyhow` para aplicações e `thiserror` para bibliotecas.",
        description: "\
O `anyhow::Result<T>` permite retornar qualquer erro que implemente `std::error::Error` usando o operador `?`.
Permite adicionar contexto com `.with_context(|| \"mensagem explicativa\")`, facilitando o rastreio da causa raiz de falhas em disco, rede ou parsing.",
        key_types: &["anyhow::Result<T>", "anyhow::Context", "anyhow::bail!"],
        key_methods: &[
            "result.context(\"falha ao carregar\")",
            "anyhow::bail!(\"motivo do erro\")",
        ],
        example_code: r#"// Adicione no Cargo.toml:
// anyhow = "1.0"

use std::fs;

fn ler_config() -> Result<String, String> {
    // Simulação de tratamento seguro
    fs::read_to_string("config_inexistente.json")
        .map_err(|e| format!("Falha de I/O amigável: {}", e))
}

fn main() {
    match ler_config() {
        Ok(c) => println!("Config: {}", c),
        Err(e) => println!("Tratamento de erro capturado com sucesso: {}", e),
    }
}
"#,
        is_std: false,
        cargo_dep: Some("anyhow = \"1.0\""),
        docs_url: "https://docs.rs/anyhow/latest/anyhow/",
    },
    RustLibItem {
        id: "crate_clap",
        name: "clap",
        category: LibCategory::Ecosystem,
        summary: "Construtor de interfaces de linha de comando (CLI) profissionais com parsing automático de flags e help.",
        description: "\
O `clap` gera menus de ajuda com cores, subcomandos, validação de argumentos e autocompletar no terminal através de structs declarativas com `#[derive(Parser)]`.",
        key_types: &["Parser", "Subcommand", "Args"],
        key_methods: &[
            "Cli::parse()",
            "#[arg(short, long)]",
            "#[command(subcommand)]",
        ],
        example_code: r#"// Adicione no Cargo.toml:
// clap = { version = "4", features = ["derive"] }

fn main() {
    println!("A própria RRS utiliza o `clap` para sua interface CLI!");
    println!("Permite que o usuário execute comandos como:");
    println!("  rrs search \"emulador\"");
    println!("  rrs install --silent");
    println!("  rrs translate main.c");
}
"#,
        is_std: false,
        cargo_dep: Some("clap = { version = \"4\", features = [\"derive\"] }"),
        docs_url: "https://docs.rs/clap/latest/clap/",
    },
    RustLibItem {
        id: "crate_rayon",
        name: "rayon",
        category: LibCategory::Ecosystem,
        summary: "Paralelismo de dados para coleções e iteradores com divisão automática de carga em threads do processador.",
        description: "\
Com o `rayon`, basta trocar `.iter()` por `.par_iter()` para que qualquer laço sobre vetores passe a ser computado em paralelo em todos os núcleos físicos da sua CPU automaticamente com work-stealing!",
        key_types: &["ParallelIterator", "IntoParallelRefIterator"],
        key_methods: &[
            "vec.par_iter().map(...).collect()",
            "vec.par_sort()",
        ],
        example_code: r#"// Adicione no Cargo.toml:
// rayon = "1.10"

fn main() {
    println!("Rayon: Paralelismo de dados em 1 linha de código!");
    println!("Exemplo conceitual:");
    println!("  use rayon::prelude::*;");
    println!("  let resultados: Vec<_> = dados.par_iter().map(|item| processar(item)).collect();");
    println!("Distribui a carga automaticamente em todos os núcleos da CPU.");
}
"#,
        is_std: false,
        cargo_dep: Some("rayon = \"1.10\""),
        docs_url: "https://docs.rs/rayon/latest/rayon/",
    },
    RustLibItem {
        id: "crate_rand",
        name: "rand",
        category: LibCategory::Ecosystem,
        summary: "Geração de números pseudo-aleatórios rápidos e criptograficamente seguros.",
        description: "\
O crate `rand` fornece geradores de aleatoriedade, sorteio de elementos em vetores (`.choose()`), embaralhamento (`.shuffle()`) e distribuições estatísticas.",
        key_types: &["Rng", "thread_rng", "random"],
        key_methods: &[
            "rng.gen_range(min..max)",
            "slice.choose(&mut rng)",
            "slice.shuffle(&mut rng)",
        ],
        example_code: r#"// Adicione no Cargo.toml:
// rand = "0.8"

fn main() {
    println!("O crate `rand` permite sortear números e probabilidades:");
    println!("  let mut rng = rand::thread_rng();");
    println!("  let dado = rng.gen_range(1..=6);");
    println!("  let vencedor = jogadores.choose(&mut rng);");
}
"#,
        is_std: false,
        cargo_dep: Some("rand = \"0.8\""),
        docs_url: "https://docs.rs/rand/latest/rand/",
    },

    // ─── COLEÇÕES ADICIONAIS ──────────────────────────────────────────────────
    RustLibItem {
        id: "std_hashset",
        name: "std::collections::HashSet",
        category: LibCategory::Collections,
        summary: "Conjunto de elementos únicos na memória com testes de pertinência O(1) e operações de união e interseção.",
        description: "\
O `HashSet<T>` implementa um conjunto matemático sem repetições baseado em tabela hash.
Ideal para deduplicação instantânea de dados, conferência de presença e testes rápidos de pertinência (`.contains()`).",
        key_types: &["HashSet<T>"],
        key_methods: &[
            ".insert(val) -> bool",
            ".contains(&val) -> bool",
            ".intersection(&other)",
            ".union(&other)",
            ".difference(&other)",
        ],
        example_code: r#"use std::collections::HashSet;

fn main() {
    let mut tags = HashSet::new();
    tags.insert("rust");
    tags.insert("segurança");
    tags.insert("concorrência");
    tags.insert("rust"); // duplicata ignorada

    println!("Total de tags únicas: {}", tags.len());
    println!("Contém 'rust'? {}", tags.contains("rust"));
}
"#,
        is_std: true,
        cargo_dep: None,
        docs_url: "https://doc.rust-lang.org/std/collections/struct.HashSet.html",
    },
    RustLibItem {
        id: "std_btreeset",
        name: "std::collections::BTreeSet",
        category: LibCategory::Collections,
        summary: "Conjunto ordenado baseado em B-Tree com iteração crescente garantida e busca por intervalos.",
        description: "\
O `BTreeSet<T>` armazena elementos únicos mantendo-os sempre ordenados através de uma árvore B.
Permite consultas por intervalo (`.range()`) e recuperar o menor ou maior elemento com facilidade.",
        key_types: &["BTreeSet<T>"],
        key_methods: &[
            ".insert(val)",
            ".range(min..=max)",
            ".first() -> Option<&T>",
            ".last() -> Option<&T>",
        ],
        example_code: r#"use std::collections::BTreeSet;

fn main() {
    let mut notas = BTreeSet::new();
    notas.insert(85);
    notas.insert(92);
    notas.insert(70);
    notas.insert(100);

    println!("Notas ordenadas: {:?}", notas);
    println!("Maior nota: {:?}", notas.last());
}
"#,
        is_std: true,
        cargo_dep: None,
        docs_url: "https://doc.rust-lang.org/std/collections/struct.BTreeSet.html",
    },

    // ─── CONCORRÊNCIA E ATÔMICOS ───────────────────────────────────────────────
    RustLibItem {
        id: "std_atomic",
        name: "std::sync::atomic::{AtomicBool, AtomicUsize}",
        category: LibCategory::SyncConcurrency,
        summary: "Primitivas atômicas lock-free com instruções de hardware para contadores e flags seguras entre threads.",
        description: "\
Tipos atômicos fornecem sincronização livre de travas (lock-free) através de instruções diretas da CPU (como CMPXCHG).
Ideais para contadores globais de estatísticas, flags de encerramento (`AtomicBool`) e métricas sem o overhead de Mutex.",
        key_types: &["AtomicBool", "AtomicUsize", "Ordering"],
        key_methods: &[
            ".load(Ordering)",
            ".store(val, Ordering)",
            ".fetch_add(val, Ordering)",
            ".compare_exchange(..)",
        ],
        example_code: r#"use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    let contador = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..5 {
        let c = Arc::clone(&contador);
        handles.push(thread::spawn(move || {
            c.fetch_add(10, Ordering::SeqCst);
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    println!("Total atômico acumulado: {}", contador.load(Ordering::SeqCst));
}
"#,
        is_std: true,
        cargo_dep: None,
        docs_url: "https://doc.rust-lang.org/std/sync/atomic/index.html",
    },
    RustLibItem {
        id: "std_once_lock",
        name: "std::sync::OnceLock",
        category: LibCategory::SyncConcurrency,
        summary: "Inicialização preguiçosa (lazy) segura e thread-safe de valores estáticos com custo zero após inicializado.",
        description: "\
`OnceLock<T>` (estabilizado em Rust 1.70+) é a solução nativa da linguagem para inicialização preguiçosa e variáveis globais seguras.
Garante que o valor seja computado exatamente uma única vez, mesmo sob concorrência intensa de múltiplas threads.",
        key_types: &["OnceLock<T>"],
        key_methods: &[
            ".get() -> Option<&T>",
            ".get_or_init(|| ...)",
            ".set(val) -> Result<()>",
        ],
        example_code: r#"use std::sync::OnceLock;

static CONFIG: OnceLock<String> = OnceLock::new();

fn get_config() -> &'static str {
    CONFIG.get_or_init(|| {
        println!("Inicializando configuração global (apenas 1x)...");
        "Modo Produção v1.0".to_string()
    })
}

fn main() {
    println!("Chamada 1: {}", get_config());
    println!("Chamada 2: {}", get_config());
}
"#,
        is_std: true,
        cargo_dep: None,
        docs_url: "https://doc.rust-lang.org/std/sync/struct.OnceLock.html",
    },

    // ─── MEMÓRIA E MUTABILIDADE INTERIOR ──────────────────────────────────────
    RustLibItem {
        id: "std_cell",
        name: "std::cell::{RefCell, Cell}",
        category: LibCategory::MemoryTypes,
        summary: "Mutabilidade interior em thread única: confere regras de empréstimo em tempo de execução ao invés de compilação.",
        description: "\
`RefCell<T>` permite mutar dados mesmo quando a referência externa é imutável (`&self`).
As regras de empréstimo de Rust (apenas 1 mutável ou N imutáveis) são conferidas em tempo de execução (`.borrow()` e `.borrow_mut()`).",
        key_types: &["RefCell<T>", "Cell<T>"],
        key_methods: &[
            ".borrow() -> Ref<T>",
            ".borrow_mut() -> RefMut<T>",
            ".get() (para Cell)",
            ".set(val) (para Cell)",
        ],
        example_code: r#"use std::cell::RefCell;

struct Logger {
    mensagens: RefCell<Vec<String>>,
}

impl Logger {
    fn registrar(&self, msg: &str) {
        // Muta o vetor interno mesmo com &self imutável
        self.mensagens.borrow_mut().push(msg.to_string());
    }
}

fn main() {
    let log = Logger { mensagens: RefCell::new(vec![]) };
    log.registrar("Iniciando app");
    log.registrar("Conexão OK");

    println!("Total de logs gravados: {}", log.mensagens.borrow().len());
}
"#,
        is_std: true,
        cargo_dep: None,
        docs_url: "https://doc.rust-lang.org/std/cell/index.html",
    },
    RustLibItem {
        id: "std_mem",
        name: "std::mem::{size_of, replace, take}",
        category: LibCategory::MemoryTypes,
        summary: "Manipulação avançada de layout de memória, tamanhos de structs e troca de valores sem cópia.",
        description: "\
O módulo `std::mem` oferece utilitários fundamentais para inspecionar memória em baixo nível (`size_of`, `align_of`)
e transferir posse de valores dentro de referências mutáveis (`replace`, `take`, `swap`) de forma 100% segura.",
        key_types: &["std::mem"],
        key_methods: &[
            "mem::size_of::<T>() -> usize",
            "mem::replace(&mut dest, src)",
            "mem::take(&mut dest)",
            "mem::drop(val)",
        ],
        example_code: r#"use std::mem;

struct Buffer {
    dados: Vec<u8>,
}

fn main() {
    let mut buf = Buffer { dados: vec![1, 2, 3, 4, 5] };
    
    // mem::take substitui o conteúdo por Vec::default() sem clonar
    let extraidos = mem::take(&mut buf.dados);

    println!("Dados extraídos: {:?}", extraidos);
    println!("Buffer original resetado: {:?}", buf.dados);
    println!("Tamanho de Buffer na stack: {} bytes", mem::size_of::<Buffer>());
}
"#,
        is_std: true,
        cargo_dep: None,
        docs_url: "https://doc.rust-lang.org/std/mem/index.html",
    },

    // ─── REDES E SERVIDORES ───────────────────────────────────────────────────
    RustLibItem {
        id: "std_net_tcplistener",
        name: "std::net::TcpListener",
        category: LibCategory::Net,
        summary: "Servidor de sockets TCP síncrono nativo para escuta de portas de rede e aceitação de conexões.",
        description: "\
`TcpListener` permite vincular uma porta local (como `127.0.0.1:8080`) e aceitar fluxos de clientes TCP (`TcpStream`).
É a base nativa sobre a qual servidores HTTP, proxies e protocolos de rede são construídos em Rust.",
        key_types: &["TcpListener", "Incoming"],
        key_methods: &[
            "TcpListener::bind(addr)",
            ".incoming() -> impl Iterator<Item = Result<TcpStream>>",
            ".accept() -> Result<(TcpStream, SocketAddr)>",
        ],
        example_code: r#"use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Falha ao abrir socket TCP");
    let porta = listener.local_addr().unwrap().port();
    println!("Servidor TCP ouvindo com sucesso na porta efêmera: {}", porta);
}
"#,
        is_std: true,
        cargo_dep: None,
        docs_url: "https://doc.rust-lang.org/std/net/struct.TcpListener.html",
    },
    RustLibItem {
        id: "std_net_udp",
        name: "std::net::UdpSocket",
        category: LibCategory::Net,
        summary: "Comunicação rápida de datagramas UDP sem conexão para jogos multiplayer, DNS e streaming.",
        description: "\
O protocolo UDP é sem conexão e sem garantia de entrega, proporcionando latência ultrabaixa.
O `UdpSocket` nativo de Rust é perfeito para protocolos de jogos multiplayer, telemetria de IoT e resolução DNS.",
        key_types: &["UdpSocket"],
        key_methods: &[
            "UdpSocket::bind(addr)",
            ".send_to(&buf, dest)",
            ".recv_from(&mut buf)",
        ],
        example_code: r#"use std::net::UdpSocket;

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:0").expect("Falha ao abrir socket UDP");
    let porta = socket.local_addr().unwrap().port();
    println!("Socket UDP pronto para datagramas na porta: {}", porta);
}
"#,
        is_std: true,
        cargo_dep: None,
        docs_url: "https://doc.rust-lang.org/std/net/struct.UdpSocket.html",
    },

    // ─── FRAMEWORKS WEB E BANCOS DE DADOS ────────────────────────────────────
    RustLibItem {
        id: "crate_axum",
        name: "axum",
        category: LibCategory::Net,
        summary: "Framework web moderno e ergonômico construído pela equipe do Tokio com extração tipada de requisições.",
        description: "\
Axum é o framework web preferido no ecossistema Rust atual.
Desenvolvido sobre Tokio e Tower, ele oferece roteamento rápido, middlewares modulares e extração declarativa de parâmetros via tipos (`Json<T>`, `Path<T>`, `Query<T>`).",
        key_types: &["Router", "Json", "Path", "response::IntoResponse"],
        key_methods: &[
            "Router::new().route(path, get(handler))",
            "serve(listener, router)",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// axum = "0.7"
// tokio = { version = "1.0", features = ["full"] }

fn main() {
    println!("Axum: rotas declarativas de altíssima performance para APIs REST.");
    println!("Exemplo conceitual de rota:");
    println!("  async fn saudar() -> &'static str {{ "Olá do Axum Web Framework!" }}");
    println!("  let app = axum::Router::new().route("/", axum::routing::get(saudar));");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"axum = "0.7"
tokio = { version = "1.0", features = ["full"] }"#),
        docs_url: "https://docs.rs/axum/latest/axum/",
    },
    RustLibItem {
        id: "crate_sqlx",
        name: "sqlx",
        category: LibCategory::Database,
        summary: "Driver SQL 100% Rust assíncrono com conferência de sintaxe e tipos das queries em tempo de compilação.",
        description: "\
SQLx permite escrever SQL puro com checagem em tempo de compilação sem necessidade de ORM pesado.
Garante em tempo de build que suas tabelas, colunas e tipos SQL batem exatamente com as structs de Rust através da macro `query_as!`. Suporta PostgreSQL, MySQL e SQLite.",
        key_types: &["PgPool", "SqlitePool", "query!", "query_as!"],
        key_methods: &[
            "SqlitePool::connect(&url).await",
            r#"query_as!(Struct, "SELECT id FROM tabela").fetch_all(&pool).await"#,
        ],
        example_code: r##"// Adicione no Cargo.toml:
// sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }

fn main() {
    println!("SQLx: queries SQL tipadas e seguras sem erros de digitação em runtime.");
    println!("Exemplo de uso:");
    println!("  let pool = SqlitePool::connect("sqlite:banco.db").await?;");
    println!("  let rows = sqlx::query!("SELECT id, nome FROM usuarios").fetch_all(&pool).await?;");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }"#),
        docs_url: "https://docs.rs/sqlx/latest/sqlx/",
    },
    RustLibItem {
        id: "crate_rusqlite",
        name: "rusqlite",
        category: LibCategory::Database,
        summary: "Bindings ergonômicos e seguros para SQLite embutido, padrão da indústria para aplicações desktop e locais.",
        description: "\
`rusqlite` é a biblioteca padrão para utilizar SQLite localmente em aplicações Rust (inclusive é a utilizada na própria RRS).
Garante transações ACID, preparação de statements cacheados e mapeamento flexível de colunas.",
        key_types: &["Connection", "Statement", "params!"],
        key_methods: &[
            "Connection::open(path)",
            ".execute(sql, params)",
            ".query_row(sql, params, |row| ..)",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// rusqlite = "0.32"

fn main() {
    println!("rusqlite: banco de dados relacional embarcado ultrarrápido sem servidor.");
    println!("  let conn = rusqlite::Connection::open("dados.db")?;");
    println!("  conn.execute("CREATE TABLE secao (id INTEGER, valor TEXT)", [])?;");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"rusqlite = "0.32""#),
        docs_url: "https://docs.rs/rusqlite/latest/rusqlite/",
    },

    // ─── DESENVOLVIMENTO DE JOGOS E GRÁFICOS ─────────────────────────────────
    RustLibItem {
        id: "crate_bevy",
        name: "bevy",
        category: LibCategory::GameDev,
        summary: "Game engine moderna e revolucionária em Rust baseada em Entity-Component-System (ECS) e renderização paralela.",
        description: "\
Bevy é a engine de jogos mais popular do ecossistema Rust.
Utiliza arquitetura puramente orientada a dados (ECS) com paralelização automática na CPU e pipeline moderno de renderização em 2D e 3D.",
        key_types: &["App", "Commands", "Query", "Res", "Component", "Resource"],
        key_methods: &[
            "App::new().add_plugins(DefaultPlugins).run()",
            "commands.spawn((ComponentA, ComponentB))",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// bevy = "0.14"

fn main() {
    println!("Bevy: motor de jogos moderno e ergonômico em Rust com ECS.");
    println!("Estrutura típica de um jogo Bevy:");
    println!("  fn movimentar_jogador(mut query: Query<&mut Transform, With<Player>>) {{ .. }}");
    println!("  App::new().add_plugins(DefaultPlugins).add_systems(Update, movimentar_jogador).run();");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"bevy = "0.14""#),
        docs_url: "https://docs.rs/bevy/latest/bevy/",
    },
    RustLibItem {
        id: "crate_wgpu",
        name: "wgpu",
        category: LibCategory::GameDev,
        summary: "API gráfica portável de altíssima performance para Vulkan, DirectX 12, Metal e WebGPU nativo.",
        description: "\
`wgpu` é a implementação em Rust da especificação WebGPU padrão da W3C.
Permite programar pipelines gráficos modernos de shaders e computação (GPGPU) compatíveis com Windows, Linux, macOS e Web sem alterar o código-fonte.",
        key_types: &["Device", "Queue", "RenderPipeline", "ShaderModule"],
        key_methods: &[
            "Instance::new(Backends::all())",
            "device.create_render_pipeline(&desc)",
            "queue.submit([command_buffer])",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// wgpu = "22.0"

fn main() {
    println!("wgpu: gráficos portáteis de última geração (DirectX 12, Vulkan, Metal, WebGPU).");
    println!("Utilizado em engines de jogos, emuladores e interfaces desktop de alto desempenho.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"wgpu = "22.0""#),
        docs_url: "https://docs.rs/wgpu/latest/wgpu/",
    },

    // ─── CRIPTOGRAFIA, REGEX E UTILITÁRIOS ────────────────────────────────────
    RustLibItem {
        id: "crate_regex",
        name: "regex",
        category: LibCategory::Ecosystem,
        summary: "Expressões regulares com tempo linear O(n) e garantia matemática contra ataques de ReDoS.",
        description: "\
O crate oficial `regex` compila expressões regulares em autômatos finitos (DFA/NFA).
Diferente de engines com backtracking (como PCRE ou Python), garante que o tempo de matching cresce estritamente proporcional ao tamanho do texto.",
        key_types: &["Regex", "Captures", "RegexBuilder"],
        key_methods: &[
            "Regex::new(pattern)",
            ".is_match(text) -> bool",
            ".captures(text) -> Option<Captures>",
            ".replace_all(text, rep)",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// regex = "1.10"

fn main() {
    let email_exemplo = "suporte@rust-lang.org";
    println!("Validação por expressão regular:");
    println!("Email a ser checado: {}", email_exemplo);
    println!("Expressão típica: ^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"regex = "1.10""#),
        docs_url: "https://docs.rs/regex/latest/regex/",
    },
    RustLibItem {
        id: "crate_sha2",
        name: "sha2",
        category: LibCategory::Ecosystem,
        summary: "Funções de hash criptográfico seguro da família SHA-2 (SHA-256 e SHA-512) com aceleração de hardware.",
        description: "\
O crate `sha2` implementa hashing criptográfico com suporte nativo a instruções de CPU (Intel SHA Extensions / ARMv8 Crypto).
É o padrão utilizado em conferência de integridade binária, autenticação de releases e blockchains.",
        key_types: &["Sha256", "Digest"],
        key_methods: &[
            "Sha256::new()",
            ".update(data)",
            ".finalize()",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// sha2 = "0.10"

fn main() {
    println!("SHA-256: hashing criptográfico seguro para conferência de instaladores e arquivos.");
    println!("Uso com a trait Digest:");
    println!("  let mut hasher = sha2::Sha256::new();");
    println!("  hasher.update(b"meu arquivo binario");");
    println!("  let hash = format!("{:x}", hasher.finalize());");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"sha2 = "0.10""#),
        docs_url: "https://docs.rs/sha2/latest/sha2/",
    },
    RustLibItem {
        id: "crate_indicatif",
        name: "indicatif",
        category: LibCategory::Ecosystem,
        summary: "Barras de progresso e spinners animados para utilitários de linha de comando com estimativas de tempo.",
        description: "\
`indicatif` fornece barras de progresso visualmente elegantes no terminal com formatação de bytes (`12.5 MB / 100 MB`), ETA e contadores paralelos.",
        key_types: &["ProgressBar", "ProgressStyle", "MultiProgress"],
        key_methods: &[
            "ProgressBar::new(total)",
            ".set_style(style)",
            ".inc(delta)",
            ".finish_with_message(msg)",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// indicatif = "0.17"

fn main() {
    println!("indicatif: feedback visual com barras de progresso para ferramentas CLI.");
    println!("  let bar = indicatif::ProgressBar::new(100);");
    println!("  bar.inc(25);");
    println!("  bar.finish_with_message("Download concluído!");");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"indicatif = "0.17""#),
        docs_url: "https://docs.rs/indicatif/latest/indicatif/",
    },
    RustLibItem {
        id: "crate_colored",
        name: "colored",
        category: LibCategory::Ecosystem,
        summary: "Colorização e estilização de texto no terminal com sintaxe fluente e suporte a Windows 10/11 ANSI.",
        description: "\
Permite adicionar cores, negrito, sublinhado e fundos ao terminal com simples chamadas como `'Texto'.green().bold()`.",
        key_types: &["Colorize"],
        key_methods: &[
            ".red()",
            ".green()",
            ".blue()",
            ".bold()",
            ".italic()",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// colored = "2.1"

fn main() {
    println!("colored: formatação colorida para terminais modernos no Windows, Linux e macOS.");
    println!("Exemplo conceitual: println!("{}", "✔ Sucesso");");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"colored = "2.1""#),
        docs_url: "https://docs.rs/colored/latest/colored/",
    },
    RustLibItem {
        id: "crate_tempfile",
        name: "tempfile",
        category: LibCategory::FsIo,
        summary: "Criação e limpeza automática de arquivos e pastas temporárias protegidas contra colisões de nome.",
        description: "\
`tempfile` gerencia arquivos na pasta `%TEMP%` do sistema que são automaticamente excluídos quando saem de escopo (RAII via trait `Drop`).",
        key_types: &["NamedTempFile", "TempDir"],
        key_methods: &[
            "NamedTempFile::new()",
            "TempDir::new()",
            ".path()",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// tempfile = "3.10"

fn main() {
    println!("tempfile: cria arquivos descartáveis para testes ou processamento intermediário.");
    println!("  let temp = tempfile::NamedTempFile::new()?;");
    println!("  // Ao final do bloco, o arquivo é automaticamente apagado do disco.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"tempfile = "3.10""#),
        docs_url: "https://docs.rs/tempfile/latest/tempfile/",
    },
    RustLibItem {
        id: "crate_toml",
        name: "toml",
        category: LibCategory::Ecosystem,
        summary: "Parser e serializador para o formato TOML, o padrão oficial de arquivos de configuração em Rust.",
        description: "\
Utilizado pelo próprio Cargo para o `Cargo.toml`. Permite carregar e salvar configurações humanas e legíveis com validação via `serde`.",
        key_types: &["toml::Value", "toml::from_str", "toml::to_string"],
        key_methods: &[
            "toml::from_str::<T>(content)",
            "toml::to_string_pretty(&val)",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// toml = "0.8"

fn main() {
    let toml_texto = "[pacote]
nome = "rrs"
versao = "0.3.0"";
    println!("Exemplo de configuração TOML:
{}", toml_texto);
}
"##,
        is_std: false,
        cargo_dep: Some(r#"toml = "0.8""#),
        docs_url: "https://docs.rs/toml/latest/toml/",
    },
    RustLibItem {
        id: "crate_bincode",
        name: "bincode",
        category: LibCategory::Ecosystem,
        summary: "Codificador e decodificador binário ultrarrápido e compacto para comunicação entre processos ou arquivos de save.",
        description: "\
Diferente de JSON ou XML, `bincode` grava structs na forma binária direta da memória.
Gera arquivos mínimos sem overhead de texto, com velocidades de serialização na ordem de gigabytes por segundo.",
        key_types: &["bincode::serialize", "bincode::deserialize"],
        key_methods: &[
            "bincode::serialize(&data)",
            "bincode::deserialize::<T>(&bytes)",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// bincode = "1.3"

fn main() {
    println!("bincode: serialização binária com máxima densidade de dados e velocidade de gravação.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"bincode = "1.3""#),
        docs_url: "https://docs.rs/bincode/latest/bincode/",
    },
    RustLibItem {
        id: "crate_sysinfo",
        name: "sysinfo",
        category: LibCategory::SystemOs,
        summary: "Monitoramento de hardware multiplataforma: uso de CPU por núcleo, memória RAM/Swap, discos e processos.",
        description: "\
`sysinfo` permite inspecionar recursos do computador em tempo real.
Ideal para diagnósticos de sistema, painéis de monitoramento e ferramentas de otimização de máquina.",
        key_types: &["System", "Disks", "Networks", "Pid"],
        key_methods: &[
            "System::new_all()",
            ".refresh_all()",
            ".total_memory()",
            ".used_memory()",
            ".cpus()",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// sysinfo = "0.30"

fn main() {
    println!("sysinfo: coleta de métricas de CPU, RAM, discos e processos ativos do sistema operacional.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"sysinfo = "0.30""#),
        docs_url: "https://docs.rs/sysinfo/latest/sysinfo/",
    },
    RustLibItem {
        id: "crate_parking_lot",
        name: "parking_lot",
        category: LibCategory::SyncConcurrency,
        summary: "Implementações compactas e significativamente mais rápidas de Mutex, RwLock e Condvar que a Standard Library.",
        description: "\
`parking_lot` substitui primitivas de sincronização da std por versões que ocupam apenas 1 byte por lock, não sofrem de envenenamento e oferecem timeouts e locks reentrantes.",
        key_types: &["Mutex", "RwLock", "Condvar", "ReentrantMutex"],
        key_methods: &[
            "Mutex::new(val)",
            ".lock()",
            "RwLock::new(val)",
            ".read()",
            ".write()",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// parking_lot = "0.12"

fn main() {
    println!("parking_lot: Mutex e RwLock ultra rápidos e leves para concorrência de alta vazão.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"parking_lot = "0.12""#),
        docs_url: "https://docs.rs/parking_lot/latest/parking_lot/",
    },
    RustLibItem {
        id: "crate_tower",
        name: "tower",
        category: LibCategory::Net,
        summary: "Abstração modular de serviços cliente/servidor com middlewares de timeout, rate limit e retentativas.",
        description: "\
`tower` padroniza componentes reutilizáveis para redes e clientes HTTP/gRPC.
Permite encadear middlewares de autenticação, limitação de taxa, tracing e fallback de forma totalmente combinável.",
        key_types: &["Service", "ServiceBuilder", "Layer", "Timeout"],
        key_methods: &[
            "ServiceBuilder::new()",
            ".timeout(Duration)",
            ".rate_limit(100, Duration)",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// tower = "0.5"

fn main() {
    println!("tower: biblioteca de middlewares e composição modular de serviços de rede.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"tower = "0.5""#),
        docs_url: "https://docs.rs/tower/latest/tower/",
    },
    RustLibItem {
        id: "crate_flume",
        name: "flume",
        category: LibCategory::SyncConcurrency,
        summary: "Canais MPMC (multi-produtor multi-consumidor) ultrarrápidos com suporte simultâneo a código síncrono e assíncrono.",
        description: "\
`flume` é um canal seguro e extremamente performático que conecta tarefas síncronas e assíncronas (Tokio/async-std) sem overhead.",
        key_types: &["Sender", "Receiver", "bounded", "unbounded"],
        key_methods: &[
            "flume::unbounded()",
            "flume::bounded(cap)",
            ".send(val)",
            ".recv()",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// flume = "0.11"

fn main() {
    println!("flume: canais MPMC síncronos e assíncronos ultrarrápidos para troca de mensagens.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"flume = "0.11""#),
        docs_url: "https://docs.rs/flume/latest/flume/",
    },
    RustLibItem {
        id: "crate_dialoguer",
        name: "dialoguer",
        category: LibCategory::SystemOs,
        summary: "Prompts interativos no terminal: perguntas, seleções de menu com setas, confirmações e senhas.",
        description: "\
`dialoguer` torna CLIs interativas e amigáveis ao usuário como instaladores profissionais.
Permite selecionar opções de listas interativas, solicitar confirmações [s/N] e digitar senhas ocultando os caracteres.",
        key_types: &["Input", "Select", "MultiSelect", "Confirm", "Password"],
        key_methods: &[
            "Input::new().interact()",
            "Select::new().items(&items).interact()",
            "Confirm::new().interact()",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// dialoguer = "0.11"

fn main() {
    println!("dialoguer: prompts interativos e menus de seleção para ferramentas de linha de comando.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"dialoguer = "0.11""#),
        docs_url: "https://docs.rs/dialoguer/latest/dialoguer/",
    },
    RustLibItem {
        id: "crate_dotenvy",
        name: "dotenvy",
        category: LibCategory::SystemOs,
        summary: "Carregamento seguro e ergonômico de variáveis de ambiente a partir de arquivos .env no início da aplicação.",
        description: "\
`dotenvy` é a continuação moderna e auditada do clássico `dotenv`, facilitando carregar configurações locais de desenvolvimento para `std::env`.",
        key_types: &["dotenv", "from_filename", "var"],
        key_methods: &[
            "dotenvy::dotenv()",
            "dotenvy::var(\"KEY\")",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// dotenvy = "0.15"

fn main() {
    println!("dotenvy: carregamento de variáveis de ambiente a partir de arquivos .env.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"dotenvy = "0.15""#),
        docs_url: "https://docs.rs/dotenvy/latest/dotenvy/",
    },
    RustLibItem {
        id: "crate_crossterm",
        name: "crossterm",
        category: LibCategory::SystemOs,
        summary: "Manipulação avançada de terminal multiplataforma (Windows e Unix): modo raw, controle de cursor e eventos.",
        description: "\
`crossterm` é o motor por trás de gerenciadores TUI modernos como Ratatui e Yazi.
Permite ler teclas em tempo real sem esperar Enter, mover o cursor para qualquer coordenada (x, y) e desenhar interfaces completas.",
        key_types: &["terminal::enable_raw_mode", "event::read", "cursor::MoveTo", "style::Print"],
        key_methods: &[
            "execute!(stdout, MoveTo(x, y))",
            "event::poll(duration)",
            "event::read()",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// crossterm = "0.28"

fn main() {
    println!("crossterm: manipulação de cursor, eventos de teclado e interfaces TUI no console.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"crossterm = "0.28""#),
        docs_url: "https://docs.rs/crossterm/latest/crossterm/",
    },
    RustLibItem {
        id: "crate_walkdir",
        name: "walkdir",
        category: LibCategory::FsIo,
        summary: "Varredura e percorrimento recursivo de pastas no sistema de arquivos com alto desempenho e filtros.",
        description: "\
`walkdir` permite iterar recursivamente sobre diretórios de forma eficiente, tratando links simbólicos e erros de permissão graciosamente.",
        key_types: &["WalkDir", "DirEntry", "IntoIter"],
        key_methods: &[
            "WalkDir::new(path)",
            ".max_depth(3)",
            ".into_iter()",
            ".filter_map(Result::ok)",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// walkdir = "2.5"

fn main() {
    println!("walkdir: varredura e navegação recursiva de diretórios no sistema de arquivos.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"walkdir = "2.5""#),
        docs_url: "https://docs.rs/walkdir/latest/walkdir/",
    },
    RustLibItem {
        id: "crate_csv",
        name: "csv",
        category: LibCategory::Ecosystem,
        summary: "Processamento de arquivos tabulares CSV em streaming com velocidade extrema e suporte a Serde.",
        description: "\
Desenvolvido pelo criador de `ripgrep`, o crate `csv` processa gigabytes de dados sem estourar a memória RAM.
Converte linhas CSV automaticamente para structs tipadas de Rust e oferece geração de relatórios com zero cópias desnecessárias.",
        key_types: &["Reader", "Writer", "StringRecord", "ByteRecord"],
        key_methods: &[
            "Reader::from_path(path)",
            ".deserialize()",
            "Writer::from_writer(w)",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// csv = "1.3"

fn main() {
    println!("csv: ingestão e processamento de dados tabulares em alta velocidade com streaming.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"csv = "1.3""#),
        docs_url: "https://docs.rs/csv/latest/csv/",
    },
    RustLibItem {
        id: "crate_uuid",
        name: "uuid",
        category: LibCategory::Ecosystem,
        summary: "Geração e validação de identificadores únicos universais (UUIDs v4 aleatórios e v7 temporais).",
        description: "\
`uuid` é o padrão para geração de identificadores únicos no ecossistema Rust, essencial para chaves primárias em bancos de dados e sessões.",
        key_types: &["Uuid", "Builder", "Version"],
        key_methods: &[
            "Uuid::new_v4()",
            "Uuid::now_v7()",
            "Uuid::parse_str(s)",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// uuid = { version = "1.8", features = ["v4", "v7", "fast-rng"] }

fn main() {
    println!("uuid: geração e parsing de identificadores únicos universais (v4 e v7).");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"uuid = { version = "1.8", features = ["v4", "v7"] }"#),
        docs_url: "https://docs.rs/uuid/latest/uuid/",
    },
    RustLibItem {
        id: "crate_futures",
        name: "futures",
        category: LibCategory::SyncConcurrency,
        summary: "Abstrações fundamentais e utilitários assíncronos: Stream, Sink, join!, try_join! e FuturesUnordered.",
        description: "\
`futures` é a biblioteca fundamental que define as traits centrais do ecossistema assíncrono de Rust.
Fornece combinadores funcionais para futures e streams, permitindo executar dezenas de tarefas em paralelo com `FuturesUnordered`.",
        key_types: &["Stream", "Sink", "channel", "stream::FuturesUnordered"],
        key_methods: &[
            "join!",
            "try_join!",
            ".next().await",
            "select!",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// futures = "0.3"

fn main() {
    println!("futures: combinadores, streams e abstrações essenciais para Rust assíncrono.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"futures = "0.3""#),
        docs_url: "https://docs.rs/futures/latest/futures/",
    },
    RustLibItem {
        id: "crate_hex",
        name: "hex",
        category: LibCategory::Ecosystem,
        summary: "Codificação e decodificação hexadecimal ultrarrápida de sequências de bytes e hashes.",
        description: "\
`hex` converte buffers binários em representações hexadecimais legíveis e vice-versa.
Possui otimizações de vetorização para processar milhões de hashes por segundo com consumo mínimo de CPU.",
        key_types: &["encode", "decode", "FromHex", "ToHex"],
        key_methods: &[
            "hex::encode(bytes)",
            "hex::decode(string)",
            "hex::encode_upper(bytes)",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// hex = "0.4"

fn main() {
    println!("hex: conversão rápida de hashes e chaves binárias para representação hexadecimal.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"hex = "0.4""#),
        docs_url: "https://docs.rs/hex/latest/hex/",
    },
    RustLibItem {
        id: "crate_dashmap",
        name: "dashmap",
        category: LibCategory::Collections,
        summary: "HashMap concorrente de alto desempenho baseado em múltiplos shards para evitar contenção de locks.",
        description: "\
`dashmap` oferece uma API similar a `std::collections::HashMap`, mas desenhada para acesso simultâneo por centenas de threads.
Elimina o gargalo de `Mutex` ou `RwLock` globais particionando as chaves em shards independentes.",
        key_types: &["DashMap", "DashSet"],
        key_methods: &[
            "DashMap::new()",
            ".insert(k, v)",
            ".get(&k)",
            ".entry(k)",
            ".remove(&k)",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// dashmap = "6.0"

fn main() {
    println!("dashmap: hashmap concorrente de alta vazão sem travamentos globais de thread.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"dashmap = "6.0""#),
        docs_url: "https://docs.rs/dashmap/latest/dashmap/",
    },
    RustLibItem {
        id: "crate_crossbeam",
        name: "crossbeam",
        category: LibCategory::SyncConcurrency,
        summary: "Ferramental avançado de concorrência: canais mpmc, scoped threads e estruturas lock-free.",
        description: "\
`crossbeam` é a fundação para sistemas paralelos complexos em Rust.
Fornece scoped threads (threads que emprestam variáveis da pilha com segurança), canais multi-produtor multi-consumidor de baixa latência e coleções atômicas.",
        key_types: &["channel::unbounded", "channel::bounded", "scope", "epoch"],
        key_methods: &[
            "channel::unbounded()",
            "scope(|s| s.spawn(...))",
            "select!",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// crossbeam = "0.8"

fn main() {
    println!("crossbeam: primitivas e estruturas concorrentes de alto desempenho para threads.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"crossbeam = "0.8""#),
        docs_url: "https://docs.rs/crossbeam/latest/crossbeam/",
    },
    RustLibItem {
        id: "crate_itertools",
        name: "itertools",
        category: LibCategory::Collections,
        summary: "Mais de 100 adaptadores e métodos utilitários extras para enriquecer os iteradores de Rust.",
        description: "\
`itertools` amplia a Standard Library com recursos expressivos como `.unique()`, `.sorted()`, `.group_by()`, `.combinations()` e `.join()`.
Permite escrever processamentos complexos em estilo funcional elegante com abstrações de custo zero.",
        key_types: &["Itertools", "GroupBy", "Chunks"],
        key_methods: &[
            ".unique()",
            ".sorted()",
            ".group_by()",
            ".combinations(k)",
            ".join(\", \")",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// itertools = "0.13"

fn main() {
    println!("itertools: biblioteca de adaptadores funcionais avançados para iteradores em Rust.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"itertools = "0.13""#),
        docs_url: "https://docs.rs/itertools/latest/itertools/",
    },
    RustLibItem {
        id: "crate_image",
        name: "image",
        category: LibCategory::Ecosystem,
        summary: "Processamento de imagens em Rust puro: carregamento, redimensionamento e gravação em PNG, JPEG, WebP e GIF.",
        description: "\
`image` é o padrão de facto para manipulação gráfica e carregamento de texturas em Rust.
Oferece buffers de pixels tipados (`RgbaImage`, `RgbImage`), conversão de formatos e algoritmos de amostragem de alta qualidade sem depender de C/C++.",
        key_types: &["DynamicImage", "RgbaImage", "GenericImageView", "imageops"],
        key_methods: &[
            "image::open(path)",
            ".resize(w, h, filter)",
            ".save(dest)",
            ".dimensions()",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// image = "0.25"

fn main() {
    println!("image: decodificação, processamento e manipulação gráfica de imagens em Rust puro.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"image = "0.25""#),
        docs_url: "https://docs.rs/image/latest/image/",
    },
    RustLibItem {
        id: "crate_rodio",
        name: "rodio",
        category: LibCategory::GameDev,
        summary: "Reprodução de áudio, trilhas sonoras e efeitos em Rust puro multiplataforma (MP3, WAV, OGG, FLAC).",
        description: "\
`rodio` simplifica a reprodução sonora em jogos, ferramentas de notificação e reprodutores multimídia.
Baseado no CPAL, gerencia saídas de áudio no Windows, Linux e macOS com controle de volume, canais independentes e efeitos espaciais.",
        key_types: &["OutputStream", "Sink", "Decoder", "Source"],
        key_methods: &[
            "OutputStream::try_default()",
            "Sink::try_new(&stream)",
            ".append(source)",
            ".set_volume()",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// rodio = "0.19"

fn main() {
    println!("rodio: reprodução de efeitos sonoros e músicas para jogos e aplicativos em Rust.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"rodio = "0.19""#),
        docs_url: "https://docs.rs/rodio/latest/rodio/",
    },
    RustLibItem {
        id: "crate_thiserror",
        name: "thiserror",
        category: LibCategory::Ecosystem,
        summary: "Macro derive padrão da indústria para implementar tipos de erro customizados expressivos com zero esforço.",
        description: "\
`thiserror` elimina o código repetitivo de implementar `std::fmt::Display` e `std::error::Error`.
Permite formatar mensagens com anotações `#[error(\"...\")]`, encadear fontes de erro com `#[from]` e manter erros tipados e limpos.",
        key_types: &["#[derive(Error)]", "#[error(\"...\")]", "#[from]"],
        key_methods: &[
            "#[error(\"...\")]",
            "#[from] std::io::Error",
            "#[source]",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// thiserror = "1.0"

fn main() {
    println!("thiserror: criação declarativa de enums de erro expressivos e idiomáticos em Rust.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"thiserror = "1.0""#),
        docs_url: "https://docs.rs/thiserror/latest/thiserror/",
    },
    RustLibItem {
        id: "crate_tracing",
        name: "tracing",
        category: LibCategory::SystemOs,
        summary: "Instrumentação estruturada, spans assíncronos e telemetria para diagnóstico em produção.",
        description: "\
Mantido pela equipe do Tokio, `tracing` é o sucessor moderno do crate `log`.
Projetado especificamente para código assíncrono, vincula eventos a spans contextuais para rastrear o fluxo de requisições inteiras através de múltiplas tarefas e threads.",
        key_types: &["info!", "warn!", "error!", "span!", "Level"],
        key_methods: &[
            "tracing_subscriber::fmt::init()",
            "info!(\"mensagem\", campo = valor)",
            "span!(Level::INFO, \"nome\")",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// tracing = "0.1"
// tracing-subscriber = "0.3"

fn main() {
    println!("tracing: instrumentação, observabilidade e logs estruturados para Rust moderno.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"tracing = "0.1""#),
        docs_url: "https://docs.rs/tracing/latest/tracing/",
    },
    RustLibItem {
        id: "crate_ratatui",
        name: "ratatui",
        category: LibCategory::Ecosystem,
        summary: "Framework moderno para construção de interfaces ricas de terminal (TUI) com widgets e layout responsivo.",
        description: "\
Sucessor espiritual e fork mantido ativamente do tui-rs. O Ratatui permite desenhar dashboards, exploradores de arquivos \
e ferramentas interativas no terminal com suporte a cores truecolor, mouse, layouts flexíveis e integração nativa com crossterm.",
        key_types: &["Terminal", "Frame", "Layout", "Block", "Paragraph", "List", "Table"],
        key_methods: &[
            "Terminal::new(backend)",
            "terminal.draw(|f| ...)",
            "Layout::default().constraints(...)",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// ratatui = "0.28"
// crossterm = "0.28"

fn main() {
    println!("ratatui: construção de interfaces interativas e dashboards direto no terminal!");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"ratatui = "0.28""#),
        docs_url: "https://docs.rs/ratatui/latest/ratatui/",
    },
    RustLibItem {
        id: "crate_hyper",
        name: "hyper",
        category: LibCategory::Net,
        summary: "Biblioteca HTTP/1.1 e HTTP/2 assíncrona, robusta e de alta performance que alimenta boa parte da web em Rust.",
        description: "\
Implementação fundacional de HTTP em Rust puro, mantida pelo time Tokio. Serve como motor subjacente para frameworks de alto nível \
como Axum e Reqwest, oferecendo controle cirúrgico sobre conexões, headers, multiplexação HTTP/2 e streaming de buffers.",
        key_types: &["Request<Incoming>", "Response<BoxBody>", "client::conn::http1", "server::conn::http2"],
        key_methods: &[
            "Builder::new()",
            "serve_connection()",
            "send_request()",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// hyper = { version = "1.0", features = ["full"] }

fn main() {
    println!("hyper: motor HTTP assíncrono ultra-rápido de baixo nível para servidores e clientes.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"hyper = { version = "1.0", features = ["full"] }"#),
        docs_url: "https://docs.rs/hyper/latest/hyper/",
    },
    RustLibItem {
        id: "crate_tonic",
        name: "tonic",
        category: LibCategory::Net,
        summary: "Implementação nativa de gRPC sobre HTTP/2 com geração automática de código a partir de Protocol Buffers.",
        description: "\
Construído sobre Tokio, Prost e Hyper, o Tonic fornece suporte completo a gRPC de nível de produção. \
Suporta chamadas unárias, streaming de cliente, streaming de servidor, streaming bidirecional e TLS.",
        key_types: &["Request<T>", "Response<T>", "Status", "transport::Server", "transport::Channel"],
        key_methods: &[
            "Server::builder()",
            "add_service()",
            "serve()",
            "Channel::from_static()",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// tonic = "0.12"
// prost = "0.13"

fn main() {
    println!("tonic: comunicação gRPC assíncrona e tipada de altíssima velocidade em Rust.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"tonic = "0.12""#),
        docs_url: "https://docs.rs/tonic/latest/tonic/",
    },
    RustLibItem {
        id: "crate_diesel",
        name: "diesel",
        category: LibCategory::Database,
        summary: "ORM e construtor de consultas seguro contra falhas em tempo de compilação para Postgres, MySQL e SQLite.",
        description: "\
O Diesel elimina erros comuns de SQL ao garantir em tempo de compilação que suas consultas sejam sintaticamente válidas \
e correspondam aos tipos do seu banco. Gera migrações seguras e código com zero custo de abstração.",
        key_types: &["PgConnection", "SqliteConnection", "QueryDsl", "RunQueryDsl", "Insertable"],
        key_methods: &[
            "establish()",
            "select()",
            "filter()",
            "load()",
            "execute()",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// diesel = { version = "2.2", features = ["postgres"] }

fn main() {
    println!("diesel: ORM robusto e tipado garantindo consultas SQL válidas em tempo de compilação.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"diesel = { version = "2.2", features = ["postgres"] }"#),
        docs_url: "https://docs.rs/diesel/latest/diesel/",
    },
    RustLibItem {
        id: "crate_sea_orm",
        name: "sea-orm",
        category: LibCategory::Database,
        summary: "ORM relacional 100% assíncrono e dinâmico construído sobre SQLx para arquiteturas de microsserviços.",
        description: "\
Projetado desde o início para Rust assíncrono (Tokio e async-std). Oferece tipagem estática elegante para entidades, \
suporte a relacionamentos 1-to-N e N-to-N, paginação integrada e migrações fluentes com suporte a Postgres, MySQL e SQLite.",
        key_types: &["Database", "EntityTrait", "ActiveModelTrait", "Select", "DatabaseConnection"],
        key_methods: &[
            "Database::connect()",
            "Entity::find()",
            "ActiveModel::insert()",
            "filter()",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// sea-orm = { version = "1.0", features = ["sqlx-postgres", "runtime-tokio-rustls"] }

fn main() {
    println!("sea-orm: ORM assíncrono e ergonômico para trabalhar com bancos de dados relacionais.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"sea-orm = { version = "1.0", features = ["sqlx-postgres", "runtime-tokio-rustls"] }"#),
        docs_url: "https://docs.rs/sea-orm/latest/sea_orm/",
    },
    RustLibItem {
        id: "crate_polars",
        name: "polars",
        category: LibCategory::Ecosystem,
        summary: "Biblioteca de DataFrames ultrarrápida com execução vetorizada e motor de consulta Lazy com Apache Arrow.",
        description: "\
Escrito do zero em Rust, o Polars é uma das ferramentas analíticas mais velozes do mundo. \
Fornece API de DataFrames no estilo Pandas/R, mas com paralelismo automático multi-thread via Rayon e otimizador de plano lazy.",
        key_types: &["DataFrame", "Series", "LazyFrame", "col", "lit", "Expr"],
        key_methods: &[
            "df.lazy()",
            ".filter()",
            ".group_by()",
            ".agg()",
            ".collect()",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// polars = { version = "0.43", features = ["lazy", "parquet"] }

fn main() {
    println!("polars: processamento analítico e DataFrames em velocidade nativa com suporte a Arrow.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"polars = { version = "0.43", features = ["lazy"] }"#),
        docs_url: "https://docs.rs/polars/latest/polars/",
    },
    RustLibItem {
        id: "crate_ndarray",
        name: "ndarray",
        category: LibCategory::Ecosystem,
        summary: "Arrays n-dimensionais com suporte a álgebra linear, fatiamento dimensional e computação numérica estilo NumPy.",
        description: "\
A espinha dorsal científica de Rust. Fornece tipos Array1, Array2, ArrayD genéricos com fatiamento eficiente, \
broadcasting, operações elemento a elemento e aceleração BLAS/LAPACK opcional.",
        key_types: &["Array", "Array1", "Array2", "ArrayView", "s!", "Axis"],
        key_methods: &[
            "Array::zeros()",
            "Array::from_vec()",
            ".slice()",
            ".dot()",
            ".mapv()",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// ndarray = "0.16"

fn main() {
    println!("ndarray: arrays multidimensionais e computação numérica avançada para Rust.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"ndarray = "0.16""#),
        docs_url: "https://docs.rs/ndarray/latest/ndarray/",
    },
    RustLibItem {
        id: "crate_candle_core",
        name: "candle-core",
        category: LibCategory::Ecosystem,
        summary: "Framework minimalista de Machine Learning e Deep Learning da Hugging Face focado em inferência e leveza.",
        description: "\
Criado pela Hugging Face para rodar modelos modernos de IA (Llama, Whisper, Stable Diffusion, etc.) \
com baixo consumo de memória e sem o peso do PyTorch/libtorch. Suporta aceleração CUDA, Metal e CPU com AVX/NEON.",
        key_types: &["Tensor", "Device", "DType", "Shape", "VarMap"],
        key_methods: &[
            "Tensor::new()",
            "Tensor::zeros()",
            ".matmul()",
            ".to_device()",
            ".forward()",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// candle-core = "0.7"

fn main() {
    println!("candle: inferência de inteligência artificial rápida e sem dependências pesadas.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"candle-core = "0.7""#),
        docs_url: "https://docs.rs/candle-core/latest/candle_core/",
    },
    RustLibItem {
        id: "crate_rustls",
        name: "rustls",
        category: LibCategory::Net,
        summary: "Biblioteca TLS (Transport Layer Security) moderna, segura por design e escrita inteiramente em Rust puro.",
        description: "\
Alternativa moderna e imune a vulnerabilidades de memória comuns do OpenSSL. Suporta TLS 1.2 e TLS 1.3, \
criptografia rápida com ring ou aws-lc-rs e é usada por projetos de grande porte como curl, Caddy e Linkerd.",
        key_types: &["ClientConfig", "ServerConfig", "Stream", "CertificateDer", "PrivateKeyDer"],
        key_methods: &[
            "ClientConfig::builder()",
            "with_root_certificates()",
            "connect()",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// rustls = "0.23"

fn main() {
    println!("rustls: segurança TLS contemporânea livre de vulnerabilidades C/OpenSSL.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"rustls = "0.23""#),
        docs_url: "https://docs.rs/rustls/latest/rustls/",
    },
    RustLibItem {
        id: "crate_ring",
        name: "ring",
        category: LibCategory::Ecosystem,
        summary: "Operações criptográficas focadas em segurança, primitivas rápidas (AES, ChaCha20, SHA, Ed25519) e tempo constante.",
        description: "\
Implementação focada em criptografia de chaves públicas, criptografia simétrica e assinaturas digitais \
com resistência comprovada a ataques de temporização (constant-time execution). Núcleo criptográfico do ecossistema.",
        key_types: &["digest::Digest", "signature::Ed25519KeyPair", "aead::OpeningKey", "rand::SystemRandom"],
        key_methods: &[
            "digest::digest()",
            "key_pair.sign()",
            "open_in_place()",
            "seal_in_place()",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// ring = "0.17"

fn main() {
    println!("ring: primitivas criptográficas velozes e seguras contra ataques de temporização.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"ring = "0.17""#),
        docs_url: "https://docs.rs/ring/latest/ring/",
    },
    RustLibItem {
        id: "crate_nom",
        name: "nom",
        category: LibCategory::Ecosystem,
        summary: "Framework de combinadores de parsers sem alocação, ideal para parsing de protocolos binários e texto.",
        description: "\
O Nom é o padrão de excelência para escrever analisadores sintáticos em Rust sem copiar dados da memória (zero-copy parsing). \
Permite compor pequenos parsers em gramáticas completas de forma legível e com performance máxima.",
        key_types: &["IResult<I, O>", "Parser", "tag", "take_while", "map_res"],
        key_methods: &[
            "nom::bytes::complete::tag()",
            "nom::character::complete::digit1()",
            ".parse()",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// nom = "7.1"

fn main() {
    println!("nom: construção modular e zero-copy de parsers de protocolos e formatos de dados.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"nom = "7.1""#),
        docs_url: "https://docs.rs/nom/latest/nom/",
    },
    RustLibItem {
        id: "crate_syn",
        name: "syn",
        category: LibCategory::Ecosystem,
        summary: "Parser completo de sintaxe Rust para análise de código e desenvolvimento de procedural macros robustas.",
        description: "\
Analisa código Rust e o transforma em uma Árvore de Sintaxe Abstrata (AST) tipada. É o crate fundamental sobre o qual \
praticamente todas as macros declarativas (#[derive], macros de atributo) do ecossistema Rust (como Serde e Clap) são criadas.",
        key_types: &["DeriveInput", "ItemStruct", "ItemEnum", "Type", "Expr", "Path"],
        key_methods: &[
            "syn::parse_macro_input!()",
            "syn::parse_str()",
            "syn::parse2()",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// syn = { version = "2.0", features = ["full", "extra-traits"] }

fn main() {
    println!("syn: análise sintática da linguagem Rust para metaprogramação com proc-macros.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"syn = { version = "2.0", features = ["full"] }"#),
        docs_url: "https://docs.rs/syn/latest/syn/",
    },
    RustLibItem {
        id: "crate_quote",
        name: "quote",
        category: LibCategory::Ecosystem,
        summary: "Quasi-quoting idiomático para geração limpa de tokens Rust em procedural macros.",
        description: "\
Complemento direto do `syn`. Permite escrever código Rust diretamente dentro da macro quote! { ... } \
interpolando variáveis e tipos da AST de forma intuitiva, convertendo estruturas de dados em tokens válidos para o compilador.",
        key_types: &["quote!", "ToTokens", "TokenStreamExt", "format_ident!"],
        key_methods: &[
            "quote! { ... }",
            "format_ident!(\"{}\", name)",
            ".to_token_stream()",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// quote = "1.0"
// proc-macro2 = "1.0"

fn main() {
    println!("quote: geração elegante de código Rust via interpolação de tokens em procedural macros.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"quote = "1.0""#),
        docs_url: "https://docs.rs/quote/latest/quote/",
    },
    RustLibItem {
        id: "crate_notify",
        name: "notify",
        category: LibCategory::FsIo,
        summary: "Monitoramento em tempo real de eventos do sistema de arquivos com suporte nativo entre plataformas.",
        description: "\
Notifica sua aplicação sempre que arquivos ou pastas forem criados, modificados, renomeados ou excluídos. \
Utiliza as APIs de eventos mais eficientes do SO (ReadDirectoryChangesW no Windows, inotify no Linux, FSEvents no macOS).",
        key_types: &["RecommendedWatcher", "Event", "EventKind", "RecursiveMode", "Watcher"],
        key_methods: &[
            "recommended_watcher()",
            "watcher.watch(path, mode)",
            "watcher.unwatch()",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// notify = "6.1"

fn main() {
    println!("notify: monitoramento de alterações em arquivos e pastas com eventos em tempo real.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"notify = "6.1""#),
        docs_url: "https://docs.rs/notify/latest/notify/",
    },
    RustLibItem {
        id: "crate_criterion",
        name: "criterion",
        category: LibCategory::Ecosystem,
        summary: "Microbenchmarking com rigor estatístico, detecção de regressões de desempenho e gráficos detalhados.",
        description: "\
A ferramenta de benchmarking mais avançada de Rust. Isola ruído do sistema operacional com amostragem estatística \
rigorosa (intervalos de confiança, teste t de Student) e detecta variações mínimas de nanosegundos em execuções de código.",
        key_types: &["Criterion", "BenchmarkGroup", "BenchmarkId", "criterion_group!", "criterion_main!"],
        key_methods: &[
            "c.bench_function(\"nome\", |b| b.iter(|| ...))",
            "c.benchmark_group()",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// [dev-dependencies]
// criterion = { version = "0.5", features = ["html_reports"] }

fn main() {
    println!("criterion: medição estatística precisa de tempo de CPU e regressão de performance.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"criterion = "0.5""#),
        docs_url: "https://docs.rs/criterion/latest/criterion/",
    },
    RustLibItem {
        id: "crate_mockall",
        name: "mockall",
        category: LibCategory::Ecosystem,
        summary: "Framework completo para criação de Mocks e stubs em testes unitários com suporte a traits e structs.",
        description: "\
Permite isolar dependências externas como conexões de rede, bancos de dados ou relógios em testes automatizados. \
Gera implementações falsas com #[automock] permitindo configurar expectativas, valores de retorno e contagem de chamadas.",
        key_types: &["#[automock]", "mock!", "Sequence", "Predicate"],
        key_methods: &[
            ".expect()",
            ".returning()",
            ".times()",
            ".with()",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// [dev-dependencies]
// mockall = "0.13"

fn main() {
    println!("mockall: isolamento de componentes e testes unitários controlados com mocks tipados.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"mockall = "0.13""#),
        docs_url: "https://docs.rs/mockall/latest/mockall/",
    },
    RustLibItem {
        id: "crate_proptest",
        name: "proptest",
        category: LibCategory::Ecosystem,
        summary: "Testes baseados em propriedades com encolhimento automático de casos de falha (QuickCheck moderno).",
        description: "\
Em vez de testar poucos exemplos manuais, define propriedades lógicas invariantes e gera milhares de entradas \
aleatórias para tentar quebrar seu código. Quando encontra um bug, encolhe (shrink) a entrada até o menor contraexemplo possível.",
        key_types: &["proptest!", "Strategy", "Just", "prop_compose!", "TestRunner"],
        key_methods: &[
            "proptest! { #[test] fn prop_test(val in any::<u32>()) { ... } }",
            "prop_assert!",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// [dev-dependencies]
// proptest = "1.5"

fn main() {
    println!("proptest: busca automática por bugs e contraexemplos através de property testing.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"proptest = "1.5""#),
        docs_url: "https://docs.rs/proptest/latest/proptest/",
    },
    RustLibItem {
        id: "crate_zeroize",
        name: "zeroize",
        category: LibCategory::MemoryTypes,
        summary: "Limpeza segura e garantida de segredos em memória contra otimizações agressivas do compilador.",
        description: "\
Compiladores modernos frequentemente descartam instruções que limpam buffers de memória se a variável não for usada em seguida. \
O zeroize utiliza barreiras de memória e código volátil para garantir que senhas, chaves criptográficas e tokens sejam apagados da RAM.",
        key_types: &["Zeroize", "ZeroizeOnDrop", "Zeroizing<Z>"],
        key_methods: &[
            ".zeroize()",
            "Zeroizing::new(secret)",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// zeroize = { version = "1.8", features = ["derive"] }

fn main() {
    println!("zeroize: eliminação garantida de segredos e chaves criptográficas da memória RAM.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"zeroize = { version = "1.8", features = ["derive"] }"#),
        docs_url: "https://docs.rs/zeroize/latest/zeroize/",
    },
    RustLibItem {
        id: "crate_semver",
        name: "semver",
        category: LibCategory::Ecosystem,
        summary: "Análise, validação e comparação de versões seguindo as regras oficiais do Semantic Versioning 2.0.0.",
        description: "\
O mesmo motor que alimenta o Cargo para gerenciar compatibilidade entre versões de pacotes. \
Permite analisar números de versão (major, minor, patch, pre-release) e avaliar operadores de restrição como ^1.2, ~2.0 e >=1.0, <2.0.",
        key_types: &["Version", "VersionReq", "Op", "Prerelease", "BuildMetadata"],
        key_methods: &[
            "Version::parse()",
            "VersionReq::parse()",
            "req.matches(&version)",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// semver = "1.0"

fn main() {
    println!("semver: compatibilidade, checagem e resolução de especificações semver 2.0.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"semver = "1.0""#),
        docs_url: "https://docs.rs/semver/latest/semver/",
    },
    RustLibItem {
        id: "crate_env_logger",
        name: "env_logger",
        category: LibCategory::Ecosystem,
        summary: "Logger configurável via variáveis de ambiente (RUST_LOG) ideal para desenvolvimento e ferramentas CLI.",
        description: "\
Implementação simples e altamente popular para o crate log. Permite habilitar e filtrar saídas de debug ou trace \
em módulos específicos dinamicamente no terminal apenas definindo a variável RUST_LOG=info ou RUST_LOG=meu_crate=debug.",
        key_types: &["Builder", "Env", "Target", "TimestampPrecision"],
        key_methods: &[
            "env_logger::init()",
            "Builder::from_env().init()",
            "Builder::new()",
        ],
        example_code: r##"// Adicione no Cargo.toml:
// log = "0.4"
// env_logger = "0.11"

fn main() {
    println!("env_logger: ativação e filtragem rápida de logs de debug via variável RUST_LOG.");
}
"##,
        is_std: false,
        cargo_dep: Some(r#"env_logger = "0.11""#),
        docs_url: "https://docs.rs/env_logger/latest/env_logger/",
    },

];

/// Busca bibliotecas por texto de pesquisa e categoria.
pub fn search_libs(query: &str, category_filter: &str) -> Vec<&'static RustLibItem> {
    let q = query.trim().to_lowercase();
    RUST_LIBS
        .iter()
        .filter(|lib| {
            let cat_ok = category_filter.eq_ignore_ascii_case("todas")
                || lib.category.label().to_lowercase().contains(&category_filter.to_lowercase());
            if !cat_ok {
                return false;
            }
            if q.is_empty() {
                true
            } else {
                lib.name.to_lowercase().contains(&q)
                    || lib.summary.to_lowercase().contains(&q)
                    || lib.description.to_lowercase().contains(&q)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogo_de_bibliotecas_tem_itens_validos() {
        assert!(RUST_LIBS.len() >= 80, "devem existir pelo menos 80 bibliotecas documentadas (atual: {})", RUST_LIBS.len());
        let mut ids = std::collections::HashSet::new();
        for lib in RUST_LIBS {
            assert!(ids.insert(lib.id), "id repetido: {}", lib.id);
            assert!(!lib.name.is_empty());
            assert!(!lib.summary.is_empty());
            assert!(!lib.description.is_empty());
            assert!(lib.example_code.contains("fn main()"), "o exemplo de {} deve ter uma main()", lib.id);
            assert!(!lib.key_types.is_empty() || !lib.key_methods.is_empty());
        }
    }

    #[test]
    fn capitulos_do_livro_oficial_estao_completos() {
        assert_eq!(RUST_BOOK_CHAPTERS.len(), 20, "o livro oficial tem 20 capítulos");
        for ch in RUST_BOOK_CHAPTERS {
            assert!(ch.number >= 1 && ch.number <= 20);
            assert!(!ch.title.is_empty());
            assert!(!ch.summary.is_empty());
            assert!(!ch.topics.is_empty());
            assert!(ch.web_url_en.starts_with("https://"));
            assert!(ch.web_url_pt.starts_with("https://"));
        }
    }

    #[test]
    fn busca_de_bibliotecas_funciona() {
        let hashmap_results = search_libs("hashmap", "todas");
        assert!(hashmap_results.iter().any(|lib| lib.id == "std_hashmap"));

        let sync_results = search_libs("", "concorrência");
        assert!(sync_results.len() >= 4);

        let gamedev_results = search_libs("", "jogos");
        assert!(gamedev_results.len() >= 2);
    }
}
