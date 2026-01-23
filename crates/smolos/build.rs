//! Build script for the `smolos` embedded application crate.
//!
//! When building inside a workspace, the linker may not find `memory.x` if it
//! only exists at the workspace root. This script copies `memory.x` from the
//! workspace root into `OUT_DIR` and adds that directory to the link search
//! path so `cortex-m-rt`'s `link.x` can include it.
//!
//! It also sets the required linker args (`--nmagic` and `-Tlink.x`) and links
//! the native `drivers` static library built by the workspace root build.
//!
//! Notes:
//! - This assumes the workspace root layout matches this repository:
//!   `SmolOS/memory.x` and `SmolOS/drivers/build/Release/libdrivers.a`.
//! - If you change those paths, update this file accordingly.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Ensure we are positioned in the SmolOS workspace root directory.
    let l_crate_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    let l_workspace_root = l_crate_dir
        .parent()
        .expect("smolos crate must live two levels under the workspace root")
        .parent()
        .expect("smolos crate must live two levels under the workspace root")
        .to_path_buf();
    env::set_current_dir(&l_workspace_root)
        .unwrap_or_else(|e| panic!("failed to set current dir to {:?}: {}", l_workspace_root, e));

    // Run drivers allocator generation script
    let l_drivers_conf = l_workspace_root.join("config").join("drivers_conf.yaml");
    println!("cargo:rerun-if-changed={}", l_drivers_conf.display());
    let l_gen_drivers_alloc_dir = l_workspace_root.join("tools").join("gen_drivers_alloc");
    println!(
        "cargo:rerun-if-changed={}",
        l_gen_drivers_alloc_dir.display()
    );
    let l_gen_status = std::process::Command::new("sh")
        .arg("tools/build/gen_drivers_alloc.sh")
        .status()
        .expect("failed to execute tools/build/gen_drivers_alloc.sh");
    if !l_gen_status.success() {
        panic!(
            "tools/build/gen_drivers_alloc.sh failed with exit status: {}",
            l_gen_status
        );
    }

    // Run CMake configure script
    let l_drivers_dir = l_workspace_root.join("drivers");
    println!("cargo:rerun-if-changed={}", l_drivers_dir.display());
    let l_config_status = std::process::Command::new("sh")
        .arg("tools/build/cmake_configure.sh")
        .status()
        .expect("failed to execute tools/build/cmake_configure.sh");
    if !l_config_status.success() {
        panic!(
            "tools/build/cmake_configure.sh failed with exit status: {}",
            l_config_status
        );
    }

    // Run libdrivers build script
    let l_build_status = std::process::Command::new("sh")
        .arg("tools/build/cmake_build.sh")
        .status()
        .expect("failed to execute tools/build/build_libdrivers.sh");
    if !l_build_status.success() {
        panic!(
            "tools/build/build_libdrivers.sh failed with exit status: {}",
            l_build_status
        );
    }

    // ---- Ensure the linker can find memory.x ----
    let l_memory_x_src = l_workspace_root.join("config").join("memory.x");
    if !l_memory_x_src.exists() {
        panic!(
            "Expected linker memory script at {:?}. \
             Make sure `memory.x` exists in config folder.",
            l_memory_x_src
        );
    }

    let l_out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR not set"));
    let l_memory_x_dst = l_out_dir.join("memory.x");

    fs::copy(&l_memory_x_src, &l_memory_x_dst).unwrap_or_else(|e| {
        panic!(
            "Failed to copy {:?} to {:?}: {}",
            l_memory_x_src, l_memory_x_dst, e
        )
    });

    // Add OUT_DIR to the linker search path so `link.x` can include `memory.x`.
    println!("cargo:rustc-link-search={}", l_out_dir.display());

    // Re-run when the memory layout changes.
    println!("cargo:rerun-if-changed={}", l_memory_x_src.display());

    // ---- Linker arguments required for cortex-m-rt embedded targets ----
    // `--nmagic` is required when memory regions are not aligned to 0x10000.
    println!("cargo:rustc-link-arg=--nmagic");

    // Use the linker script provided by cortex-m-rt (it includes `memory.x`).
    println!("cargo:rustc-link-arg=-Tlink.x");

    // ---- Link the native drivers static library (built elsewhere) ----
    // The library is expected at: workspace_root/drivers/build/Release/libdrivers.a
    // We add the directory to the native link search path and link `-ldrivers`.
    let l_drivers_lib_dir = l_workspace_root
        .join("drivers")
        .join("build")
        .join("Release");
    println!(
        "cargo:rustc-link-search=native={}",
        l_drivers_lib_dir.display()
    );
    println!("cargo:rustc-link-lib=static=drivers");
}
