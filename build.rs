use flate2::read::GzDecoder;
use std::{error, fs::File, io::BufReader, path::Path};
use tar::Archive;

fn untar_libs(to_path: &Path) -> Result<(), Box<dyn error::Error>> {
    let os = std::env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not found");
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH not found");

    let path_as_str = format!("sqlite-vss/sqlite-vss-{}-{}.tar.gz", os, arch);
    let archive_path = Path::new(&path_as_str);

    if !archive_path.exists() {
        panic!("Unsupported platform.");
    }

    let archive = File::open(archive_path)?;
    let buf_reader = BufReader::new(archive);
    let decoder = GzDecoder::new(buf_reader);
    let mut archive = Archive::new(decoder);

    archive.unpack(to_path)?;
    Ok(())
}

const SQLITE_LIBS_PATH: &str = "lib/sqlite-vss";

fn main() -> Result<(), Box<dyn error::Error>> {
    let lib_path = Path::new(SQLITE_LIBS_PATH);

    if !lib_path.exists() {
        untar_libs(lib_path)?;
    }

    // OpenMP does not seem like a commonly installed library, so let's
    // just statically link it.
    let openmp_lib = std::env::var("DEP_OPENMP_LIB").expect("Must have OpenMP in path");
    println!("cargo:rustc-link-search=native={}", openmp_lib);
    println!("cargo:rustc-link-lib=static=omp");

    println!(
        "cargo:rustc-link-search=native={}",
        lib_path.to_string_lossy()
    );
    println!("cargo:rustc-link-lib=static=faiss_avx2");
    println!("cargo:rustc-link-lib=static=sqlite_vector0");
    println!("cargo:rustc-link-lib=static=sqlite_vss0");

    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-arg=-Wl,-undefined,dynamic_lookup");
    } else if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=dylib=gomp");
        println!("cargo:rustc-link-lib=dylib=atlas");
        println!("cargo:rustc-link-lib=dylib=blas");
        println!("cargo:rustc-link-lib=dylib=lapack");
        println!("cargo:rustc-link-lib=dylib=m");
        println!("cargo:rustc-link-lib=dylib=stdc++");
    }

    Ok(())
}
