# Oxide Motive

Oxide Motive is a Rust-based project designed for embedded systems development, providing a modular and extensible framework for interacting with hardware, implementing communication protocols, and running applications on both embedded devices and host systems.

## Features

*   **Modular Design**: Organized into several crates for clear separation of concerns.
*   **Hardware Abstraction**: `oxide-hal` provides a generic interface for various hardware platforms.
*   **Communication Protocol**: `oxide-protocol` defines the communication standards for the system.
*   **Embedded Firmware**: `oxide-firmware` contains the core logic for embedded applications.
*   **Host Applications**: `oxide-host` enables interaction with embedded devices from a host machine.
*   **Mathematical Utilities**: `oxide-math` offers common mathematical functions.
*   **Simulation Capabilities**: `oxide-sim` provides tools for simulating both MCU and host environments.
*   **Cross-platform Compatibility**: Supports various HALs (STM32H7, NXP S32K, Renesas RA) and host environments.

## Project Structure

The project is organized as a Rust workspace, with each directory representing a distinct crate:

*   `oxide-protocol/`: Defines the data structures and communication protocols used throughout the system.
*   `oxide-firmware/`: Contains the embedded firmware application logic, built on top of `oxide-hal` and `oxide-protocol`.
*   `oxide-host/`: Provides host-side applications and utilities for interacting with the embedded firmware.
*   `oxide-hal/`: A Hardware Abstraction Layer (HAL) crate, offering a unified interface to different microcontroller peripherals.
*   `oxide-math/`: A utility crate providing common mathematical operations and algorithms.
*   `oxide-sim/mcu/`: Simulation environment for the embedded microcontroller unit.
*   `oxide-sim/host/`: Simulation environment for the host-side applications.
*   `redox/`: (Potentially related to Redox OS, or another specific component - further details would be needed for a precise description).
*   `tests/`: Integration and unit tests for the various crates.
*   `scripts/`: Contains various utility scripts for development, building, or testing.

## Getting Started

To build the entire workspace, navigate to the project root and run:

```bash
cargo build --workspace
```

To run specific examples or tests, refer to the individual crate's documentation or examples.

### Building Firmware for a Specific HAL

To build the firmware for a specific HAL (e.g., `stm32h7`), you might use:

```bash
cargo build --workspace --features "stm32h7"
```

*(Note: Specific flashing or running instructions for embedded targets would depend on the hardware setup and toolchain.)*

## License

This project is licensed under the Apache License, Version 2.0. See the [LICENSE](LICENSE) file for details.
