#!/usr/bin/env sh

download_destination=$(mktemp -d)
git clone https://github.com/powdream/fenv "$download_destination"
pushd "$download_destination"> /dev/null
./setup_fenv.sh
popd > /dev/null
