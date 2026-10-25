//! Catálogo e Referência Completa da Biblioteca Rust (Standard Library `std` & Top Crates do Ecossistema).
//! Fornece documentação em português, assinatura de tipos essenciais, métodos principais e exemplos de código
//! executáveis prontos para testar no editor da RRS e integrar a projetos Cargo.

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
}

impl LibCategory {
    pub fn label(self) -> &'static str {
        match self {
            LibCategory::Collections => "Coleções (std::collections)",
            LibCategory::SyncConcurrency => "Concorrência & Sincronização (std::sync)",
            LibCategory::FsIo => "Arquivos & Entrada/Saída (std::fs, std::io)",
            LibCategory::Net => "Redes & Sockets (std::net)",
            LibCategory::SystemOs => "Processos & Sistema (std::process, std::env)",
            LibCategory::MemoryTypes => "Tipos & Memória (std::cell, std::time, std::path)",
            LibCategory::Ecosystem => "Top Crates do Ecossistema (Cargo)",
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
        assert!(RUST_LIBS.len() >= 20, "devem existir mais de 20 bibliotecas documentadas");
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
    fn busca_de_bibliotecas_funciona() {
        let hashmap_results = search_libs("hashmap", "todas");
        assert!(hashmap_results.iter().any(|lib| lib.id == "std_hashmap"));

        let sync_results = search_libs("", "concorrência");
        assert!(sync_results.len() >= 3);
    }
}
