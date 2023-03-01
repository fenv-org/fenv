#!/usr/bin/env bash

set -e

if [[ "$1" = "--debug" ]]; then
  export FENV_DEBUG=1
  shift
fi

if [[ -n "$FENV_DEBUG" ]]; then
  # https://wiki-dev.bash-hackers.org/scripting/debuggingtips#making_xtrace_more_useful
  export PS4='+(${BASH_SOURCE}:${LINENO}): ${FUNCNAME[0]:+${FUNCNAME[0]}(): }'
  set -x
fi

if [[ -z "$FENV_ROOT" ]]; then
  fenv_home=$HOME/.fenv
else
  fenv_home="${FENV_ROOT%/}"
fi

mkdir -p $fenv_home/bin
for command in $(ls -1 fenv*); do
  cp "$command" "$fenv_home/bin"
  chmod a+x "$fenv_home/bin/$command"
done

mkdir -p "$fenv_home/"{shims,versions}
for command in $(ls -1 shims); do
  cp "shims/$command" "$fenv_home/shims"
  chmod a+x "$fenv_home/shims/$command"
done

set +e
is_in_path="$(env | grep ^PATH= | grep $fenv_home/bin)"
set -e
{
  echo '# Please execute the following command and following instructions if you have not setup `fenv` yet:'
  echo ''
  echo "$fenv_home/bin/fenv init"
  echo ''
} >&2
