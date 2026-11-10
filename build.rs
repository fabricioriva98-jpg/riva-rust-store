//! Embute o logo oficial (`assets/rrs.ico`) e os dados de versão no `.exe` do Windows.
//! O ícone aparece no Explorer, na barra de tarefas e nos atalhos criados pelo instalador
//! (que usam `IconLocation = <exe>,0`). Sem o Windows SDK (rc.exe), o build continua sem ícone.

fn main() {
    println!("cargo:rerun-if-changed=assets/rrs.ico");
    println!("cargo:rerun-if-changed=build.rs");
    #[cfg(windows)]
    {
        if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
            let mut res = winresource::WindowsResource::new();
            res.set_icon("assets/rrs.ico");
            res.set("ProductName", "RRS - Riva Rust Store");
            res.set("FileDescription", "RRS - Riva Rust Store");
            res.set("CompanyName", "Riva");
            res.set("LegalCopyright", "RRS - Riva Rust Store");
            if let Err(e) = res.compile() {
                println!("cargo:warning=ícone do .exe não embutido (rc.exe do Windows SDK indisponível?): {e}");
            }
        }
    }
}
