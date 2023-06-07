use std::path::PathBuf;

pub struct PackageConfigReader {
    package_path: PathBuf,
}

impl PackageConfigReader {
    #[must_use]
    #[inline]
    pub fn new(package_path: PathBuf) -> Self {
        Self { package_path }
    }

    #[inline]
    pub fn package_path(&self) -> &PathBuf {
        &self.package_path
    }
}
