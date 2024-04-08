use darling_packages as darling;

pub static PACKAGE_MANAGER: Darling = Darling;

pub struct Darling;

impl darling::PackageManager for Darling {
    fn name(&self) -> String {
        "darling".to_owned()
    }

    fn install(&self, context: &darling::Context, package: &darling::InstallationEntry) -> anyhow::Result<Option<String>> {
        crate::install_module_without_cache_update(context, &package.name)?;
        Ok(None)
    }

    fn uninstall(&self, context: &darling::Context, package: &darling::InstallationEntry) -> anyhow::Result<()> {
        crate::uninstall_module_without_cache_update(context, &package.name)
    }

    fn get_all_explicit(&self, context: &darling::Context) -> anyhow::Result<Vec<String>> {
        let module_pattern = regex_macro::regex!("^(.+)::PACKAGE_MANAGER,$");
        std::fs::read_to_string(context.config.source_location.clone() + "/src/modules.rs")?
            .lines()
            .map(|line| Ok(module_pattern.captures(line).ok_or_else(|| anyhow::anyhow!("Malformatted cache file"))?[1].to_owned()))
            .collect::<Result<Vec<_>, _>>()
    }
}
