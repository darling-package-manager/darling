use clap::Parser as _;
use colored::Colorize as _;
use darling::{Context, DarlingConfig, InstallationEntry, PackageManager};
use darling_packages as darling;
use std::io::Write as _;

mod bootstrap;

fn main() -> anyhow::Result<()> {
    let args = Args::try_parse()?;
    let module = *modules()
        .iter()
        .find(|module| module.name() == args.module)
        .ok_or_else(|| anyhow::anyhow!("Module \"{}\" not found", args.module))?;
    run(module, args.command)?;
    Ok(())
}

#[derive(clap::Parser)]
struct Args {
    module: String,

    #[command(subcommand)]
    command: SubCommand,
}

static MODULES: std::sync::OnceLock<Vec<&'static dyn darling::PackageManager>> = std::sync::OnceLock::new();

#[allow(incomplete_include)]
fn modules() -> &'static [&'static dyn darling::PackageManager] {
    MODULES.get_or_init(|| vec![include!("./modules.rs")])
}

#[derive(clap::Subcommand)]
enum SubCommand {
    Install { package_name: String },
    InstallModule { package_name: String },
    Remove { package_name: String },
    Reload,
}

pub fn install_module_without_cache_update(context: &darling::Context, module_name: &str) -> anyhow::Result<()> {
    std::process::Command::new("cargo")
        .arg("add")
        .arg(format!("darling-{}", module_name))
        .current_dir(&context.config.source_location)
        .spawn()?;
    let mut module_file = std::fs::OpenOptions::new().append(true).open(context.config.source_location.clone() + "/src/modules.rs")?;
    writeln!(module_file, "{},", module_name)?;
    Ok(())
}

pub fn uninstall_module_without_cache_update(context: &darling::Context, module_name: &str) -> anyhow::Result<()> {
    std::process::Command::new("cargo")
        .arg("remove")
        .arg(format!("darling-{}", module_name))
        .current_dir(&context.config.source_location)
        .spawn()?;
    let module_file_lines = std::fs::read_to_string(context.config.source_location.clone() + "/src/modules.rs")?
        .lines()
        .filter(|line| line != &format!("darling_{module_name}::PACKAGE_MANAGER,"))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(context.config.source_location.clone() + "/src/modules.rs", module_file_lines)?;
    Ok(())
}

/// Reads the config file, creating it if it doesn't exist.
///
/// # Returns
/// The config file as a TOML document.
fn read_config() -> anyhow::Result<toml_edit::DocumentMut> {
    // Create the config file if it doesn't exist
    if !std::path::Path::new(&format!("{home}/.config/darling/darling.toml", home = std::env::var("HOME")?)).exists() {
        std::fs::create_dir_all(format!("{home}/.config/darling/", home = std::env::var("HOME")?))
            .map_err(|err| anyhow::anyhow!(format!("Error creating config directory: {err:?}").red().bold()))?;
        std::fs::write(format!("{home}/.config/darling/darling.toml", home = std::env::var("HOME")?), "[packages]")
            .map_err(|err| anyhow::anyhow!("Error writing initial config file: {err:?}"))?;
    }

    // Read the config file
    let config = std::fs::read_to_string(format!("{home}/.config/darling/darling.toml", home = std::env::var("HOME")?))
        .map_err(|err| anyhow::anyhow!(format!("Error reading config file: {err:?}").red().bold()))?;

    // Parse the config file
    let config_toml: toml_edit::DocumentMut = config.parse()?;
    Ok(config_toml)
}

/// Runs the main program.
///
/// # Parameters
/// - `distro` - The distro to use for installing packages.
///
/// # Returns
/// An error if the program could not be run.
fn run(distro: &dyn PackageManager, command: SubCommand) -> anyhow::Result<()> {
    let context = Context { config: DarlingConfig::default() };
    let mut config = read_config()?;

    // Run the subcommand
    match command {
        SubCommand::Install { package_name } => {
            let mut package = InstallationEntry {
                name: package_name,
                properties: std::collections::HashMap::new(),
            };

            // Print an installation message
            println!("{}", format!("Installing package \"{}\"...", &package.name).cyan().bold());

            // Install the package in the system
            distro.install(&context, &package)?;

            // Get the packages table from the config file
            let toml_edit::Item::Table(packages) = config
                .get_mut("packages")
                .ok_or_else(|| anyhow::anyhow!("Corrupted config file: No package field found".red().bold()))?
            else {
                anyhow::bail!("Corrupted config file: \"packages\" field exists in config, but is not a table".red().bold());
            };

            // If no version is specified, set it to "latest"
            if package.properties.get("version").is_none() {
                package.properties.insert("version".to_owned(), "latest".to_owned());
            }

            // Serialize the package data into TOML
            let mut properties_table: toml_edit::InlineTable = toml_edit::InlineTable::new();
            for (key, value) in package.properties {
                properties_table[&key] = toml_edit::Value::String(toml_edit::Formatted::new(value));
            }
            properties_table.set_dotted(false);
            packages[&package.name] = toml_edit::Item::Value(toml_edit::Value::InlineTable(properties_table));

            // Write the config file
            std::fs::write(format!("{}/.config/darling/darling.toml", std::env::var("HOME")?), config.to_string())?;

            // Print a success message
            println!("{}", format!("Package \"{}\" installed successfully!", &package.name).green().bold());
        }

        SubCommand::Remove { package_name } => {
            let package_entry = InstallationEntry {
                name: package_name,
                properties: std::collections::HashMap::new(),
            };
            println!("{}", format!("Removing package \"{}\"...", &package_entry.name).red().bold());
            distro.uninstall(&context, &package_entry)?;
            let toml_edit::Item::Table(packages) = config.get_mut("packages").ok_or_else(|| anyhow::anyhow!("No packages in config"))? else {
                anyhow::bail!("Packages in config is not a table");
            };
            packages.remove(&package_entry.name);
            std::fs::write(format!("{}/.config/darling/darling.toml", std::env::var("HOME")?), config.to_string())?;
            println!("{}", format!("Package \"{}\" installed successfully!", &package_entry.name).green().bold());
        }
        _ => unimplemented!(),
    };

    Ok(())
}
