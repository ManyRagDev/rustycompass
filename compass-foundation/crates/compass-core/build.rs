use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn collect(path: &Path, files: &mut Vec<PathBuf>) {
    if path.is_dir() {
        for entry in fs::read_dir(path).expect("read source directory") {
            collect(&entry.expect("source entry").path(), files);
        }
    } else if path.is_file() {
        files.push(path.to_owned());
    }
}

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let root = manifest.parent().unwrap().parent().unwrap();
    let mut files = Vec::new();
    for name in [
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        "crates",
        "research/contracts",
    ] {
        let path = root.join(name);
        println!("cargo:rerun-if-changed={}", path.display());
        collect(&path, &mut files);
    }
    for name in [
        "CONTRATO_CANDIDATO.md",
        "schemas/report.schema.json",
        "schemas/source.schema.json",
        "schemas/metric-policy.schema.json",
        "schemas/limits.schema.json",
        "schemas/error.schema.json",
        "fixtures/metric-policy.json",
    ] {
        let path = root.join("research/proposals/d0-c2").join(name);
        println!("cargo:rerun-if-changed={}", path.display());
        files.push(path);
    }
    let rd = root.join("docs/entregas/rd/CONTRATO-RD-V0.1.md");
    println!("cargo:rerun-if-changed={}", rd.display());
    files.push(rd);
    let read = root.join("docs/entregas/rd/CONTRATO-RD-LEITURA-V0.1.md");
    println!("cargo:rerun-if-changed={}", read.display());
    files.push(read);
    files.sort();
    let mut hash = Sha256::new();
    hash.update(b"compass-build-v1\0");
    for path in files {
        let relative = path
            .strip_prefix(root)
            .unwrap()
            .to_str()
            .expect("source path UTF-8");
        let bytes = fs::read(&path).expect("read build input");
        hash.update((relative.len() as u64).to_le_bytes());
        hash.update(relative.as_bytes());
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    }
    println!(
        "cargo:rustc-env=COMPASS_BUILD_ID=sha256:{:x}",
        hash.finalize()
    );
}
