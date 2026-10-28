# aoshell

> **A**ozora **O**rchestration **S**hell
> *A high-speed, extensionless CLI dispatcher and script manager.*

`aoshell` is a zero-overhead command router and script manager built for the Aozora Linux ecosystem. It acts as a smart traffic cop for your custom automation scripts, providing a clean, centralized namespace while leaving the heavy lifting of script execution entirely to the Linux kernel and your preferred scripting engines.

The design philosophy of `aoshell` is radical simplicity: **extensionless scripts driven by native shebangs.** No custom virtual machines, no runtime parsing—just drop an executable file into your scripts directory, and `aoshell` hands it straight to the OS.

---

## Key Features

* **Extensionless Script Discovery:** Manage scripts cleanly without clunky file extensions like `.sh`, `.py`, or `.js`. The kernel reads the shebang line automatically.
* **Polyglot Execution:** Run Bash, Python, Perl, Lua, or Rust scripts interchangeably under a single consistent interface.
* **Transparent Argument Forwarding:** Pass flags and parameters straight through to your scripts (e.g., forwarding `--freeze` to a custom screenshot utility).
* **Intelligent Editor Fallback:** The built-in `edit` command automatically checks `$VISUAL` and `$EDITOR` environment variables, falling back smoothly to `nano`.
* **Zero-Overhead Rust Core:** Compiled with aggressive release optimizations (`opt-level = "z"`, LTO, and binary stripping) for instant execution.
* **Seamless Shell Integration:** Automatically handles shell alias setup (`cs="aoshell"`) for Bash, Zsh, and Fish during initialization.

---

## How It Works & Script Routing

When you invoke `aoshell` (or your shorthand alias `cs`), it parses your input, checks the configured scripts directory (`~/.config/aoshell/scripts/`), verifies execution permissions (`+x`), and delegates execution via standard process spawning.

```
                                  ┌─ init ──────> Create directory & shell alias
                                  ├─ list ──────> Enumerate available scripts
                                  ├─ new ───────> Scaffold extensionless script template
User Input (aoshell / cs) ───────>├─ edit ──────> Open script in $VISUAL / $EDITOR / nano
                                  ├─ remove ────> Delete script file
                                  └─ <script> ──> Forward execution + args to kernel shebang

```

Instead of acting as a heavy scripting engine, `aoshell` functions as a high-speed lobby desk that keeps your window manager keybinds clean and your dotfiles organized.

---

## Installation

### Prerequisites

Ensure you have Rust and Cargo installed:

```bash
sudo pacman -S --needed rust

```

### Installation via `puyo` (Recommended)

Since `aoshell` includes a `.puyo` manifest, you can install and track it natively on your Arch system using `puyo`:

```bash
puyo https://github.com/CatCodeLabs/aoshell
# or from a local clone directory
puyo .

```

### Manual Installation

Clone the repository and build via Cargo:

```bash
git clone https://github.com/CatCodeLabs/aoshell
cd aoshell
cargo build --release
sudo cp target/release/aoshell /usr/bin/aoshell

```

---

## Usage

`aoshell` provides a clean subcommand interface alongside a shorthand execution pattern.

### 1. Initializing Environment & Shell Alias

Run the initialization routine to set up your config directory and automatically inject the `cs` alias into your active shell (`.bashrc`, `.zshrc`, or `config.fish`):

```bash
aoshell init

```

### 2. Creating New Scripts

Scaffold a new extensionless script template with automatic executable permissions applied:

```bash
aoshell new screenshot

```

This generates `~/.config/aoshell/scripts/screenshot` pre-configured with a clean Bash shebang and comment headers.

### 3. Executing Scripts (with Shorthand)

Run any custom script by name, appending any arguments you need:

```bash
cs screenshot area
# or explicitly:
aoshell screenshot --freeze output

```

### 4. Listing, Editing, and Removing Scripts

List all available automation scripts currently installed:

```bash
aoshell list

```

Open an existing script for editing using your system editor (`$VISUAL`, `$EDITOR`, or falling back to `nano`):

```bash
aoshell edit screenshot
```

Safely delete a script when it's no longer needed:

```bash
aoshell remove old-script
# or
aoshell rm old-script
```

---

## License

`aoshell` is part of [CatCodeLabs](https://www.google.com/search?q=https://github.com/CatCodeLabs), and is licensed under the GNU General Public License v3.0. See `LICENSE` for more information.