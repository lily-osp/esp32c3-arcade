# ESP32-C3 Retro Arcade Game Stack

Six tilt-controlled arcade games for the ESP32-C3 SuperMini, using a 128x64 SSD1306 I2C OLED display and an MPU-6050 accelerometer/gyroscope.

The repository includes two implementations:
1. **Rust (`rust/`)**: Bare-metal `#![no_std]` firmware using `esp-hal` 1.2.2 and `embedded-graphics` 0.8.1, reproducible via a Nix flake.
2. **Arduino C++**: Standalone sketches for each game plus a unified multi-game launcher, compatible with the Arduino IDE and `arduino-cli`.

---

## Games

| Game | Controls | Description |
|---|---|---|
| **Shooter** | Tilt vertical (MPU X) | Horizontal space shooter with auto-fire, double/triple shot power-ups, fire-rate bonuses, escalating enemy waves, and a multi-phase boss with an HP bar. |
| **Pong** | Tilt vertical (MPU X) | Single-player Pong against an adaptive AI paddle with dynamic deflection angles and 4 difficulty tiers. |
| **Snake** | Tilt 4-way (MPU X/Y) | Grid-based snake with edge wrapping, food spawning, progressive speed ramp, and self-collision checks. |
| **Tilt Racer** | Tilt horizontal (MPU Y) | 3-lane vertical survival racer with momentum-based steering, scrolling road lines, randomized traffic patterns, and invincibility respawns. |
| **Breakout** | Tilt horizontal (MPU Y) | Brick breaker with 32 multi-hit bricks, dynamic ball deflection, dropping power-ups (wide paddle, slow ball, extra life), and 4 difficulty tiers. |
| **Fall Down** | Tilt horizontal (MPU Y) | Vertical descent rolling a ball through gaps in rising floors, featuring gravity physics, collectible gems, hazard ceiling spikes, and 4 difficulty tiers. |
| **Random Arcade** | Shake to start | Unified multi-game launcher that cycles through all six games without repeating the previous game. |

---

## Hardware and pinout

The SSD1306 OLED and MPU-6050 share the default hardware I2C bus on GPIO9 (SDA) and GPIO8 (SCL) at 400 kHz:

| Peripheral | Pin | ESP32-C3 SuperMini | I2C Address |
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

## Rust implementation (`rust/`)

### Architecture
- **`#![no_std]` environment**: Zero dynamic heap allocations; all game state, entity buffers, and graphics targets use static memory.
- **I2C bus sharing**: Uses `embedded-hal-bus::i2c::RefCellDevice` to multiplex display rendering and accelerometer polling over I2C0.
- **PRNG**: In-crate XorShift32 seeded from accelerometer analog noise and startup timer ticks.
- **Flash footprint**:
  - Standalone binaries: ~115–125 KB each.
  - Multi-game launcher (`arcade`): 162 KB, occupying 3.92% of the 4 MB flash partition.

### Development environment

A Nix flake provides the pinned toolchain, RISC-V cross-compiler target, and flashing tools:

```sh
cd rust
nix develop
```

### Build and flash

Build release binaries:
```sh
cargo build --release
```

Flash directly to `/dev/ttyACM0`:
```sh
# Multi-game launcher
espflash flash --port /dev/ttyACM0 target/riscv32imc-unknown-none-elf/release/arcade

# Standalone game binaries
espflash flash --port /dev/ttyACM0 target/riscv32imc-unknown-none-elf/release/shooter
espflash flash --port /dev/ttyACM0 target/riscv32imc-unknown-none-elf/release/pong
espflash flash --port /dev/ttyACM0 target/riscv32imc-unknown-none-elf/release/snake
espflash flash --port /dev/ttyACM0 target/riscv32imc-unknown-none-elf/release/tilt-racer
espflash flash --port /dev/ttyACM0 target/riscv32imc-unknown-none-elf/release/breakout
espflash flash --port /dev/ttyACM0 target/riscv32imc-unknown-none-elf/release/falldown
```

---

## Arduino C++ reference

Each game is also provided as a standalone Arduino sketch:
- `shooter/shooter.ino`
- `pong/pong.ino`
- `snake/snake.ino`
- `tilt-racer/tilt-racer.ino`
- `breakout/breakout.ino`
- `falldown/falldown.ino`
- `random-arcade/random-arcade.ino`

Compile and flash with `arduino-cli`:
```sh
arduino-cli compile --fqbn esp32:esp32:esp32c3 random-arcade
arduino-cli upload --fqbn esp32:esp32:esp32c3 -p /dev/ttyACM0 random-arcade
```

---

## License

MIT License. Maintained under the [Lily-OSP](https://github.com/lily-osp) organization.
