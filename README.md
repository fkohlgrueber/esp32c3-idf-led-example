# Rust: esp32c3 RGB LED example

This example for the esp32c3 uses [esp-idf](https://github.com/espressif/esp-idf) and the [RMT peripheral](https://docs.espressif.com/projects/esp-idf/en/latest/esp32c3/api-reference/peripherals/rmt.html) to drive the WS2812 RGB LED that's mounted on the respective dev boards (e.g. [ESP32-C3-DevKitM-1](https://docs.espressif.com/projects/esp-idf/en/latest/esp32c3/hw-reference/esp32c3/user-guide-devkitm-1.html)).

**[Demo video](https://www.youtube.com/watch?v=BOyUL12WjQs)**

The project is based on [ivmarkov/rust-esp32-std-hello](https://github.com/ivmarkov/rust-esp32-std-hello). The implementation of `LedStrip` (using the RMT peripheral through esp-idf to drive the LED) is inspired by the [respective implementation in esp-idf](https://github.com/espressif/esp-idf/tree/master/examples/common_components/led_strip).

## Preparation

- Install the nightly toolchain of Rust (necessary, because we utilize a few unstable Cargo features): `rustup toolchain install nightly`
- Make sure the toolchains are up to date, as one of the utilized unstable Cargo features landed just a few months ago: `rustup update`
- Switch to nightly (as per above, necessary for Cargo): `rustup override set nightly`
- The build is using the `ldproxy` linker wrapper from `embuild`, so install [ldproxy](https://crates.io/crates/embuild/ldproxy):
  - `cargo install ldproxy`
- For flashing and monitoring execution, I recommend `espflash` and `espmonitor` crates:
  - `cargo install espflash espmonitor`

## Build & Run

Simply build the project using cargo:

```
cargo build
```

Next, flash it (you might have to replace `/dev/ttyUSB0` with your respective port):

```
espflash /dev/ttyUSB0 target/riscv32imc-esp-espidf/debug/esp32c3-idf-led-example
```

If you want to see the output of the program:

```
espmonitor --speed 115200 /dev/ttyUSB0
```

