# ESP32-C3 Retro Arcade Game Stack

A high-performance embedded retro game suite designed for the **ESP32-C3 SuperMini**, **SSD1306 OLED (128x64)**, and **MPU6050 6-Axis IMU**.

This repository contains both:
1. **Pure `no_std` Rust Implementations (`rust/`)**: Highly optimized bare-metal RISC-V firmware built with `esp-hal` v1.2.2 and `embedded-graphics` v0.8.1 in a reproducible Nix flake environment.
2. **Original Arduino C++ Implementations**: Battle-tested reference sketches for Arduino IDE / `arduino-cli`.

---

## 🕹️ The Games

| Game | Controls | Description |
|---|---|---|
| **Shooter** | Tilt vertical (MPU X) | Horizontal space shooter with auto-fire, triple/double shot power-ups, fire-rate scaling, enemy waves, and a multi-phase boss with an HP bar. |
| **Pong** | Tilt vertical (MPU X) | Classic single-player Pong vs. adaptive AI with 4 difficulty levels (Easy, Medium, Hard, Extreme) and dynamic ball spin. |
| **Snake** | Tilt 4-way (MPU X/Y) | Direction-steered snake with border wrapping, food spawning, progressive speed ramp, and self-collision detection. |
| **Tilt Racer** | Tilt horizontal (MPU Y) | 3-lane vertical highway racer with momentum steering, scrolling road markers, oncoming traffic obstacles, and invincibility frames. |
| **Random Arcade** | Shake to start | Unified multi-game arcade launcher that seamlessly rotates across all 4 games without repeating the previous game. |

---

## ⚡ Hardware Pinout

Both the SSD1306 OLED and MPU6050 IMU share a single fast I2C bus on GPIO9 (SDA) and GPIO8 (SCL):

| Peripheral | Pin | ESP32-C3 SuperMini | Bus Address |
|---|---|---|---|
| **SSD1306 OLED (128x64)** | VCC | 3.3V | — |
| | GND | GND | — |
| | SDA | GPIO9 | `0x3C` |
| | SCL | GPIO8 | `0x3C` |
| **MPU-6050 (6-Axis IMU)** | VCC | 3.3V | — |
| | GND | GND | — |
| | SDA | GPIO9 | `0x68` |
| | SCL | GPIO8 | `0x68` |

---

## 🦀 Rust Embedded Stack (`rust/`)

### Key Architecture Features
- **Pure `no_std` & Zero Dynamic Allocations**: No heap allocator needed; deterministic frame pacing (~50 FPS) with static buffers.
- **Shared I2C Bus Arbitration**: `embedded-hal-bus::i2c::RefCellDevice` multiplexing display communication with sensor reading on I2C0.
- **Fast PRNG**: In-crate `Xorshift32` pseudo-random number generator seeded from accelerometer thermal noise and chip startup timer.
- **Footprint**:
  - Standalone binaries: ~115–124 KB (less than 3% of 4MB flash).
  - All-in-one Random Arcade (all 4 games combined): **133 KB** (only 3.2% of flash).

### Quickstart with Nix
```sh
cd rust
nix develop
```

### Build & Flash

Build all release binaries:
```sh
cargo build --release
```

Flash a specific target directly to `/dev/ttyACM0`:
```sh
# Flash all-in-one multi-game arcade
espflash flash --port /dev/ttyACM0 target/riscv32imc-unknown-none-elf/release/arcade

# Or flash standalone games
espflash flash --port /dev/ttyACM0 target/riscv32imc-unknown-none-elf/release/shooter
espflash flash --port /dev/ttyACM0 target/riscv32imc-unknown-none-elf/release/pong
espflash flash --port /dev/ttyACM0 target/riscv32imc-unknown-none-elf/release/snake
espflash flash --port /dev/ttyACM0 target/riscv32imc-unknown-none-elf/release/tilt-racer
```

---

## 🛠️ Arduino C++ Reference

Each game is also available in its standalone `.ino` sketch directory:
- `shooter/shooter.ino`
- `pong/pong.ino`
- `snake/snake.ino`
- `tilt-racer/tilt-racer.ino`
- `random-arcade/random-arcade.ino`

Compile with `arduino-cli`:
```sh
arduino-cli compile --fqbn esp32:esp32:esp32c3 random-arcade
arduino-cli upload --fqbn esp32:esp32:esp32c3 -p /dev/ttyACM0 random-arcade
```

---

## 📜 License

MIT License. Designed and maintained under the [Lily-OSP](https://github.com/lily-osp) organization.
