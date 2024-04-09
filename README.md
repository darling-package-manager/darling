# darling

**D**eclarative **A**nd **R**eproducable **LIN**ux **G**eneralized

An extendable and declarative package management system for Linux.

Darling allows existing package managers of almost any form to be managed declaratively, such as
- OS Package managers like `pacman`, `apt`, `dnf`, etc.
- NodeJS global packages
- Visual Studio Code extensions

... and more.

## Installation

### Manual

Each time you add a new module to `darling`, The code must be rebuilt from source to include the new Rust library. Thus, the `darling` source code must always live on your machine. The default location that does not require configuration changes is to place the source at `~/.local/share/daring/soure`. You can locate it there as such:

```bash
git clone https://github.com/darling-package-manager/darling.git
mkdir -p ~/.local/share/darling
mv darling ~/.local/share/darling/source
```

In your `.bashrc` (or somewhere that you edit your `$PATH`) add:

```bash
export $PATH="$PATH:~/.local/share/darling/source/target/release
```

Also, ensure you build the project at least once, by `cd`ing into the `source` directory and running `cargo build --release`. After that, `darling` will rebuild itself when new modules are added.

## Implementing Darling
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