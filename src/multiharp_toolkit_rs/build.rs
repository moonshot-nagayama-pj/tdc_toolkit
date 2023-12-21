use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");
    let os = env::consts::OS;

    if os == "linux" {
        // this requires the MHLib_v3.x.x_64bit directory to be in the same directory as the Cargo.toml file
        let lib_dir = format!(
            "{}/MHLib_v3.1.0.0_64bit/library",
            env::var("CARGO_MANIFEST_DIR").unwrap()
        );
        println!("cargo:rustc-link-search={}", lib_dir);
        println!("cargo:rustc-link-lib=dylib=mhlib");
        println!("cargo:rustc-link-arg=-Wl,-rpath={}", lib_dir);
    }

    if os == "windows" {
        println!("cargo:rustc-link-search=native=C:\\Program Files\\PicoQuant\\MultiHarp-MHLibv31");
        println!("cargo:rustc-link-lib=dylib=mhlib64");
    }

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
