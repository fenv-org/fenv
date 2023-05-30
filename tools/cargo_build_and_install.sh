#!/usr/bin/env bash

set -euox

cargo clean
cargo build --release --locked
# cargo test --release --locked

artifact=target/release/fenv

if [[ -z "$FENV_ROOT" ]]; then
  fenv_home=$HOME/.fenv
else
  fenv_home="${FENV_ROOT%/}"
fi

if ! [[ -d "$fenv_home/bin" ]]; then
  mkdir -p "$fenv_home/bin"
fi
if ! [[ -d "$fenv_home/shims" ]]; then
  mkdir -p "$fenv_home/shims"
fi
rm -f $fenv_home/bin/fenv* $fenv_home/shims/*

# copy the build `fenv` to `$fenv_home/bin`
cp $artifact $fenv_home/bin/fenv

# copy scripts in `shims` to `$fenv_home/shims`
cp ./shims/* $fenv_home/shims
chmod +x $fenv_home/shims/*
