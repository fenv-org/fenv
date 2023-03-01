# fenv

`fenv` is a CLI tool that helps manage multiple versions of flutter SDKs in
your local machine. `fenv` does never require any other dependencies because
it is implemented with `bash` only.

This is a premature hobby project and currently work-in-progress.
This project is highly inspired by [pyenv][], which are mostly implemented with
`bash` scripts.

## Table of contents

- [fenv](#fenv)
  - [Table of contents](#table-of-contents)
  - [How to install](#how-to-install)
  - [How to use](#how-to-use)
  - [How to set local flutter SDK](#how-to-set-local-flutter-sdk)
  - [Trouble shootings](#trouble-shootings)
    - [If the `.flutter-version` file exists but not the corresponding flutter SDK isn't installed](#if-the-flutter-version-file-exists-but-not-the-corresponding-flutter-sdk-isnt-installed)
    - [When your VS Code could not find Dart PATH or Flutter SDK PATH](#when-your-vs-code-could-not-find-dart-path-or-flutter-sdk-path)

## How to install

1.  Execute the following command in your terminal:
    ```shell
    $ curl -sSL "https://raw.githubusercontent.com/powdream/fenv/main/init.sh" \
        | sh -
    ```
1.  If you install `fenv` for the first time in your machine, then, you will see
    instruction like:
    ```shell
    # Please execute the following command and following instructions if you have not setup `fenv` yet:

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
      # Add fenv executable to PATH by running
      # the following interactively:

      set -Ux FENV_ROOT $HOME/.fenv
      fish_add_path $FENV_ROOT/bin

      # Load fenv automatically by appending
      # the following to ~/.config/fish/conf.d/fenv.fish:

      fenv init - | source


      # Restart your shell for the changes to take effect.

      exec $SHELL -l
      ```
1.  Remove `FLUTTER_HOME`, `FLUTTER_SDK` environmental variables if exist.
1.  Eliminate any existing `<FLUTTER_SDK>/bin` from your `PATH`.

## How to use

1.  List-up available flutter SDKs.
    ```shell
    $ fenv install -l
    ```
1.  Install any flutter SDK you want.
    ```shell
    $ fenv install <version>
    $ fenv versions
    <version>
    ```
1.  Select the download SDK as the globally-using flutter SDK.
    ```shell
    $ fenv global <version>
    $ fenv version
    <version> (set by $HOME/.fenv/version)
    ```
1.  Test if `flutter` is working correctly:
    ```shell
    $ flutter --version
    $ fenv which flutter
    ```

## How to set local flutter SDK

1.  CD to your flutter project directory and specify local version:
    ```shell
    $ cd PROJECT
    $ fenv local <local-version>
    ```
1.  `fenv` will generate the following two files `.flutter-version` and
    `.flutter` symbolic link, which is a link to
    `$HOME/.fenv/versions/<local-version>`.
1.  Test if `flutter` is working correctly:
    ```shell
    $ fenv version
    $ fenv which flutter
    $ flutter --version
    ```
1.  We recommend staging `.flutter-version` to VCS, but not `.flutter`.
1.  After change your local flutter version, do `flutter clean` and re-launch
    VS code.

## Trouble shootings

### If the `.flutter-version` file exists but not the corresponding flutter SDK isn't installed

1.  If you run into an error message like:
    ```shell
    $ flutter
    fenv: no installed versions match the prefix '{VERSION}': try to execute 'fenv install && fenv local --symlink'
    ```
1.  Then, following the instruction, please execute:
    ```shell
    $ fenv install
    $ fenv local --symlink
    ```
    or
    ```shell
    $ fenv install && fenv local --symlink
    ```

### When your VS Code could not find Dart PATH or Flutter SDK PATH

- If you are using `fenv local`, please check if `.flutter` symlink is
  installed.
  If `.flutter` is not installed yet, `fenv` will let you know
  you need to execute `fenv local --symlink[-s]`.
- If you are using `fenv global`, please run `flutter pub get` once and
  re-launch VS code.

[pyenv]: https://github.com/pyenv/pyenv
