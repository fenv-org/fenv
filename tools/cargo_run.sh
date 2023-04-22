#!/usr/bin/env bash

mkdir -p ~/temp/pwd
mkdir -p ~/temp/fenv
FENV_ROOT=~/temp/fenv/ FENV_DIR=~/temp/pwd/ cargo run -- $@
