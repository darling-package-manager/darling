use darling_api as darling;

pub static PACKAGE_MANAGER: Darling = Darling;

pub struct Darling;

impl darling::PackageManager for Darling {
    fn name(&self) -> String {
        "module".to_owned()
    }

    fn install(&self, context: &darling::Context, package: &darling::InstallationEntry) -> anyhow::Result<Option<String>> {
        // Add the crate dependency
        std::process::Command::new("cargo")
            .arg("add")
            .arg(format!("darling_{}", &package.name))
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

        // Rebuild from source
        std::process::Command::new("cargo")
            .arg("build")
            .arg("--release")
            .current_dir(&context.config.source_location)
            .spawn()?
            .wait()?;

        // Get the version info
        let tree_info = String::from_utf8(
            std::process::Command::new("cargo")
                .arg("tree")
                .current_dir(&context.config.source_location)
                .output()?
                .stdout,
        )?;
        let version_pattern = regex_macro::regex!(r"(?ms).{3}\s(\w+)\s(\S+)");
        let mut version = version_pattern
            .captures(&tree_info)
            .ok_or_else(|| anyhow::anyhow!("Error parsing version info for crate"))?[2]
            .to_owned();
        version.replace_range(0..1, "=");
        Ok(Some(version))
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
            .filter(|line| line != &format!("darling_{}::PACKAGE_MANAGER,", &package.name))
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
