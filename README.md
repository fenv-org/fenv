[![Rust](https://github.com/fenv-org/fenv/actions/workflows/rust.yml/badge.svg)](https://github.com/fenv-org/fenv/actions/workflows/rust.yml)
[![codecov](https://codecov.io/github/fenv-org/fenv/branch/main/graph/badge.svg?token=VPDI3URNT0)](https://codecov.io/github/fenv-org/fenv)

---

# fenv

`fenv` is a CLI tool that helps manage multiple versions of flutter SDKs in your
local machine. `fenv` does never require any other dependencies even `Dart`(‼️)
because it consists of a single static-linking executable.

This is a hobby project, which is highly inspired by [pyenv][pyenv]. Any kinds
of feedbacks are welcome.

## Table of contents

- [fenv](#fenv)
  - [Table of contents](#table-of-contents)
  - [fenv vs. FVM](#fenv-vs-fvm)
  - [Supported OS and CPU architecture](#supported-os-and-cpu-architecture)
  - [How to install **fenv**](#how-to-install-fenv)
    - [Install the latest version](#install-the-latest-version)
    - [Install an older version](#install-an-older-version)
  - [How to use](#how-to-use)
    - [List up all the available Flutter SDKs](#list-up-all-the-available-flutter-sdks)
    - [List up all the installed Flutter SDKs](#list-up-all-the-installed-flutter-sdks)
    - [Install the specific version of Flutter SDK](#install-the-specific-version-of-flutter-sdk)
    - [Install the latest snapshot of a **_channel_** Flutter SDK](#install-the-latest-snapshot-of-a-channel-flutter-sdk)
    - [How to specify the globally used Flutter SDK](#how-to-specify-the-globally-used-flutter-sdk)
    - [How to specify the locally used Flutter SDK](#how-to-specify-the-locally-used-flutter-sdk)
    - [See more help](#see-more-help)
  - [How to migrate](#how-to-migrate)
    - [From v0.0.x to v0.1.x](#from-v00x-to-v01x)
  - [Trouble shootings](#trouble-shootings)
    - [If `"fenv init"` and `"fenv init -"` misunderstand your shell](#if-fenv-init-and-fenv-init---misunderstand-your-shell)
    - [If the `.flutter-version` file exists but not the corresponding flutter SDK isn't installed](#if-the-flutter-version-file-exists-but-not-the-corresponding-flutter-sdk-isnt-installed)
    - [If IDE could not find Flutter SDK path and Dart path correctly](#if-ide-could-not-find-flutter-sdk-path-and-dart-path-correctly)
    - [If Dart-based CLI tools (such as `"melos"`) do not work well after switching Flutter SDK](#if-dart-based-cli-tools-such-as-melos-do-not-work-well-after-switching-flutter-sdk)

## fenv vs. FVM

`fenv` is the tool to solve the completely same problem that [FVM] attempts to
solve. However, `fenv` is born to address the weakness of [FVM].

|                                    | **fenv**               | [**FVM**][FVM]                                                                                                 |
| ---------------------------------- | ---------------------- | -------------------------------------------------------------------------------------------------------------- |
| _**How to run `flutter`**_         | `flutter pub get`      | `fvm flutter pub get` or<br/> `.fvm/flutter_sdk/bin/flutter pub get`                                           |
| _**How to run `dart`**_            | `dart pub get`         | `fvm dart pub get` or<br/> `.fvm/flutter_sdk/bin/dart pub get`                                                 |
| _**Generates a symlink**_          | **None**               | `.fvm/flutter_sdk` is generated                                                                                |
| _**Where to leave memo**_          | `.flutter-version`     | `.fvm/fvm_config.json`                                                                                         |
| _**What have to do for VS Code**_  | Need to do **nothing** | Need to set `dart.flutterSdkPath` to `.fvm/flutter_sdk` manually<br/>whenever switching a version              |
| _**What have to do for IntelliJ**_ | `fenv workspace .`     | Need to set Flutter SDK path and Dart SDK path to `.fvm/flutter_sdk` manually<br/>whenever switching a version |
| _**Supports "global" version**_    | `fenv global 3.10`     | Not supported                                                                                                  |
| _**supports "local" version**_     | `fenv local 3.10`      | `fvm use 3.10`                                                                                                 |

As you can see from the above table, `fenv` does neither require the annoying
`fvm` prefix to run a `flutter` command nor require manual setup of SDK paths
for IDEs. `fenv` is an out-of-the-box tool that is developed for professional
Flutter developers.

## Supported OS and CPU architecture

- Linux x86_64
- Linux aarch64
- MacOS x86_64
- MacOS aarch64

## How to install **fenv**

### Install the latest version

1. Execute the following command in your terminal:
   ```shell
   $ curl -fsSL "https://fenv-install.jerry.company" | bash
   ```
1. When the installation ends up, you will see instruction like:
   ```shell
   # Installation succeeds
   # Please execute the following command

   $HOME/.fenv/bin/fenv init

   # And follow the instructions if you have not setup `fenv` yet:
   ```
1. Execute `$HOME/.fenv/bin/fenv init` then follow the next instructions. `fenv`
   will suggest the different instructions guessing your shell.

   - zsh:
     ```shell
     # Load fenv automatically by appending the following to
     # ~/.zprofile (for login shells)
     # and ~/.zshrc (for interactive shells) :

     export FENV_ROOT="$HOME/.fenv"
     command -v fenv >/dev/null || export PATH="$FENV_ROOT/bin:$PATH"
     eval "$(fenv init -)"

     # Restart your shell for the changes to take effect:

     exec $SHELL -l
     ```
   - bash:
     ```shell
     # Load fenv automatically by appending the following to
     # ~/.bash_profile if it exists, otherwise ~/.profile (for login shells)
     # and ~/.bashrc (for interactive shells) :

     export FENV_ROOT="$HOME/.fenv"
     command -v fenv >/dev/null || export PATH="$FENV_ROOT/bin:$PATH"
     eval "$(fenv init -)"

     # Restart your shell for the changes to take effect:

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

     # Restart your shell for the changes to take effect:

     exec $SHELL -l
     ```
   - ksh:
     ```shell
     # Load fenv automatically by appending the following to
     # ~/.profile :

     export FENV_ROOT="$HOME/.fenv"
     command -v fenv >/dev/null || export PATH="$FENV_ROOT/bin:$PATH"
     eval "$(fenv init -)"

     # Restart your shell for the changes to take effect:

     exec $SHELL -l
     ```

1. Remove `FLUTTER_HOME`, `FLUTTER_SDK` environmental variables if exist.
1. Eliminate any existing `<FLUTTER_SDK>/bin` from your `PATH`.

### Install an older version

- You can specify the target version of **`fenv`** with the `FENV_VERSION`
  environment variable.

- Specify the version tag explicitly like:

  ```shell
  $ curl -fsSL "https://fenv-install.jerry.company" | FENV_VERSION=vX.Y.Z bash
  ```

  instead of:

  ```shell
  $ curl -fsSL "https://fenv-install.jerry.company" | bash
  ```

- The available releases can be found from the
  [release page](https://github.com/fenv-org/fenv/releases).

## How to use

### List up all the available Flutter SDKs

```shell
$ fenv list-remote
# or
$ fenv install --list # or -l
```

### List up all the installed Flutter SDKs

```shell
$ fenv list
# or
$ fenv versions
```

### Install the specific version of Flutter SDK

`fenv` supports to install the specific version.

```shell
# Install 3.10.0
$ fenv install 3.10.0
# Install the latest version of 3.7.x at the moment. It may install 3.7.12
$ fenv install 3.7
# Install the latest version of 2.x.y at the moment. It may install 2.10.5
$ fenv install 2
$ fenv versions
```

`fenv` does not permit to run `flutter upgrade`, `flutter downgrade`, and
`flutter channel` commands with the version Flutter SDK.

```shell
$ fenv local 3.10.0
$ fenv version
3.10.0 (set by `.../.flutter-version`)
$ flutter upgrade   # NG
fenv: `flutter upgrade` is not allowed. use `fenv install/uninstall` instead
$ flutter downgrade # NG
fenv: `flutter downgrade` is not allowed. use `fenv install/uninstall` instead
$ flutter channel   # NG
fenv: `flutter channel` is not allowed. use `fenv install/uninstall` instead
```

Nevertheless, you can execute those disallowed command like:

```shell
$ $FENV_ROOT/versions/3.10.0/bin/flutter upgrade
```

**HOWEVER, DON'T DO THAT BECAUSE `fenv` REGARDS THOSE SDKS ARE POLLUTED**.

### Install the latest snapshot of a **_channel_** Flutter SDK

`fenv` also supports to install the latest snapshot of `dev`, `master`, `beta`,
and `stable`.

```shell
$ fenv install stable # or s
$ fenv versions
```

`fenv` permits to run `flutter upgrade` and `flutter downgrade` with the channel
Flutter SDKs but not `flutter channel` command.

```shell
$ fenv local stable
$ fenv version
stable (set by `.../.flutter-version`)
$ flutter upgrade   # ok
...
$ flutter downgrade # ok
...
$ flutter channel   # NG
fenv: `flutter channel` is not allowed. use `fenv install/uninstall` instead
```

### How to specify the globally used Flutter SDK

```shell
$ fenv global stable
# Let's check
$ fenv global
stable
$ fenv version
stable (set by `$FENV_ROOT/version`)
$ fenv which flutter
$FENV_ROOT/versions/stable/bin/flutter
```

After switching Flutter version, do `flutter pub get` in your workspace root to
regenerate the `.dart_tool/package_json.config` file. For more information, see
also [here](#if-ide-could-not-find-flutter-sdk-path-and-dart-path-correctly).

### How to specify the locally used Flutter SDK

```shell
$ fenv global stable
$ cd my_dir
$ fenv local 3.10.0
# Let's check
$ fenv global
stable
$ fenv local
3.10.0
$ fenv version
3.10.0 (set by `.../my_dir/.flutter-version`)
$ cd more_deeper
$ fenv version
3.10.0 (set by `.../my_dir/.flutter-version`)
$ fenv which flutter
$FENV_ROOT/versions/3.10.0/bin/flutter
```

After switching Flutter version, do `flutter pub get` in your workspace root to
regenerate the `.dart_tool/package_json.config` file. For more information, see
also [here](#if-ide-could-not-find-flutter-sdk-path-and-dart-path-correctly).

### See more help

```shell
$ fenv --help # or -h
```

For each command:

```shell
$ fenv completions --help
$ fenv versions --help
```

## How to migrate

### From v0.0.x to v0.1.x

1. Remove the `.flutter` symlink from your Flutter workspace.
2. `flutter doctor` with the Flutter SDKs installed by `fenv` v0.0.x may show
   messages like:
   ```shell
   $ flutter doctor
   [✓] Flutter (Channel unknown, 3.0.0, on ...)
   ```
   or
   ```shell
   [!] Flutter (Channel unknown, 3.3.0, on ...)
      ! Flutter version 3.3.0 on channel unknown at $FENV_ROOT/versions/3.3.0
      ! Upstream repository unknown
   ```
   The _Channel_ of Flutters, which was installed by `fenv` v0.0.x, are
   `unknown`.

   To make channels `stable`, please uninstall and reinstall them with `fenv`
   v0.1.x or later one. Then, you cannot see those warning message anymore.
3. For VS code users only: Remove `dart.flutterSdkPath` from `settings.json` If
   you previously specified the Flutter SDK path by `dart.flutterSdkPath` in
   `settings.json` whatever a user setting or a workspace setting.

## Trouble shootings

### If `"fenv init"` and `"fenv init -"` misunderstand your shell

- You can explicitly specify your shell with `--shell` option:

  ```shell
  $ $HOME/.fenv/bin/fenv init [--shell|-s] [bash|zsh|fish|ksh]
  $ $HOME/.fenv/bin/fenv init - [--shell|-s] [bash|zsh|fish|ksh]
  ```

  You can omit `$HOME/.fenv/bin` if you already add the path to your `$PATH`.

### If the `.flutter-version` file exists but not the corresponding flutter SDK isn't installed

Run the following instruction:

```shell
$ fenv install
```

then, `fenv` will install the Flutter version specified in `.flutter-version`.

### If IDE could not find Flutter SDK path and Dart path correctly

To find the locations of Dart SDK and Flutter SDK, Dart and Flutter plugins for
IDEs such as Visual Studio Code and IntelliJ rely on some autogenerated files
`.dart_tool/package_json.config`, which is generated by `flutter pub get`, and
`.idea/libraries/Dart_SDK.xml`, which is generated by IntelliJ.

Therefore, to let those IDEs reload the SDKs after switching Flutter version,
you need to run `flutter pub get` in your workspace root to regenerate the
`.dart_tool/package_json.config` file.

```shell
$ cd workspace
$ fenv version
3.3.0 (set by `...`)
$ fenv local 3.10.0
$ fenv version
3.10.0 (set by `$FENV_ROOT/version`)

# If a Flutter version is switched, please do one of the followings:
$ flutter pub get  # in most cases, this might be fine.
$ melos bs         # if your project are managed by "melos".
$ fenv workspace . # if you are an IntelliJ IDEA's user. "." means the current directory
```

Unlike Visual Studio Code, IntelliJ's `Dart` plugin requires a little more step
to find the correct Dart SDK path. If you are an IntelliJ IDEA' user or an
Android Studio's user, you may need to run `fenv workspace .` to regenerate
`.idea/libraries/Dart_SDK.xml` file as well as `.dart_tool/package_json.config`
file.

If you were opening an IDE like Visual Studio Code and IntelliJ IDEA (including
Android Studio), close and re-open it to let the IDE reload the Flutter SDK and
the Dart SDK path.

### If Dart-based CLI tools (such as `"melos"`) do not work well after switching Flutter SDK

Dart-based CLI tools, for example [`melos`][melos] and
[`vector_graphics_compiler`][vector_graphics_compiler], may show a warning like
_`"Can't load Kernel binary: Invalid kernel binary format version."`_ after
switching Flutter SDK.

This is not the problem of `fenv` but a intrinsic problem of Dart-based CLI
tools. You can do activate the CLI once more to suppress these warnings
messages.

For example, you can re-activate `melos` like:

```shell
$ flutter pub global activate melos
```

[FVM]: https://fvm.app/
[melos]: https://melos.invertase.dev/~melos-latest/
[pyenv]: https://github.com/pyenv/pyenv
[vector_graphics_compiler]: https://pub.dev/packages/vector_graphics_compiler
