use std::env;
use std::env::VarError;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

// TODO write idiomatic, clear Rust

fn main() {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    if !(arch == "x86_64" && (os == "linux" || os == "windows")) || ! cfg!(feature = "multiharp") {
        // cannot link to the driver library on non-x64 architectures,
        // just use the stub
        println!("cargo::warning=Using the stub driver implementation.");
        return;
    }

    let mut include_dir = String::from("not_found");
    let mut lib_dir = String::from("not_found");
    let mut lib_dir_path: PathBuf;
    if os == "linux" && arch == "x86_64" {
        // locate the lib dir or download the files if necessary
        match env::var("MHLIB_LIB_DIR") {
            Ok(val) => {
                // the environment variable takes priority over all else
                lib_dir_path = PathBuf::from(val);
                assert!(
                    lib_dir_path.exists(),
                    "MHLIB_LIB_DIR was set, but the path does not seem to exist. Don't know what to do, exiting. Value was: {}",
                    lib_dir_path.display()
                );
                println!(
                    "cargo::warning=Using the value of the MHLIB_LIB_DIR environment variable to find the Multiharp shared library."
                );
            }
            Err(e) => match e {
                VarError::NotPresent => {
                    // Check the directory holding the Cargo.toml file to see if
                    // someone manually placed the library there.
                    let manifest_dir =
                        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
                    lib_dir_path =
                        PathBuf::from(manifest_dir.clone()).join("MHLib_v3.1.0.0_64bit/library");

                    if lib_dir_path.exists() {
                        println!(
                            "cargo::warning=Using an existing copy of the Multiharp shared library in CARGO_MANIFEST_DIR {manifest_dir}"
                        );
                    } else {
                        // Check to see if it was installed systemwide.
                        lib_dir_path = PathBuf::from("/usr/local/lib/mh150");
                        if lib_dir_path.exists() {
                            println!(
                                "cargo::warning=Using the system installation of the Multiharp shared library."
                            );
                        } else {
                            // Download it ourselves.
                            Command::new(
                                PathBuf::from(manifest_dir.clone())
                                    .join("install_mhlib.sh")
                                    .as_os_str(),
                            )
                            .output()
                            .expect("mhlib download failed");
                            lib_dir_path = PathBuf::from(manifest_dir.clone())
                                .join("MHLib_v3.1.0.0_64bit/library");
                            assert!(
                                lib_dir_path.exists(),
                                "Could not find a copy of the Multiharp library. Attempted to download the Multiharp library and failed. Don't know what to do. Exiting.",
                            );
                            println!(
                                "cargo::warning=Downloaded a new copy of the Multiharp shared library and placed it in CARGO_MANIFEST_DIR {manifest_dir}."
                            );
                        }
                    }
                }
                unknown_error => {
                    panic!(
                        "Unknown error occurred trying to read the MHLIB_PATH environment variable, are you sure this is Linux? Error: {unknown_error}"
                    );
                }
            },
        }

        lib_dir = lib_dir_path.to_string_lossy().into_owned();
        if lib_dir_path.join("mhlib.h").exists() {
            println!(
                "cargo::warning=Found Multiharp header files in the same directory as the shared library. Using them."
            );
            include_dir = lib_dir.clone();
        } else {
            // Most likely someone decided to properly separate header
            // files into the system include directory.
            let include_dir_path = Path::new("/usr/local/include/mh150");
            assert!(
                include_dir_path.exists(),
                "Could not find header files in the lib_dir {}, tried to find them in the include_dir {} instead but this directory also appears to be missing. Don't know what to do; exiting.",
                lib_dir_path.display(),
                include_dir_path.display(),
            );
            println!(
                "cargo::warning=Did not find Multiharp header files in the same directory as the shared library. Using a separate include directory."
            );
            include_dir = include_dir_path.to_string_lossy().into_owned();
        }

        println!("cargo::rustc-link-search=native={}", lib_dir);
        println!("cargo::rustc-link-lib=dylib=mhlib");
        println!("cargo::rustc-link-arg=-Wl,-rpath={}", lib_dir);
    } else if os == "windows" && arch == "x86_64" {
        include_dir = String::from("C:\\Program Files\\PicoQuant\\MultiHarp-MHLibv31");
        lib_dir = include_dir.clone();

        assert!(
            Path::new(&include_dir).exists(),
            "Include and library directory does not exist: {}",
            include_dir
        );

        println!("cargo::rustc-link-search=native={}", lib_dir);
        println!("cargo::rustc-link-lib=dylib=mhlib64");
    }

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        .clang_arg(format!("-I{}", include_dir))
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

    println!("cargo::rerun-if-changed=wrapper.h");
    println!("cargo::rerun-if-changed={include_dir}");
    println!("cargo::rerun-if-changed={lib_dir}");

    println!("cargo::warning=Value of include_dir: {include_dir}");
    println!("cargo::warning=Value of lib_dir: {lib_dir}");
}
