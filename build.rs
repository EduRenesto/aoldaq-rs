fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let enum_cfg = cbindgen::EnumConfig {
        rename_variants: Some(cbindgen::RenameRule::ScreamingSnakeCase),
        prefix_with_name: true,
        ..cbindgen::EnumConfig::default()
    };

    let ccfg = cbindgen::Config {
        enumeration: enum_cfg,
        ..cbindgen::Config::default()
    };

    // Generate the C header
    cbindgen::Builder::new()
        .with_config(ccfg)
        .with_crate(crate_dir)
        .with_language(cbindgen::Language::C)
        .with_include_guard("AOLDAQ_H")
        .exclude_item("nifpga.rs")
        .rename_item("Aoldaq", "aoldaq_t")
        .rename_item("AoldaqArgs", "aoldaq_args_t")
        .rename_item("AoldaqMode", "aoldaq_mode")
        .generate()
        .expect("Failed to generate public C API")
        .write_to_file("aoldaq.h");

    // Generate the NiFpga bindings
    let nifpga = bindgen::Builder::default()
        .header("NiFpga.h")
        .generate()
        .expect("Failed to generate NiFpga Rust bindings");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    nifpga
        .write_to_file(out_path.join("nifpga.rs"))
        .expect("Failed to write to nifpga.rs");

    // Compile the NiFpga.c file and link to it
    cc::Build::new()
        .file("NiFpga.c")
        .include("src")
        .compile("nifpga");

    println!("cargo:rustc-link-search={}", out_path.to_str().unwrap());
    println!("cargo:rustc-link-lib=nifpga");
}
