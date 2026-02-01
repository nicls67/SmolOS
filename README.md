# SmolOS

SmolOS is a lightweight, hybrid Rust/C micro-kernel operating system designed for the **STM32F769I-DISCO** development board. It leverages Rust's safety for higher-level OS logic while utilizing optimized C drivers for low-level hardware interaction.

## Global Architecture

The project follows a micro-kernel architecture where core services and hardware abstractions are decoupled.

- **Kernel (`crates/kernel`)**: Manages system time, task scheduling, and core OS primitives.
- **HAL Interface (`crates/hal_interface`)**: Provides a type-safe Rust wrapper around the underlying C drivers. It implements a locking mechanism to ensure exclusive access to hardware resources.
- **Drivers (`drivers/`)**: Contains C drivers generated with STM32CubeMX. These are compiled into a static library (`libdrivers.a`) and linked into the final Rust binary.
- **Application (`crates/smolos`)**: The main entry point that initializes the HAL, configures the kernel, and starts the system.
- **Display (`crates/display`)**: A dedicated crate for managing display output.

### Driver Configuration and Generation

Hardware interfaces are defined in `config/drivers_conf.yaml`. A Python-based tool (`tools/gen_drivers_alloc`) processes this configuration to:
1. Generate C headers (`drivers/Interface/Inc/drivers_alloc.h`) for the driver library.
2. Generate Rust code (`crates/smolos/src/interrupts.rs`) to handle hardware interrupts and bridge them to the OS.

## Technical Stack

- **Languages**: Rust (edition 2024), C (C99/C11)
- **Toolchain**: `arm-none-eabi-gcc`, `rustup` (target `thumbv7em-none-eabihf`)
- **Build System**: Cargo (Rust) + CMake & Ninja (C)
- **Flashing & Debugging**: `probe-rs`
- **Scripting**: Python (with PyYAML)

## Project Structure

```text
.
├── config/             # Linker scripts (memory.x) and driver definitions (drivers_conf.yaml)
├── crates/             # Rust workspace members
│   ├── smolos/         # Main application entry point
│   ├── kernel/         # OS core logic
│   ├── hal_interface/  # Rust/C hardware abstraction bridge
│   └── display/        # Display management crate
├── drivers/            # C drivers and CMake build configuration
├── tools/              # Build scripts and code generation tools
└── Cargo.toml          # Workspace definition
```

## Getting Started

### Prerequisites

The project is primarily developed on Arch Linux. A helper script is provided to install all necessary dependencies:

```bash
./tools/install_dependencies.sh
```

This script installs:
- CMake, Ninja, and the ARM GNU Toolchain.
- Python and PyYAML.
- The Rust `thumbv7em-none-eabihf` target.
- `probe-rs` for flashing.

### Building the Project

The build process is fully integrated into Cargo. Running a build command in the `smolos` crate (or the workspace root) will automatically trigger the C driver build and code generation:

```bash
cargo build
```

The `build.rs` script in `crates/smolos` handles:
1. Running `gen_drivers_alloc` to update driver bindings.
2. Configuring and building the C driver library using CMake.
3. Linking the resulting static library and setting up the linker script.

### Running on Hardware

To flash the OS onto an STM32F769I-DISCO board and see the output:

```bash
cargo run
```

This uses `probe-rs` as defined in `.cargo/config.toml` to flash the binary and provide real-time logging.

## Development

- **Modifying Drivers**: If you change the C drivers, they will be recompiled during the next `cargo build`.
- **Adding Interfaces**: To add a new hardware interface (e.g., a new GPIO or UART), update `config/drivers_conf.yaml` and rebuild.
- **Kernel Changes**: Most kernel logic resides in `crates/kernel`.
