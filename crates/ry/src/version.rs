use ry_info::{
    COMPILER_OWNER, COMPILER_VERSION, REPOSITORY, RY_REPOSITORY_OWNER, RY_REPOSITORY_VERSION,
    STD_OWNER, STD_VERSION,
};

pub fn compiler_version_command() {
    println!(
        "Compiler version: {COMPILER_VERSION}.\nCurrently owned by: {COMPILER_OWNER}\nRepository: {REPOSITORY}",
    );
}

pub fn std_version_command() {
    println!(
        "Standard library version: {STD_VERSION}\nCurrently owned by: {STD_OWNER}\nRepository: {REPOSITORY}",
    );
}

pub fn package_manager_version_command() {
    println!(
        "Package manager version: {RY_REPOSITORY_VERSION}\nCurrently owned by: {RY_REPOSITORY_OWNER}",
    );
}
