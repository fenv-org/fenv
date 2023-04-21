#/bin/env bash

# This is a bash script that run tests and build code coverage reports
# from the test results.
#
# Referred to "https://blog.rng0.io/how-to-do-code-coverage-in-rust" article.

set -euox pipefail

function install_grcov() {
  if [[ -z "$(command -v grcov || true)" ]]; then
    local url=$(curl -L \
      https://api.github.com/repos/mozilla/grcov/releases/latest 2> /dev/null \
      | jq --raw-output \
        '.assets[] | { name, browser_download_url } | select(.name == "grcov-x86_64-apple-darwin.tar.bz2") | .browser_download_url')
    bin_path=$HOME/bin
    mkdir -p $bin_path
    curl -sL $url | tar jxf - -C "$bin_path"
    export PATH=$bin_path:$PATH
  fi
}

function run_test() {
  rm -f *.profraw
  CARGO_INCREMENTAL=0 \
  RUSTFLAGS='-Cinstrument-coverage' \
  LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' \
  cargo test
}

function gen_html_coverage_report() {
  mkdir -p coverage
  grcov . \
    --binary-path ./target/debug/deps/ \
    -s . \
    -t html \
    --branch \
    --ignore-not-existing \
    --ignore '../*' \
    --ignore "/*" \
    -o coverage/html
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
  install_grcov
  run_test
  gen_lcov_coverage_report
}

main
