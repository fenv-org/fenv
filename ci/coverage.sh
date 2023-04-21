#!bin/env bash

set -ex

function download_gcov() {
    bin_path="$HOME/.local/bin"
    mkdir -p $bin_path
    curl -sL https://github.com/mozilla/grcov/releases/download/v0.8.13/grcov-x86_64-unknown-linux-gnu.tar.bz2 \
        | tar jxf - -C "$bin_path"
    echo "$bin_path" >> $GITHUB_PATH
}

function run_test() {
    CARGO_INCREMENTAL=0 \
    RUSTFLAGS='-Cinstrument-coverage' \
    LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' \
    cross test --target $1
}

function gen_lcov_coverage_report() {
    mkdir -p coverage
    grcov . \
        --binary-path ./target/debug/deps/ \
        -s . \
        -t lcov \
        --branch \
        --ignore-not-existing \
        --ignore '../*' \
        --ignore "/*" \
        -o coverage/tests.lcov
}

function main() {
    download_gcov
    run_test $1
    gen_lcov_coverage_report
}

main $1
