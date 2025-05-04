#!/usr/bin/env just --justfile
export CARGO_TARGET_DIR := env_var_or_default('CARGO_TARGET_DIR', justfile_directory() + "/target")

ares := "/Applications/ares.app/Contents/MacOS/ares"
mgba := "mgba"

gba:
  (cd bin-gba && cargo run --release)  

gba-dbg:
  (cd bin-gba && cargo build --release && mgba "$CARGO_TARGET_DIR/thumbv4t-none-eabi/release/bin-gba")

desktop:
  (cargo run --bin bin-desktop)  


ares $bin:
  mkdir -p "$CARGO_TARGET_DIR/gba-out"
  agb-gbafix -o ./target/gba-out/game.gba "$bin"
  "{{ares}}" --system 'Game Boy Advance' "$CARGO_TARGET_DIR/gba-out/game.gba"

mgba $bin:
  "{{mgba}}" -C logToStdout=1 -C logLevel.gba.debug=127 "$bin"