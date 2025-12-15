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

fn main() {
    // Run drivers allocator generation script
    let status = std::process::Command::new("sh")
        .arg("tools/build/gen_drivers_alloc.sh")
        .status()
        .expect("failed to execute tools/build/gen_drivers_alloc.sh");
    if !status.success() {
        panic!(
            "tools/build/gen_drivers_alloc.sh failed with exit status: {}",
            status
        );
    }

    // Run CMake configure script
    let status = std::process::Command::new("sh")
        .arg("tools/build/cmake_configure.sh")
        .status()
        .expect("failed to execute tools/build/cmake_configure.sh");
    if !status.success() {
        panic!(
            "tools/build/cmake_configure.sh failed with exit status: {}",
            status
        );
    }

    // Run libdrivers build script
    let status = std::process::Command::new("sh")
        .arg("tools/build/cmake_build.sh")
        .status()
        .expect("failed to execute tools/build/build_libdrivers.sh");
    if !status.success() {
        panic!(
            "tools/build/build_libdrivers.sh failed with exit status: {}",
            status
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
    //println!("cargo:rerun-if-changed=memory.x");

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
