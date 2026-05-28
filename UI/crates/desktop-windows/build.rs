use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=resources/app.rc");
    println!("cargo:rerun-if-changed=build.rs");

    let target = env::var("TARGET").unwrap_or_default();
    if !target.contains("windows") {
        return;
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let rc_path = manifest_dir.join("resources").join("app.rc");
    if !rc_path.exists() {
        return;
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let res_path = out_dir.join("app.res");

    let compiled = if target.contains("msvc") {
        Command::new("rc.exe")
            .arg("/nologo")
            .arg("/fo")
            .arg(&res_path)
            .arg(&rc_path)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    } else {
        let coff_path = out_dir.join("app.o");
        Command::new("windres")
            .arg("--input")
            .arg(&rc_path)
            .arg("--output")
            .arg(&coff_path)
            .arg("--output-format=coff")
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    };

    if compiled {
        println!("cargo:rustc-link-search=native={}", out_dir.display());
        if target.contains("msvc") {
            println!("cargo:rustc-link-arg-bins=/RES:{}", res_path.display());
        } else {
            println!("cargo:rustc-link-arg-bins={}/app.o", out_dir.display());
        }
    }
}
