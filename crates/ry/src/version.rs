use ry_info::{
    COMPILER_OWNER, COMPILER_VERSION, REPOSITORY, RY_REPOSITORY_OWNER, RY_REPOSITORY_VERSION,
    STD_OWNER, STD_VERSION,
};

pub fn compiler_version_command() {
    println!(
        "Compiler version: {}.\nCurrently owned by: {}\nRepository: {}",
        COMPILER_VERSION, COMPILER_OWNER, REPOSITORY
    );
}

pub fn std_version_command() {
    println!(
        "Standard library version: {}\nCurrently owned by: {}\nRepository: {}",
        STD_VERSION, STD_OWNER, REPOSITORY
    );
}

pub fn package_manager_version_command() {
    println!(
        "Package manager version: {}\nCurrently owned by: {}",
        RY_REPOSITORY_VERSION, RY_REPOSITORY_OWNER
    );
}
