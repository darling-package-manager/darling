## Config Schema
The config file has the following fields:
- `[packages]: { [string]: { version: string }}` - List of base 

## Example Configurations
- Arch:
```toml
[packages]
firefox = { version = "124.0.2-1" }
vim = { version = "9.1.0252-1" }
neovim = { version = "0.9.5-5" }
libx11 = { version = "1.8.8-3", window_system = "X11" }
xorg-xwayland = { version = "23.2.5-1", window_system = "Wayland" }
joshuto = { version = "0.9.8-1", source = "AUR" }
```

## Implementing Darling
- Add `darling-manager` to your dependencies
- Create an empty struct that implements `darling::Declarative`
- **In your main file, declare a `pub static` variable of your struct with the name `package`.**