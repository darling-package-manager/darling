# darling

**D**eclarative **A**nd **R**eproducable **LIN**ux **G**eneralized

An extendable and declarative package management system for Linux.

Darling allows existing package managers of almost any form to be managed declaratively, such as
- OS Package managers like `pacman`, `apt`, `dnf`, etc.
- NodeJS global packages
- Visual Studio Code extensions
- Global cargo packages

... and more.

## Installation

### With `cargo` and [darling-installer](https://github.com/darling-package-manager/darling-installer) (Recommended)

```bash
cargo install darling-installer
install-darling
```

### Manual

Each time you add a new module to `darling`, The code must be rebuilt from source to include the new Rust library. Thus, the `darling` source code must always live on your machine. The default location that does not require configuration changes is to place the source at `~/.local/share/daring/soure`. You can locate it there as such:

```bash
git clone https://github.com/darling-package-manager/darling.git
mkdir -p ~/.local/share/darling
mv darling ~/.local/share/darling/source
```

In your `.bashrc` (or somewhere that you edit your `$PATH`) add:

```bash
export PATH="$PATH:~/.local/share/darling/source/target/release
```

Also, ensure you build the project at least once, by `cd`ing into the `source` directory and running `cargo build --release`. After that, `darling` will rebuild itself when new modules are added.

## Usage

First, install your module of choice. The list of (known) available modules is covered below. We'll use Arch Linux's `pacman` as an example. 
- First, run `darling module install arch` to install the Arch module. 
- Optionally, you can run `darling arch load-installed` to load all existing *explicitly* (not dependencies) installed `pacman` packages on your system. Note that this often adds hardware-specific packages that shouldn't be tracked by `darling` and generally should go through manual review.
- When you want to install a package, instead of running something like `pacman -S ripgrep`, you can run `darling arch install ripgrep`, which will install the package through your system through `pacman`, but also add `ripgrep` to your declarative configuration file (`~/.config/darling/darling.toml`). Alternatively, you can employ a NixOS-style workflow, and manually insert your package into your configuration file, and then run `darling arch reload` to ensure all packages listed in the declarative configuration are installed.

### Configuration Schema

The `darling` configuration file (`~/.config/darling/darling.toml`) has the following schema:

```toml
[module-name-1]
package-1 = { ...package_data-1 }
package-2 = { ...package_data-2 }

[module-name-2]
package-3 = { ...package_data-3 }
package-4 = { ...package_data-4 }
```

Where `module-name-1` and `module-name-2` are module names such as `arch` or `vscode`, `package-1` through `package-4` are package names like `ripgrep` or `"svelte.svelte-vscode"`, and `package-data-1` through `package-data-4` are tables such as `{ version = "1.0.0" }` that provide metadata about the package. Below is an example configuration that I've personally used:

```toml
[module]
arch = { version = "=1.0.81" }
vscode = { version = "=1.0.81" }

[arch]
base-devel = { version = "1-1" }
cargo-wizard = { version = "0.2.2-1" }
discord = { version = "0.0.48-1" }
firefox = { version = "124.0.2-1" }
go = { version = "2:1.22.2-1" }
neovim = { version = "0.9.5-5" }
nodejs = { version = "21.7.2-1" }
npm = { version = "10.5.1-1" }
ripgrep = { version = "14.1.0-1" }
unzip = { version = "6.0-20" }
vscodium = { version = "1.88.0.24096-1" }
wezterm = { version = "20240203.110809.5046fc22-1" }
wget = { version = "1.24.5-1" }

[vscode]
"svelte.svelte-vscode" = { version = "108.3.3" }
"violetiapalucci.one-midnight" = { version = "1.0.0" }
"usernamehw.errorlens" = { version = "3.16.0" }
"tamasfe.even-better-toml" = { version = "0.19.1" }
"redhat.java" = { version = "1.29.0" }
"pkief.material-icon-theme" = { version = "4.34.0" }
"rust-lang.rust-analyzer" = { version = "0.3.1916" }
"vscodevim.vim" = { version = "1.27.2" }
```

## Implementing Darling

### Implementing From Template

[darling-template](https://github.com/darling-package-manager/darling-template) provides a starting point for implementing darling. To use it, first set the template up onto your local machine:

#### With version contol

Open the repository and click "Use this template > Create a new repository". This will set up a git repo that you can clone onto your local machine and begin development with git immediately.

#### Without version control

If you don't want to make a git repository for your project right now, you can just clone the template to get it onto your local machine.

#### Next steps

Next, rename the struct and implement the missing methods. Read the documentation for each method carefully to understand what it must do and not do. **Do not rename the global `PACKAGE_MANAGER` variable**.

Edit your crate name in `Cargo.toml`, and ensure it starts with `darling-`, such as `darling-npm` or `darling-vscode`. Publish your crate when it's done!

That's it! once your crate is published, it can be used by anyone with darling, no updates required.

### Manual Implementation

Darling is designed specifically to be extendible *without changing darling itself*. This means that new package-related tools can add their own support for darling. `darling` uses a very specific protocol for creating modules. The process is as follows:

- Create a rust (library) project. **It must start with `darling-`**. For example, `cargo new darling-example --lib`. Ensure that your name isn't taken on `crates.io`.
- Add `darling-api` to your dependencies with `cargo add darling-api`.
- Create an empty struct that implements `darling::PackageManager`.
	- Ensure that the `get_name()` function returns a consistent value on all calls, and that **it does not return "module"**. `module` is a built-in reserved name used by darling to manage itself. Also, it should be unique to other darling modules, or else they will be incompatible. It is convention to make it the name of your crate, without the `darling-` prefix. For example, the `darling
- **Declare a `pub static` variable of your struct with the name `PACKAGE_MANAGER` that is accessible from your crate root.**
- Publish your crate on `crates.io` with `cargo publish`

## Contributor notes

For developers or contributors to the project, the following points should be noted:

- use `git update-index --skip-worktree <FILENAME>` on `Cargo.toml`, `Cargo.lock`, and `src/modules.rs`. This allows you to use `darling` and install modules onto your machine without pushing those user-specific installations through git. If you need to add a new dependency, temporarily use `git update-index --no-skip-worktree` on `Cargo.toml` and `Cargo.lock` to add the dependency, and then skip them once more to prevent other changes from being pushed. If you do not plan on using/testing darling on your system and installing modules with it, then this point isn't necessary. Also, if you want to *use* darling but not necessarily test your development version, you could always develop in a location other than `~/.local/share/darling/source` and leave your usage version in that location.
- **`darling` is in an Alpha phase, and there will be breaking changes. If you do not want to deal with updating an implementation of darling for a specific package manager, please wait until 1.0**.