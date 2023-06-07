use std::path::PathBuf;

pub struct ProjectPathResolver {
    root: PathBuf,
}

impl ProjectPathResolver {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn root_config(&self) -> PathBuf {
        self.root.join("config.xml")
    }

    pub fn readme(&self) -> PathBuf {
        self.root.join("README.md")
    }

    pub fn src_directory(&self) -> PathBuf {
        self.root.join("src")
    }

    pub fn build_directory(&self) -> PathBuf {
        self.root.join("build")
    }

    pub fn build_package_directory(&self, package_name: &str) -> PathBuf {
        self.build_directory().join(package_name)
    }

    pub fn build_package_config(&self, package_name: &str) -> PathBuf {
        self.build_package_directory(package_name)
            .join("config.xml")
    }
}
