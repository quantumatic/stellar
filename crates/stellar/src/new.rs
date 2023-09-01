use std::{
    fs::{self, File},
    io::Write,
    process::exit,
};

use crate::prefix::log;

fn check_package_name(name: &str) -> Option<usize> {
    let mut chars = name.chars();
    let first_char = chars.next()?;

    if !first_char.is_ascii_alphabetic() && first_char != '_' {
        return Some(0);
    }

    for (idx, char) in chars.enumerate() {
        if !char.is_ascii_alphabetic() && !char.is_ascii_alphanumeric() && char != '_' {
            return Some(idx + 1);
        }
    }

    None
}

pub fn command(package_name: &str) {
    if let Some(e) = check_package_name(package_name) {
        log("error", ": cannot create package with a given name");
        log(
            "note",
            format!(
                ": character `{}` doesn't correspond to the pattern: {}",
                package_name
                    .chars()
                    .nth(e)
                    .unwrap_or_else(|| panic!("Cannot get the {e}-nth character of package name",)),
                if e == 0 {
                    "`a` to `z`, `A` to `Z` or `_`"
                } else {
                    "`0` to `9`, `a` to `z`, `A` to `Z` or `_`"
                }
            ),
        );
        exit(1);
    }

    fs::create_dir(package_name).unwrap_or_else(|_| {
        log("error", ": cannot create package folder");
        exit(1);
    });

    fs::create_dir(format!("{package_name}/bin")).unwrap_or_else(|_| {
        log("error", ": cannot create `bin` package folder");
        exit(1);
    });

    let mut main_file = File::create(format!("{package_name}/bin/main.sr")).unwrap_or_else(|_| {
        log("error", ": cannot create `bin/main.sr`");
        exit(1);
    });

    main_file
        .write_all(b"fun main() {\n  println(\"Hello, world!\");\n}")
        .unwrap_or_else(|_| {
            log("error", ": cannot write to `bin/main.sr`");
            exit(1);
        });

    let mut package_file =
        File::create(format!("{package_name}/package.json")).unwrap_or_else(|_| {
            log("error", ": cannot create `package.json`");
            exit(1);
        });

    package_file
        .write_all(
            format!(
                "{{
\"name\": \"{package_name}\",
\"version\": \"0.0.1\",
\"dependencies\": []
}}"
            )
            .as_bytes(),
        )
        .unwrap_or_else(|_| {
            log("error", ": cannot write to `package.json`");
            exit(1);
        });

    log("   Created ", package_name);
}
