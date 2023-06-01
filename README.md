[![Rust](https://github.com/fenv-org/fenv/actions/workflows/rust.yml/badge.svg)](https://github.com/fenv-org/fenv/actions/workflows/rust.yml)
[![codecov](https://codecov.io/github/fenv-org/fenv/branch/main/graph/badge.svg?token=VPDI3URNT0)](https://codecov.io/github/fenv-org/fenv)

---

# fenv

`fenv` is a CLI tool that helps manage multiple versions of flutter SDKs in your
local machine. `fenv` does never require any other dependencies because it
consists of a single static-linking executable.

This is a hobby project, which is highly inspired by [pyenv][pyenv]. Any kinds
of feedbacks are welcome.

## Table of contents

- [fenv](#fenv)
  - [Table of contents](#table-of-contents)
  - [Supported OS and CPU architecture](#supported-os-and-cpu-architecture)
  - [How to install](#how-to-install)
  - [How to install the specific version of `fenv`](#how-to-install-the-specific-version-of-fenv)
  - [How to use](#how-to-use)
  - [How to set local flutter SDK](#how-to-set-local-flutter-sdk)
  - [Trouble shootings](#trouble-shootings)
    - [If `fenv init` and `fenv init -` misunderstand your shell](#if-fenv-init-and-fenv-init---misunderstand-your-shell)
    - [If the `.flutter-version` file exists but not the corresponding flutter SDK isn't installed](#if-the-flutter-version-file-exists-but-not-the-corresponding-flutter-sdk-isnt-installed)
    - [If IDE could not find Flutter SDK path and Dart path correctly](#if-ide-could-not-find-flutter-sdk-path-and-dart-path-correctly)
    - [If Dart-based CLI tools (such as `melos`) does not work well after switching Flutter SDK](#if-dart-based-cli-tools-such-as-melos-does-not-work-well-after-switching-flutter-sdk)

## Supported OS and CPU architecture

- Linux x86_64
- Linux aarch64
- MacOS x86_64
- MacOS aarch64

## How to install

1. Execute the following command in your terminal:
   ```shell
   $ curl -fsSL "https://fenv-install.jerry.company" \
       | bash
   ```
1. When the installation ends up, you will see instruction like:
   ```shell
   # Installation succeeds
   # Please execute the following command

   $HOME/.fenv/bin/fenv init

   # And follow the instructions if you have not setup `fenv` yet:
   ```
1. Execute `$HOME/.fenv/bin/fenv init` then follow the next instructions. `fenv`
   tries to show the different instructions guessing your shell.

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

1. Remove `FLUTTER_HOME`, `FLUTTER_SDK` environmental variables if exist.
1. Eliminate any existing `<FLUTTER_SDK>/bin` from your `PATH`.

## How to install the specific version of `fenv`

Specify the version tag explicitly like:

```shell
$ curl -fsSL "https://fenv-install.jerry.company" \
      | FENV_VERSION=vX.Y.Z bash
```

instead of:

```shell
$ curl -fsSL "https://fenv-install.jerry.company" \
      | bash
```

However, we don't support downloading the older versions than `v0.1.0` anymore.

## How to use

1. List-up available flutter SDKs.
   ```shell
   $ fenv install -l
   ```
1. Install any flutter SDK you want.
   ```shell
   $ fenv install <version>
   $ fenv versions
   <version>
   ```
1. Select the download SDK as the globally-using flutter SDK.
   ```shell
   $ fenv global <version>
   $ fenv version
   <version> (set by $HOME/.fenv/version)
   ```
1. Test if `flutter` is working correctly:
   ```shell
   $ flutter --version
   $ fenv which flutter
   ```
1. To see more usages, do `fenv [--help|-h]`

## How to set local flutter SDK

1. CD to your flutter project directory and specify local version:
   ```shell
   $ cd PROJECT
   $ fenv local <local-version>
   ```
1. `fenv` will generate the following two files `.flutter-version` and
   `.flutter` symbolic link, which is a link to
   `$HOME/.fenv/versions/<local-version>`.
1. Test if `flutter` is working correctly:
   ```shell
   $ fenv version
   $ fenv which flutter
   $ flutter --version
   ```
1. We recommend staging `.flutter-version` to VCS, but not `.flutter`.
1. After change your local flutter version, do `flutter clean` and re-launch VS
   code.
1. If you are using IDEs such as Visual Studio Code, IntelliJ IDEA, and Android
   Studio, we recommend to run `fenv workspace .` and re-launch the IDE after
   you change the sdk version. Since Flutter plugins for those IDEs caches the
   SDK path in their memory while running.

## Trouble shootings

### If `fenv init` and `fenv init -` misunderstand your shell

- You can explicitly specify your shell with `--shell` option:

  ```shell
  $ $HOME/.fenv/bin/fenv init [--shell|-s] [bash|zsh|fish|ksh]
  $ $HOME/.fenv/bin/fenv init - [--shell|-s] [bash|zsh|fish|ksh]
  ```

  You can omit `$HOME/.fenv/bin` if you already add the path to your `$PATH`.

### If the `.flutter-version` file exists but not the corresponding flutter SDK isn't installed

1. If you run into an error message like:
   ```shell
   $ flutter
   fenv: no installed versions match the prefix '{VERSION}': try to execute 'fenv install && fenv local --symlink'
   ```
1. Then, following the instruction, please execute:
   ```shell
   $ fenv install
   $ fenv local --symlink
   ```
   or
   ```shell
   $ fenv install && fenv local --symlink
   ```

### If IDE could not find Flutter SDK path and Dart path correctly

1. Run the following command on your flutter workspace root. The root must be
   the directory that contains `pubspec.yaml` file.

   ```shell
   $ cd $PROJECT_ROOT
   $ fenv workspace .
   ```

   or

   ```shell
   $ fenv workspace $PROJECT_ROOT
   ```
1. Close IDE and re-open it. This is required because VS code and IntelliJ
   (Android Studio) try to look up Flutter SDK and Dart SDK when the IDE is
   opening only.
1. If this problem is not resolved, check whether the versions of `"flutter"` in
   `.dart_tool/package_config.json` and `lib/core` in
   `.idea/libraries/Dart_SDK.xml` are correctly set.

### If Dart-based CLI tools (such as `melos`) does not work well after switching Flutter SDK

Dart-based CLI tools, for example `melos` and `vector_graphics_compiler`, may
show a warning like
_`"Can't load Kernel binary: Invalid kernel binary format version."`_ after
switching Flutter SDK.

This is not the problem of `fenv` but a intrinsic problem of Dart-based CLI
tools. You can do activate the CLI once more to suppress these warnings
messages.

For example, you can re-activate `melos` like:

```shell
$ flutter pub global activate melos
```

[pyenv]: https://github.com/pyenv/pyenv
