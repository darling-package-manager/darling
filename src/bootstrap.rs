use std::io::Write as _;

use colored::Colorize as _;
use darling_api as darling;

pub static PACKAGE_MANAGER: Darling = Darling;

pub struct Darling;

const OFFICIAL_MODULES: &[&str] = &["arch", "cargo", "module", "npm", "vscode"];

impl darling::PackageManager for Darling {
    fn name(&self) -> String {
        "module".to_owned()
    }

    fn install(&self, context: &darling::Context, package: &darling::InstallationEntry) -> anyhow::Result<()> {
        // Warn that it's community made
        if !OFFICIAL_MODULES.contains(&package.name.as_str()) {
            println!(
                "{} module {} is an unofficial community-made module which may contain malicious or unsafe code.",
                "Warning:".yellow().bold(),
                package.name.cyan().bold()
            );
            print!("Would you still like to install it? (Y/n): ");
            std::io::stdout().flush()?;
            let mut buffer = String::new();
            std::io::stdin().read_line(&mut buffer)?;
            if buffer.trim().to_lowercase() != "y" {
                anyhow::bail!("Cancelled installation due to unofficial package.");
            }
        }
        // Add the crate dependency
        std::process::Command::new("cargo")
            .arg("add")
            .arg(format!("darling-{}", &package.name))
            .current_dir(&context.config.source_location)
            .spawn()?
            .wait()?;

        // Write to the include! file
        let module_file = std::fs::read_to_string(context.config.source_location.clone() + "/src/modules.rs")?;
        let mut lines = module_file.lines().collect::<Vec<_>>();
        lines.remove(lines.len() - 1);
        let entry = format!("\t&darling_{}::PACKAGE_MANAGER,", &package.name);
        lines.push(&entry);
        lines.push("]");
        std::fs::write(context.config.source_location.clone() + "/src/modules.rs", lines.join("\n"))?;

        Ok(())
    }

    fn post_install(&self, context: &darling::Context) -> anyhow::Result<()> {
        // Rebuild from source
        std::process::Command::new("cargo")
            .arg("build")
            .arg("--release")
            .current_dir(&context.config.source_location)
            .spawn()?
            .wait()?;

        Ok(())
    }

    fn uninstall(&self, context: &darling::Context, package: &darling::InstallationEntry) -> anyhow::Result<()> {
        // Add the crate dependency
        std::process::Command::new("cargo")
            .arg("remove")
            .arg(format!("darling-{}", &package.name))
            .current_dir(&context.config.source_location)
            .spawn()?
            .wait()?;

        // Write to the module include! file
        let module_file_lines = std::fs::read_to_string(context.config.source_location.clone() + "/src/modules.rs")?
            .lines()
            .filter(|line| line.trim() != format!("&darling_{}::PACKAGE_MANAGER,", &package.name))
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(context.config.source_location.clone() + "/src/modules.rs", module_file_lines)?;
        Ok(())
    }

    fn get_all_explicit(&self, context: &darling::Context) -> anyhow::Result<Vec<(String, String)>> {
        let tree_info = String::from_utf8(
            std::process::Command::new("cargo")
                .arg("tree")
                .current_dir(&context.config.source_location)
                .output()?
                .stdout,
        )?;
        let version_pattern = regex_macro::regex!(r"(?ms).{3}\sdarling-(\w+)\s(\S+)");

        let mut packages = Vec::new();
        for dependency in version_pattern.captures_iter(&tree_info) {
            if &dependency[1] == "api" {
                continue;
            }
            packages.push((dependency[1].to_owned(), dependency[2].to_owned()));
        }

        Ok(packages)
    }
}
