# Pswitcher

**[Español](README.md)**

Pswitcher is a Linux desktop app for installing and managing PHP, Node.js, Composer, Go, and Flutter versions in a directory you choose. Select the active versions in the app; optional Bash integration also makes the selected commands available in new terminal sessions.

## Download

Download the x86_64 AppImage from [Releases](https://github.com/gsanmartindougan/pswitcher/releases), then run:

```sh
chmod +x Pswitcher-*-x86_64.AppImage
./Pswitcher-*-x86_64.AppImage
```

## Usage

- Choose the install directory in **Settings**. The default is `~/Tools/pswitcher`.
- Install and select multiple versions of each tool from **Versions**. Catalogs load when you open an install screen; Node, Go, and Flutter support searching by version number.
- Choose PHP extensions during installation or reinstallation. Pswitcher builds PHP from its official source and verifies the requested extensions.
- Enable Bash integration in **Settings** to let Pswitcher manage its marked block in `~/.bashrc`. Open a new terminal after changing the selection.
- Without Bash integration, the selection applies inside Pswitcher and does not change the `PATH` in terminal sessions.
- **Settings** can check system build tools. It shows dependency guidance for `apt` on Debian/Ubuntu derivatives and `dnf` on Fedora/RHEL derivatives.

Versions are kept separately by tool and version under the chosen directory. App settings are stored in the user's XDG configuration directory.

## Requirements and scope

- Linux on x86_64 or aarch64. The AppImage in this release is x86_64.
- Building PHP requires a C compiler, GNU Make, `tar`, `xz`, and development libraries required by the selected extensions. Pswitcher does not install system packages.
- PHP extension availability depends on the development libraries installed on your distribution.
- Node.js, Composer, Go, and Flutter are downloaded from their official catalogs and sources; Pswitcher validates downloads when a checksum is provided by the catalog.
- This version manages global selections from the app; it does not apply per-project versions.

## Build from source

You need stable Rust and Iced's Linux development dependencies (X11/Wayland). Run the development build with:

```sh
cargo run
```

Build the x86_64 AppImage with the packaging container:

```sh
./packaging/build-appimage.sh
```

The result is written to the project root as `Pswitcher-<version>-x86_64.AppImage`.
