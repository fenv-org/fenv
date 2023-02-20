# fenv

`fenv` is a CLI tool that helps manage multiple versions of flutter SDKs in
your local machine. `fenv` does never require any other dependencies because
it is implemented with `bash` only.

This is a premature hobby project and currently work-in-progress.
This project is highly inspired by [pyenv][], which are mostly implemented with
`bash` scripts.

## How to install

1.  Execute the following command in your terminal:

```shell
$ curl -sSL "https://raw.githubusercontent.com/powdream/fenv/main/init.sh" \
    | sh -
```

2.  Then, you will see instruction like:

```shell
# Please execute the following command and folling instructions:

$HOME/.fenv/bin/fenv init
```

1.  Execute `$HOME/.fenv/bin/fenv init` then follow the next instructions:
    - zsh:

      ```shell
      # Load fenv automatically by appending
      # the following to
      ~/.zprofile (for login shells)
      and ~/.zshrc (for interactive shells) :

      export FENV_ROOT="$HOME/.fenv"
      command -v fenv >/dev/null || export PATH="$FENV_ROOT/bin:$PATH"
      eval "$(fenv init -)"

      # Restart your shell for the changes to take effect.

      exec $SHELL -l
      ```
    - bash:
      ```shell
      # Load fenv automatically by appending``
      # the following to
      ~/.bash_profile if it exists, otherwise ~/.profile (for login shells)
      and ~/.bashrc (for interactive shells) :

      export FENV_ROOT="$HOME/.fenv"
      command -v fenv >/dev/null || export PATH="$FENV_ROOT/bin:$PATH"
      eval "$(fenv init -)"

      # Restart your shell for the changes to take effect.

      exec $SHELL -l
      ```
    - fish:
      ```shell
      ````````````````````

[pyenv]: https://github.com/pyenv/pyenv
