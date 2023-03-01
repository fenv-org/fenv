#!/usr/bin/env sh

download_destination=$(mktemp -d)
latest_version="$(git ls-remote --tags https://github.com/powdream/fenv \
                  | grep 'refs/tags/v[0-9]\+.[0-9]\+.[0-9]\+$' \
                  | sed "s/^.*tags\///g" \
                  | sort --version-sort \
                  | tail -n 1)"
git clone \
  -c advice.detachedHead=false \
  -b "$latest_version" \
  https://github.com/powdream/fenv \
  "$download_destination"
pushd "$download_destination" > /dev/null
./setup_fenv.sh
popd > /dev/null
rm -rf "$download_destination" > /dev/null
