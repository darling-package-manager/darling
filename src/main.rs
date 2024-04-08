use clap::Parser as _;
use colored::Colorize as _;
use darling_api as darling;

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
    MODULES.get_or_init(|| include!("./modules.rs"))
}

#[derive(clap::Subcommand)]
enum SubCommand {
    Install { package_name: String },
    Remove { package_name: String },
    Rebuild,
    Reload,
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
        std::fs::write(format!("{home}/.config/darling/darling.toml", home = std::env::var("HOME")?), "[module]")
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
fn run(distro: &dyn darling::PackageManager, command: SubCommand) -> anyhow::Result<()> {
    let context = darling::Context {
        config: darling::DarlingConfig::default(),
    };
    let mut config = read_config()?;

    // Run the subcommand
    match command {
        SubCommand::Install { package_name } => {
            let mut package = darling::InstallationEntry {
                name: package_name,
                properties: std::collections::HashMap::new(),
            };

            // Print an installation message
            println!("{}", format!("Installing package \"{}\"...", &package.name).cyan().bold());

            // Install the package in the system
            let version = distro.install(&context, &package)?;
            if let Some(mut version_string) = version {
                version_string.replace_range(0..1, "=");
                package.properties.insert("version".to_owned(), version_string);
            }

            // If no version is specified, set it to "latest"
            if package.properties.get("version").is_none() {
                package.properties.insert("version".to_owned(), "latest".to_owned());
            }

            // Serialize the package data into TOML
            let mut properties_table: toml_edit::InlineTable = toml_edit::InlineTable::new();
            for (key, value) in package.properties {
                properties_table.insert(&key, toml_edit::Value::String(toml_edit::Formatted::new(value)));
            }
            properties_table.set_dotted(false);

            // Get the packages table from the config file
            let mut blank_table = toml_edit::Item::Table(toml_edit::Table::new());
            let packages_item = config.get_mut(&distro.name()).unwrap_or(&mut blank_table).to_owned();
            let toml_edit::Item::Table(mut packages) = packages_item else {
                anyhow::bail!("Corrupted config file: \"{}\" is not a table", distro.name())
            };

            packages[&package.name] = toml_edit::Item::Value(toml_edit::Value::InlineTable(properties_table));
            config.insert(&distro.name(), toml_edit::Item::Table(packages));

            // Write the config file
            std::fs::write(format!("{}/.config/darling/darling.toml", std::env::var("HOME")?), config.to_string())?;

            // Print a success message
            println!("{}", format!("Package \"{}\" installed successfully!", &package.name).green().bold());
        }

        SubCommand::Remove { package_name } => {
            let package_entry = darling::InstallationEntry {
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

        SubCommand::Rebuild => {
            let items = config.as_table();
            for (name, package_item) in items {
                println!("{} packages for module {}...", "Installing".green().bold(), name.cyan().bold());
                let module = *modules()
                    .iter()
                    .find(|module| module.name() == name)
                    .ok_or_else(|| anyhow::anyhow!("Corrupted config file: Module \"{}\" not found", name))?;
                let toml_edit::Item::Table(packages) = package_item else {
                    anyhow::bail!("Corrupted config file: Module \"{}\" is found, but isn't a table.", name)
                };

                for (package_name, _package_data) in packages {
                    println!("\t{} package {}", "Installing".green().bold(), package_name.cyan().bold());
                    module.install(
                        &context,
                        &darling::InstallationEntry {
                            name: package_name.to_owned(),
                            properties: std::collections::HashMap::new(),
                        },
                    )?;
                }
            }
        }

        _ => unimplemented!(),
    };

    Ok(())
}
