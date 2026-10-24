//! Tradutor e Transpilador Estrutural de C / C++ para Rust.
//!
//! Suporta dois modos:
//! - `SafeRust`: converte para Rust idiomático e seguro (`Vec<T>`, `String`, `for i in 0..n`, `println!`, `&mut T`).
//! - `FfiBindings`: gera tipos e assinaturas compatíveis com ABI C (`#[repr(C)]`, `*mut T`, `extern "C"`).

use std::fmt::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslateMode {
    /// Rust idiomático e seguro (Vec, String, slices, ranges)
    SafeRust,
    /// Bindings / FFI compatível com ABI C (#[repr(C)], ponteiros brutos, extern "C")
    FfiBindings,
}

#[derive(Debug, Clone)]
pub struct TranslationResult {
    pub rust_code: String,
    pub notes: Vec<String>,
}

pub struct PresetExample {
    pub name: &'static str,
    pub description: &'static str,
    pub cpp_code: &'static str,
}

pub const EXAMPLES: &[PresetExample] = &[
    PresetExample {
        name: "1. Struct + Funções C (Física / Jogos)",
        description: "Converte #define, struct C, loops for clássicos, ponteiros e printf para Rust.",
        cpp_code: r#"#include <stdio.h>
#include <stdint.h>

#define MAX_PARTICLES 500
#define GRAVITY 9.81f

struct Vec2 {
    float x;
    float y;
};

int somar_energia(int base, int bonus) {
    int total = base + bonus;
    for (int i = 0; i < 5; i++) {
        total += i;
    }
    if (total > 100) {
        printf("Energia maxima atingida: %d\n", total);
    }
    return total;
}
"#,
    },
    PresetExample {
        name: "2. C++ Moderno (std::vector, std::string, cout)",
        description: "Converte containers da STL (std::vector, std::string) e std::cout para equivalentes nativos em Rust.",
        cpp_code: r#"#include <iostream>
#include <vector>
#include <string>

struct Jogador {
    std::string nome;
    int32_t nivel;
    float vida;
};

void exibir_status(int pontos) {
    std::vector<int> historico;
    std::string titulo = "Mestre do Codigo";
    int contador = 0;

    while (contador < 3) {
        historico.push_back(pontos + contador);
        contador++;
    }

    std::cout << "Jogador pronto!" << std::endl;
}
"#,
    },
    PresetExample {
        name: "3. Header C / Engenharia Reversa (FFI / Struct Binária)",
        description: "Ideal para usar no modo FFI (#[repr(C)]) ao portar headers de jogos ou analisar pacotes binários.",
        cpp_code: r#"#include <stdint.h>
#include <stdbool.h>

#define PACKET_MAGIC 0x52525331
#define HEADER_SIZE 16

enum PacketType {
    PKT_HANDSHAKE = 0,
    PKT_LOGIN = 1,
    PKT_DATA = 2,
    PKT_DISCONNECT = 3
};

struct PacketHeader {
    uint32_t magic;
    uint16_t version;
    uint16_t payload_len;
    uint64_t timestamp;
    bool encrypted;
};

bool validar_pacote(uint32_t magic, uint16_t len) {
    if (magic == PACKET_MAGIC) {
        return true;
    }
    return false;
}
"#,
    },
];

pub fn translate_c_cpp_to_rust(source: &str, mode: TranslateMode) -> TranslationResult {
    let source = source.trim_start_matches('\u{feff}');
    let mut out = String::new();
    let mut notes = Vec::new();

    let _ = writeln!(
        out,
        "// ============================================================================"
    );
    let _ = writeln!(
        out,
        "// Código Rust gerado pelo Tradutor RRS (Riva Rust Store) — Modo: {}",
        match mode {
            TranslateMode::SafeRust => "Rust Seguro & Idiomático",
            TranslateMode::FfiBindings => "FFI / Compatível com ABI C (#[repr(C)])",
        }
    );
    let _ = writeln!(
        out,
        "// ============================================================================\n"
    );

    // 1. Detectar #includes que precisam de `use` no Rust
    let mut needs_hashmap = false;
    let mut needs_btreemap = false;
    let mut needs_ffi = false;

    for line in source.lines() {
        let t = line.trim();
        if t.contains("std::unordered_map") || t.contains("<unordered_map>") {
            needs_hashmap = true;
        }
        if t.contains("std::map") || t.contains("<map>") {
            needs_btreemap = true;
        }
        if mode == TranslateMode::FfiBindings && (t.contains("char*") || t.contains("void*")) {
            needs_ffi = true;
        }
    }

    if needs_hashmap {
        let _ = writeln!(out, "use std::collections::HashMap;");
    }
    if needs_btreemap {
        let _ = writeln!(out, "use std::collections::BTreeMap;");
    }
    if needs_ffi {
        let _ = writeln!(out, "use std::ffi::{{c_char, c_void}};");
    }
    if needs_hashmap || needs_btreemap || needs_ffi {
        let _ = writeln!(out);
    }

    // Estado de parsing estrutural linha a linha
    #[derive(PartialEq, Clone, Copy)]
    enum BlockKind {
        Normal,
        Struct,
        Enum,
    }

    let mut stack: Vec<BlockKind> = Vec::new();
    let mut indent_level: usize = 0;

    for (line_idx, raw_line) in source.lines().enumerate() {
        let line_num = line_idx + 1;
        let trimmed = raw_line.trim();

        if trimmed.is_empty() {
            let _ = writeln!(out);
            continue;
        }

        // Preservar comentários
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            let _ = writeln!(out, "{}{}", "    ".repeat(indent_level), trimmed);
            continue;
        }

        // Ignorar #include / #pragma / #ifndef / #endif
        if trimmed.starts_with("#include")
            || trimmed.starts_with("#pragma")
            || trimmed.starts_with("#ifndef")
            || trimmed.starts_with("#ifdef")
            || trimmed.starts_with("#endif")
        {
            continue;
        }

        // Converter #define NOME VALOR -> pub const
        if let Some(rest) = trimmed.strip_prefix("#define ") {
            if let Some((name, val)) = split_define(rest) {
                let (clean_val, ty) = infer_const_type(val);
                let _ = writeln!(
                    out,
                    "{}pub const {}: {} = {};",
                    "    ".repeat(indent_level),
                    name,
                    ty,
                    clean_val
                );
                continue;
            }
        }

        // Fechamento de bloco (`};` ou `}`)
        if trimmed == "}" || trimmed == "};" {
            indent_level = indent_level.saturating_sub(1);
            let _ = stack.pop();
            let _ = writeln!(out, "{}}}", "    ".repeat(indent_level));
            continue;
        }

        let current_block = stack.last().copied().unwrap_or(BlockKind::Normal);

        // Dentro de `struct` ou `class`
        if current_block == BlockKind::Struct {
            if trimmed == "public:" || trimmed == "private:" || trimmed == "protected:" {
                continue;
            }
            if let Some(field_line) = translate_struct_field(trimmed, mode) {
                let _ = writeln!(out, "{}{}", "    ".repeat(indent_level), field_line);
                continue;
            }
        }

        // Dentro de `enum`
        if current_block == BlockKind::Enum {
            let clean_enum = trimmed.trim_end_matches(';');
            let _ = writeln!(out, "{}{}", "    ".repeat(indent_level), clean_enum);
            continue;
        }

        // Início de `struct Nome {` ou `class Nome {`
        if let Some(struct_name) = parse_struct_or_class_header(trimmed) {
            if mode == TranslateMode::FfiBindings {
                let _ = writeln!(out, "{}#[repr(C)]", "    ".repeat(indent_level));
                let _ = writeln!(
                    out,
                    "{}#[derive(Debug, Clone, Copy, Default)]",
                    "    ".repeat(indent_level)
                );
            } else {
                let _ = writeln!(
                    out,
                    "{}#[derive(Debug, Clone, Default)]",
                    "    ".repeat(indent_level)
                );
            }
            let _ = writeln!(
                out,
                "{}pub struct {} {{",
                "    ".repeat(indent_level),
                struct_name
            );
            stack.push(BlockKind::Struct);
            indent_level += 1;
            continue;
        }

        // Início de `enum Nome {`
        if let Some(enum_name) = parse_enum_header(trimmed) {
            let _ = writeln!(out, "{}#[repr(C)]", "    ".repeat(indent_level));
            let _ = writeln!(
                out,
                "{}#[derive(Debug, Clone, Copy, PartialEq, Eq)]",
                "    ".repeat(indent_level)
            );
            let _ = writeln!(
                out,
                "{}pub enum {} {{",
                "    ".repeat(indent_level),
                enum_name
            );
            stack.push(BlockKind::Enum);
            indent_level += 1;
            continue;
        }

        // Início de função `tipo nome(params) {`
        if indent_level == 0 && trimmed.ends_with('{') && trimmed.contains('(') && trimmed.contains(')') {
            if let Some(fn_header) = translate_function_header(trimmed, mode, &mut notes, line_num) {
                let _ = writeln!(out, "{}{}", "    ".repeat(indent_level), fn_header);
                stack.push(BlockKind::Normal);
                indent_level += 1;
                continue;
            }
        }

        // Loop `for (int i = a; i < b; i++) {`
        if let Some(for_rust) = translate_for_loop(trimmed) {
            let _ = writeln!(out, "{}{}", "    ".repeat(indent_level), for_rust);
            stack.push(BlockKind::Normal);
            indent_level += 1;
            continue;
        }

        // `while (cond) {`
        if let Some(while_rust) = translate_while_loop(trimmed) {
            let _ = writeln!(out, "{}{}", "    ".repeat(indent_level), while_rust);
            stack.push(BlockKind::Normal);
            indent_level += 1;
            continue;
        }

        // `if (cond) {` / `} else if (cond) {` / `} else {`
        if let Some((dedent_before, if_rust, indent_after)) = translate_if_else(trimmed) {
            if dedent_before {
                indent_level = indent_level.saturating_sub(1);
                let _ = stack.pop();
            }
            let _ = writeln!(out, "{}{}", "    ".repeat(indent_level), if_rust);
            if indent_after {
                stack.push(BlockKind::Normal);
                indent_level += 1;
            }
            continue;
        }

        // Instruções dentro de funções (printf, std::cout, return, incremento, variáveis)
        let stmt = translate_statement(trimmed, mode, &mut notes, line_num);
        let _ = writeln!(out, "{}{}", "    ".repeat(indent_level), stmt);
        if trimmed.ends_with('{') {
            stack.push(BlockKind::Normal);
            indent_level += 1;
        }
    }

    if notes.is_empty() {
        notes.push(
            "✔ Tradução concluída sem alertas críticos. Estruturas, tipos e controle de fluxo convertidos."
                .into(),
        );
    }

    TranslationResult {
        rust_code: out,
        notes,
    }
}

/// Testa se o código Rust gerado compila limpo usando o `rustc` instalado no sistema.
pub fn verify_rust_syntax(rust_code: &str) -> Result<String, String> {
    let temp_dir = std::env::temp_dir();
    let src_path = temp_dir.join("rrs_translated_check.rs");
    let out_path = temp_dir.join("rrs_translated_check.rmeta");

    std::fs::write(&src_path, rust_code).map_err(|e| format!("Falha ao criar arquivo temporário: {e}"))?;

    let output = std::process::Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-type=lib")
        .arg("--emit=metadata")
        .arg("-A")
        .arg("dead_code")
        .arg("-A")
        .arg("unused_variables")
        .arg("-A")
        .arg("unused_mut")
        .arg("-A")
        .arg("non_camel_case_types")
        .arg(&src_path)
        .arg("-o")
        .arg(&out_path)
        .output()
        .map_err(|e| format!("Não foi possível executar o rustc: {e}"))?;

    let _ = std::fs::remove_file(&src_path);
    let _ = std::fs::remove_file(&out_path);

    if output.status.success() {
        Ok("✔ Código Rust validado pelo compilador `rustc` (0 erros de sintaxe/tipos)!".into())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(stderr.trim().to_string())
    }
}

fn split_define(rest: &str) -> Option<(&str, &str)> {
    let rest = rest.trim();
    let idx = rest.find(char::is_whitespace)?;
    let name = rest[..idx].trim();
    let val = rest[idx..].trim();
    if name.contains('(') || val.is_empty() {
        return None;
    }
    Some((name, val))
}

fn infer_const_type(val: &str) -> (String, &'static str) {
    let v = val.trim().trim_end_matches(';');
    if v.starts_with("0x") || v.starts_with("0X") {
        return (v.to_string(), "u32");
    }
    if v.ends_with('f') || v.ends_with('F') {
        let num = v.trim_end_matches(['f', 'F']);
        return (num.to_string(), "f32");
    }
    if v.contains('.') {
        return (v.to_string(), "f64");
    }
    if v.starts_with('"') && v.ends_with('"') {
        return (v.to_string(), "&str");
    }
    if v.parse::<i64>().is_ok() {
        return (v.to_string(), "i32");
    }
    (v.to_string(), "i32")
}

fn parse_struct_or_class_header(line: &str) -> Option<String> {
    let rest = line
        .strip_prefix("struct ")
        .or_else(|| line.strip_prefix("class "))
        .or_else(|| line.strip_prefix("typedef struct "))?;
    let name = rest.trim_end_matches('{').trim();
    if name.is_empty() || name.contains(' ') || name.contains('(') {
        return None;
    }
    Some(name.to_string())
}

fn parse_enum_header(line: &str) -> Option<String> {
    let rest = line
        .strip_prefix("enum class ")
        .or_else(|| line.strip_prefix("enum "))?;
    let name = rest.trim_end_matches('{').split(':').next()?.trim();
    if name.is_empty() || name.contains(' ') {
        return None;
    }
    Some(name.to_string())
}

fn translate_struct_field(line: &str, mode: TranslateMode) -> Option<String> {
    let clean = line.trim_end_matches(';').trim();
    if clean.is_empty() || clean.contains('(') {
        return None;
    }
    let last_space = clean.rfind(char::is_whitespace)?;
    let c_type = clean[..last_space].trim();
    let var_name = clean[last_space..].trim();

    // Suporte a arrays fixos tipo `char nome[32]`
    if let Some(br_idx) = var_name.find('[') {
        let real_name = &var_name[..br_idx];
        let len_str = var_name[br_idx + 1..].trim_end_matches(']');
        let r_type = map_c_type(c_type, mode);
        return Some(format!("pub {real_name}: [{r_type}; {len_str}],"));
    }

    let r_type = map_c_type(c_type, mode);
    Some(format!("pub {var_name}: {r_type},"))
}

fn translate_function_header(
    line: &str,
    mode: TranslateMode,
    notes: &mut Vec<String>,
    line_num: usize,
) -> Option<String> {
    let without_brace = line.trim_end_matches('{').trim();
    let open_paren = without_brace.find('(')?;
    let close_paren = without_brace.rfind(')')?;

    let before_paren = without_brace[..open_paren].trim();
    let params_raw = without_brace[open_paren + 1..close_paren].trim();

    let last_space = before_paren.rfind(char::is_whitespace)?;
    let ret_c = before_paren[..last_space]
        .trim()
        .trim_start_matches("static ")
        .trim_start_matches("inline ")
        .trim();
    let fn_name = before_paren[last_space..].trim();

    let mut rust_params = Vec::new();
    if !params_raw.is_empty() && params_raw != "void" {
        for p in params_raw.split(',') {
            let p = p.trim();
            if let Some(idx) = p.rfind(char::is_whitespace) {
                let p_type_raw = p[..idx].trim();
                let mut p_name = p[idx..].trim();
                let mut ptr_prefix = "";
                while let Some(stripped) = p_name.strip_prefix('*') {
                    p_name = stripped;
                    ptr_prefix = "*";
                }
                while let Some(stripped) = p_name.strip_prefix('&') {
                    p_name = stripped;
                    ptr_prefix = "&";
                }
                let full_c_type = format!("{p_type_raw}{ptr_prefix}");
                let r_type = map_c_type(&full_c_type, mode);
                if full_c_type.contains('*') {
                    notes.push(format!(
                        "Linha {line_num}: Parâmetro ponteiro `{p_name}` em `{fn_name}` convertido para `{r_type}`."
                    ));
                }
                rust_params.push(format!("{p_name}: {r_type}"));
            }
        }
    }

    let ret_part = if ret_c == "void" || fn_name == "main" {
        String::new()
    } else {
        format!(" -> {}", map_c_type(ret_c, mode))
    };

    let prefix = if mode == TranslateMode::FfiBindings && fn_name != "main" {
        "#[no_mangle]\npub extern \"C\" fn"
    } else if fn_name == "main" {
        "fn"
    } else {
        "pub fn"
    };

    Some(format!(
        "{prefix} {fn_name}({}){ret_part} {{",
        rust_params.join(", ")
    ))
}

fn translate_for_loop(line: &str) -> Option<String> {
    let rest = line.strip_prefix("for")?.trim();
    let rest = rest.strip_prefix('(')?.trim_end_matches('{').trim();
    let rest = rest.strip_suffix(')')?.trim();

    let parts: Vec<&str> = rest.split(';').map(|s| s.trim()).collect();
    if parts.len() != 3 {
        return None;
    }

    // Ex: `int i = 0`
    let init = parts[0];
    let eq_idx = init.find('=')?;
    let start_val = init[eq_idx + 1..].trim();
    let lhs = init[..eq_idx].trim();
    let var = lhs.split_whitespace().last()?;

    // Ex: `i < 10` ou `i <= 10`
    let cond = parts[1];
    if let Some(lt_idx) = cond.find("<=") {
        let end_val = cond[lt_idx + 2..].trim();
        return Some(format!("for {var} in {start_val}..={end_val} {{"));
    }
    if let Some(lt_idx) = cond.find('<') {
        let end_val = cond[lt_idx + 1..].trim();
        return Some(format!("for {var} in {start_val}..{end_val} {{"));
    }
    None
}

fn translate_while_loop(line: &str) -> Option<String> {
    let rest = line.strip_prefix("while")?.trim();
    if !rest.starts_with('(') || !rest.ends_with('{') {
        return None;
    }
    let cond = rest
        .trim_end_matches('{')
        .trim()
        .strip_prefix('(')?
        .strip_suffix(')')?
        .trim();
    Some(format!("while {cond} {{"))
}

fn translate_if_else(line: &str) -> Option<(bool, String, bool)> {
    if line == "} else {" || line == "else {" {
        let dedent = line.starts_with('}');
        return Some((dedent, "} else {".to_string(), true));
    }
    let (dedent, rest) = if let Some(r) = line.strip_prefix("} else if") {
        (true, r.trim())
    } else if let Some(r) = line.strip_prefix("else if") {
        (false, r.trim())
    } else if let Some(r) = line.strip_prefix("if") {
        if !r.starts_with(' ') && !r.starts_with('(') {
            return None;
        }
        (false, r.trim())
    } else {
        return None;
    };

    if !rest.starts_with('(') || !rest.ends_with('{') {
        return None;
    }
    let cond = rest
        .trim_end_matches('{')
        .trim()
        .strip_prefix('(')?
        .strip_suffix(')')?
        .trim()
        .replace("NULL", "None")
        .replace("nullptr", "None");

    let kw = if dedent {
        "} else if"
    } else if line.starts_with("else if") {
        "else if"
    } else {
        "if"
    };
    Some((dedent, format!("{kw} {cond} {{"), true))
}

fn translate_statement(
    line: &str,
    mode: TranslateMode,
    notes: &mut Vec<String>,
    line_num: usize,
) -> String {
    let no_semi = line.trim_end_matches(';').trim();

    // 1. Incremento / decremento simples (`i++`, `++i`, `i--`)
    if let Some(v) = no_semi.strip_suffix("++").or_else(|| no_semi.strip_prefix("++")) {
        return format!("{} += 1;", v.trim());
    }
    if let Some(v) = no_semi.strip_suffix("--").or_else(|| no_semi.strip_prefix("--")) {
        return format!("{} -= 1;", v.trim());
    }

    // 2. `printf("fmt", a, b)` -> `println!(...)`
    if let Some(inner) = no_semi
        .strip_prefix("printf(")
        .and_then(|s| s.strip_suffix(')'))
    {
        return convert_printf(inner);
    }

    // 3. `std::cout << ...` -> `println!(...)`
    if no_semi.starts_with("std::cout") || no_semi.starts_with("cout") {
        return convert_cout(no_semi);
    }

    // 4. `free(ptr)` / `delete ptr`
    if let Some(ptr) = no_semi
        .strip_prefix("free(")
        .and_then(|s| s.strip_suffix(')'))
        .or_else(|| no_semi.strip_prefix("delete "))
    {
        notes.push(format!(
            "Linha {line_num}: `{line}` substituído por gerenciamento automático de memória (RAII/drop) do Rust."
        ));
        return format!("drop({}); // Memória liberada automaticamente em Rust", ptr.trim());
    }

    // 5. Declaração de container vazio `std::vector<T> nome;`
    if no_semi.starts_with("std::vector<") && !no_semi.contains('=') {
        if let Some(idx) = no_semi.rfind(char::is_whitespace) {
            let c_ty = no_semi[..idx].trim();
            let v_name = no_semi[idx..].trim();
            let r_ty = map_c_type(c_ty, mode);
            return format!("let mut {v_name}: {r_ty} = Vec::new();");
        }
    }

    // 6. Declaração de variável com inicialização `tipo nome = valor;`
    if let Some(eq_idx) = no_semi.find('=') {
        // Garantir que não é `==`, `!=`, `+=`, `-=`, `<=`, `>=`
        let before_eq = no_semi[..eq_idx].trim();
        let after_eq = no_semi[eq_idx + 1..].trim();
        let prev_char = no_semi[..eq_idx].chars().last().unwrap_or(' ');
        let next_char = after_eq.chars().next().unwrap_or(' ');

        if !['=', '!', '+', '-', '*', '/', '<', '>'].contains(&prev_char) && next_char != '=' {
            let tokens: Vec<&str> = before_eq.split_whitespace().collect();
            if tokens.len() >= 2 {
                let is_const = tokens[0] == "const" || tokens[0] == "constexpr";
                let var_name = tokens.last().unwrap().trim_start_matches('*');
                let c_ty = tokens[..tokens.len() - 1]
                    .iter()
                    .copied()
                    .filter(|&t| t != "const" && t != "constexpr")
                    .collect::<Vec<_>>()
                    .join(" ");

                let mut rhs = after_eq
                    .replace("nullptr", "None")
                    .replace("NULL", "None");
                if rhs.ends_with('f')
                    && rhs[..rhs.len() - 1]
                        .chars()
                        .all(|c| c.is_ascii_digit() || c == '.')
                {
                    rhs.pop();
                }
                if c_ty == "std::string" && rhs.starts_with('"') {
                    rhs = format!("String::from({rhs})");
                }

                let mut_kw = if is_const { "let" } else { "let mut" };
                if c_ty == "auto" {
                    return format!("{mut_kw} {var_name} = {rhs};");
                }
                let r_ty = map_c_type(&c_ty, mode);
                return format!("{mut_kw} {var_name}: {r_ty} = {rhs};");
            }
        }
    }

    // 7. `push_back(` -> `push(`
    if line.contains(".push_back(") {
        return line.replace(".push_back(", ".push(");
    }

    line.to_string()
}

fn convert_printf(inner: &str) -> String {
    let mut parts = inner.splitn(2, ',');
    let fmt_raw = parts.next().unwrap_or("\"\"").trim();
    let args = parts.next().map(|s| s.trim());

    // Converter especificadores C (%d, %i, %u, %f, %s, %ld, %zu) para `{}` do Rust
    let mut rust_fmt = fmt_raw
        .replace("\\n\"", "\"")
        .replace("%d", "{}")
        .replace("%i", "{}")
        .replace("%u", "{}")
        .replace("%ld", "{}")
        .replace("%zu", "{}")
        .replace("%f", "{}")
        .replace("%s", "{}");
    if rust_fmt.is_empty() {
        rust_fmt = "\"\"".to_string();
    }

    match args {
        Some(a) if !a.is_empty() => format!("println!({rust_fmt}, {a});"),
        _ => format!("println!({rust_fmt});"),
    }
}

fn convert_cout(no_semi: &str) -> String {
    let segments: Vec<&str> = no_semi
        .split("<<")
        .skip(1)
        .map(|s| s.trim())
        .filter(|&s| s != "std::endl" && s != "endl" && s != "\"\\n\"")
        .collect();

    if segments.len() == 1 && segments[0].starts_with('"') {
        return format!("println!({});", segments[0]);
    }
    let placeholders = vec!["{}"; segments.len()].join("");
    format!("println!(\"{placeholders}\", {});", segments.join(", "))
}

pub fn map_c_type(c_type: &str, mode: TranslateMode) -> String {
    let t = c_type
        .trim()
        .trim_start_matches("struct ")
        .trim_start_matches("class ")
        .trim();

    // Templates C++: std::vector<T>, std::unique_ptr<T>, std::shared_ptr<T>, std::optional<T>
    if let Some(inner) = t
        .strip_prefix("std::vector<")
        .or_else(|| t.strip_prefix("vector<"))
        .and_then(|s| s.strip_suffix('>'))
    {
        return format!("Vec<{}>", map_c_type(inner, mode));
    }
    if let Some(inner) = t
        .strip_prefix("std::unique_ptr<")
        .and_then(|s| s.strip_suffix('>'))
    {
        return format!("Box<{}>", map_c_type(inner, mode));
    }
    if let Some(inner) = t
        .strip_prefix("std::shared_ptr<")
        .and_then(|s| s.strip_suffix('>'))
    {
        return format!("std::sync::Arc<{}>", map_c_type(inner, mode));
    }
    if let Some(inner) = t
        .strip_prefix("std::optional<")
        .and_then(|s| s.strip_suffix('>'))
    {
        return format!("Option<{}>", map_c_type(inner, mode));
    }

    // Ponteiros e referências
    if t == "const char*" || t == "const char *" {
        return match mode {
            TranslateMode::SafeRust => "&str".into(),
            TranslateMode::FfiBindings => "*const c_char".into(),
        };
    }
    if t == "char*" || t == "char *" {
        return match mode {
            TranslateMode::SafeRust => "String".into(),
            TranslateMode::FfiBindings => "*mut c_char".into(),
        };
    }
    if t == "void*" || t == "void *" {
        return "*mut c_void".into();
    }
    if t == "const void*" || t == "const void *" {
        return "*const c_void".into();
    }
    if let Some(base) = t.strip_suffix('*') {
        let base_clean = base.trim();
        if let Some(const_base) = base_clean.strip_prefix("const ") {
            let inner = map_c_type(const_base, mode);
            return match mode {
                TranslateMode::SafeRust => format!("&{inner}"),
                TranslateMode::FfiBindings => format!("*const {inner}"),
            };
        }
        let inner = map_c_type(base_clean, mode);
        return match mode {
            TranslateMode::SafeRust => format!("&mut {inner}"),
            TranslateMode::FfiBindings => format!("*mut {inner}"),
        };
    }
    if let Some(base) = t.strip_suffix('&') {
        let base_clean = base.trim();
        if let Some(const_base) = base_clean.strip_prefix("const ") {
            return format!("&{}", map_c_type(const_base, mode));
        }
        return format!("&mut {}", map_c_type(base_clean, mode));
    }

    match t {
        "int" | "signed int" | "int32_t" | "long" | "int32" => "i32".into(),
        "unsigned int" | "unsigned" | "uint32_t" | "unsigned long" | "DWORD" | "uint32" => "u32".into(),
        "short" | "int16_t" | "int16" => "i16".into(),
        "unsigned short" | "uint16_t" | "WORD" | "uint16" => "u16".into(),
        "char" | "int8_t" => "i8".into(),
        "unsigned char" | "uint8_t" | "BYTE" | "byte" => "u8".into(),
        "long long" | "int64_t" | "int64" => "i64".into(),
        "unsigned long long" | "uint64_t" | "QWORD" | "uint64" => "u64".into(),
        "size_t" | "uintptr_t" => "usize".into(),
        "ssize_t" | "ptrdiff_t" | "intptr_t" => "isize".into(),
        "float" => "f32".into(),
        "double" => "f64".into(),
        "bool" | "_Bool" | "BOOL" => "bool".into(),
        "std::string" | "string" => "String".into(),
        "std::string_view" | "string_view" => "&str".into(),
        "void" => "()".into(),
        other => other.to_string(),
    }
}

/// Cria um projeto Cargo completo com Cargo.toml, .gitignore e src/main.rs pronto para rodar `cargo run`.
pub fn generate_cargo_project(
    base_dir: &std::path::Path,
    project_name: &str,
    rust_code: &str,
) -> anyhow::Result<std::path::PathBuf> {
    let clean_name = project_name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect::<String>();
    let clean_name = if clean_name.is_empty() { "meu_projeto_rust".to_string() } else { clean_name };

    let proj_dir = base_dir.join("projetos_traduzidos").join(&clean_name);
    let src_dir = proj_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;

    let cargo_toml = format!(
        r#"[package]
name = "{clean_name}"
version = "0.1.0"
edition = "2021"
description = "Projeto gerado automaticamente pelo Tradutor C/C++ da RRS - Riva Rust Store"

[dependencies]
"#
    );
    std::fs::write(proj_dir.join("Cargo.toml"), cargo_toml)?;
    std::fs::write(proj_dir.join(".gitignore"), "/target\n**/*.rs.bk\n")?;

    let main_code = if rust_code.contains("fn main(") {
        rust_code.to_string()
    } else {
        format!(
            "{rust_code}\n\nfn main() {{\n    println!(\"✔ Código traduzido pela RRS executado com sucesso!\");\n}}\n"
        )
    };
    std::fs::write(src_dir.join("main.rs"), main_code)?;

    Ok(proj_dir)
}
