#[derive(Clone)]
pub struct InstallationEntry {
    pub name: String,

    /// Additional properties specified on the command line. These are arbitrary String-String mappings passed as long arguments
    /// by the user, and are used for distro-specific or package-manager-specific operations. For example, on Arch linux, a user
    /// may run `darling install joshuto --source=aur` to install a package such as joshuto from the AUR.
    pub properties: std::collections::HashMap<String, String>,
}

pub struct Context {
    pub config: DarlingConfig,
}

pub struct DarlingConfig {
    pub source_location: String,
}

impl std::default::Default for DarlingConfig {
    fn default() -> Self {
        Self {
            source_location: std::env::var("HOME").unwrap() + "/.local/share/darling/source",
        }
    }
}

pub trait PackageManager: Send + Sync {
    fn name(&self) -> String;

    /// Installs a package with the given version. If no version is supplied, this should install the latest version.
    /// Note that this ***does not*** affect the cache file. This simply supplies the system package install command.
    ///
    /// # Parameters
    /// - `package` - The name of the package to install.
    /// - `version` - The version of the package to install. If `None`, the latest version should be installed.
    ///
    /// # Returns
    /// The version of the package installed, if none was specified in the input. Otherwise, `None`, or an
    /// error if the package could not be installed.
    fn install(&self, context: &Context, package: &InstallationEntry) -> anyhow::Result<Option<String>>;

    /// Uninstalls a package from the system. This does ***not*** affect the cache file, it simply removes the package
    /// from the system itself, and `darling-core` will handle removing the package from the cache file.
    ///
    /// # Parameters
    /// - `package` - The name of the package to remove.
    ///
    /// # Returns
    /// An error if the package could not be removed.
    fn uninstall(&self, context: &Context, package: &InstallationEntry) -> anyhow::Result<()>;

    /// Returns all *explicitly* installed packages on the system; That is, packages which are not dependencies of
    /// other packages. This **should not** read from a darling file; Instead, darling uses this method to update
    /// the file when running `darling require-all`
    ///
    /// # Returns
    /// The names of all explicitly installed packages as a `Vec<String>`, or an error if there was a system error
    /// retrieving them.
    fn get_all_explicit(&self, context: &Context) -> anyhow::Result<Vec<String>>;
}
