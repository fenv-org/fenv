#!/usr/bin/env bash

cargo clean
cargo build --release --locked
cargo test --release --locked

artifact=target/release/fenv

if [[ -z "$FENV_ROOT" ]]; then
  fenv_home=$HOME/.fenv
else
  fenv_home="${FENV_ROOT%/}"
fi

if ! [[ -d "$fenv_home/bin" ]]; then
  mkdir -p "$fenv_home/bin"
fi
cp $artifact $fenv_home/bin/fenv

