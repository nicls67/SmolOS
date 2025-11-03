//! This build script copies the `memory.x` file from the crate root into
//! a directory where the linker can always find it at build time.
//! For many projects this is optional, as the linker always searches the
//! project root directory -- wherever `Cargo.toml` is. However, if you
//! are using a workspace or have a more complicated build setup, this
//! build script becomes required. Additionally, by requesting that
//! Cargo re-run the build script whenever `memory.x` is changed,
//! updating `memory.x` ensures a rebuild of the application with the
//! new memory settings.
//!
//! The build script also sets the linker flags to tell it which link script to use.

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Generate drivers allocation file
    println!("cargo:rerun-if-changed=drivers_conf.yaml");
    let gen_status = Command::new("python3")
        .arg("tools/gen_drivers_alloc.py")
        .arg("drivers_conf.yaml")
        .output()
        .expect("Failed to execute Python script");

    if !gen_status.status.success() {
        panic!(
            "Drivers allocation generation failed: {}",
            String::from_utf8_lossy(&gen_status.stderr)
        );
    }

    // Build drivers lib
    println!("cargo:rerun-if-changed=drivers/Interface/Src/drivers_alloc.c");
    let build_status = Command::new("cmake")
        .current_dir("drivers")
        .arg("--build")
        .arg("--target")
        .arg("drivers")
        .arg("--preset")
        .arg("Debug")
        .output()
        .expect("Failed to build drivers");
    if !build_status.status.success() {
        panic!(
            "Drivers library build failed: {}",
            String::from_utf8_lossy(&gen_status.stderr)
        );
    }

    // Put `memory.x` in our output directory and ensure it's
    // on the linker search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    // By default, Cargo will re-run a build script whenever
    // any file in the project changes. By specifying `memory.x`
    // here, we ensure the build script is only re-run when
    // `memory.x` is changed.
    println!("cargo:rerun-if-changed=memory.x");

    // Specify linker arguments.

    // `--nmagic` is required if memory section addresses are not aligned to 0x10000,
    // for example the FLASH and RAM sections in your `memory.x`.
    // See https://github.com/rust-embedded/cortex-m-quickstart/pull/95
    println!("cargo:rustc-link-arg=--nmagic");

    // Set the linker script to the one provided by cortex-m-rt.
    println!("cargo:rustc-link-arg=-Tlink.x");

    println!("cargo:rustc-link-search=native=drivers/build/Release/");
    println!("cargo:rustc-link-lib=static=drivers");
}
