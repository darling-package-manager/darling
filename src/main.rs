use clap::Parser as _;
use colored::Colorize as _;
use darling::InstallationEntry;
use darling_api as darling;
use std::io::Write as _;

/// The bootstrap module. This can essentially be thought of as `darling-darling`. This module uses `darling` to
/// manage its own modules.
mod bootstrap;

/// The main function of the binary executable. This runs the `darling` command with the arguments
/// passed on the command line, and returns an error if anything goes wrong during execution (malformatted input,
/// network error installing a package, package doesn't exist, etc.)
fn main() -> anyhow::Result<()> {
    let args = Args::try_parse()?;
    let module = *modules()
        .iter()
        .find(|module| module.name() == args.module)
        .ok_or_else(|| anyhow::anyhow!("Module \"{}\" not found", args.module))?;
    println!();
    run(module, args.command)?;
    Ok(())
}

/// The command-line arguments passed to the `darling` executable. This is automatially parsed by `clap`.
/// using `Args::parse()` or `Args::try_parse()`
#[derive(clap::Parser)]
struct Args {
    /// The module to run the subcommand on, i.e. `arch` or `npm`.
    module: String,

    /// The command to run, such as `install` or `uninstall`.
    #[command(subcommand)]
    command: SubCommand,
}

/// The modules that are currently present. These act as "plugins" to `darling`, adding new package management
/// systems to the program. This is a lazily-evaluated static (hence the `OnceLock`) via the [modules] function.
static MODULES: std::sync::OnceLock<Vec<&'static dyn darling::PackageManager>> = std::sync::OnceLock::new();

/// Fetches, or initializes (if it is not yet initialized) the [MODULES] for this build. This returns the data
/// as a slice to the underlying `Vec` stored in `MODULES`. This is loaded by reading the file at `./modules.rs`,
/// which is edited manually by the `darling` application itself.
#[allow(incomplete_include)]
fn modules() -> &'static [&'static dyn darling::PackageManager] {
    MODULES.get_or_init(|| include!("./modules.rs"))
}

/// A subcommand for a specific module in the program. For example, in a call to `darling arch install ripgrep` or
/// `darling module install npm`, this represents the `install` portion of the command. This is automatically parsed
/// by `clap` when using the [Args] struct.
#[derive(clap::Subcommand)]
enum SubCommand {
    Install { package_name: String },
    Remove { package_name: String },
    Rebuild,
    LoadInstalled,
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
            install(
                distro,
                &context,
                &mut config,
                InstallationEntry {
                    name: package_name,
                    properties: std::collections::HashMap::new(),
                },
                true,
            )?;
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

                let all = module.get_all_explicit(&context)?.into_iter().map(|tuple| tuple.0).collect::<Vec<_>>();

                for (package_name, _package_data) in packages {
                    print!("\t{} package {}... ", "Installing".green().bold(), package_name.cyan().bold());
                    std::io::stdout().flush()?;
                    if all.contains(&package_name.to_owned()) {
                        println!("Already installed and up to date. {}", "Skipping.".yellow().bold());
                    } else {
                        module.install(
                            &context,
                            &darling::InstallationEntry {
                                name: package_name.to_owned(),
                                properties: std::collections::HashMap::new(),
                            },
                        )?;
                        println!("{}", "Done!".green().bold());
                    }
                }
                println!("{} installing packages for module {}!", "Finished".green().bold(), name.cyan().bold());
                println!();
            }
        }

        SubCommand::LoadInstalled => {
            let installed = distro.get_all_explicit(&context)?;
            for (package, version) in installed {
                install(
                    distro,
                    &context,
                    &mut config,
                    InstallationEntry {
                        name: package,
                        properties: std::collections::HashMap::from([("version".to_owned(), version)]),
                    },
                    false,
                )?;
            }
        }
    };

    Ok(())
}

fn install(
    distro: &dyn darling::PackageManager,
    context: &darling::Context,
    config: &mut toml_edit::DocumentMut,
    mut package: darling::InstallationEntry,
    with_system: bool,
) -> anyhow::Result<()> {
    // Print an installation message
    println!("{}", format!("Installing package \"{}\"...", &package.name).cyan().bold());

    // Install the package in the system
    if with_system {
        distro.install(context, &package)?;
        distro.post_install(context)?;
        let version = distro
            .get_all_explicit(context)?
            .iter()
            .find(|(pack, _ver)| pack == &package.name)
            .ok_or_else(|| anyhow::anyhow!("Package {} was just installed explicitly, yet no version information could be found for it!", &package.name))?
            .1
            .to_owned();
        package.properties.insert("version".to_owned(), version);
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

    Ok(())
}
