use std::{
    fs::{self, File},
    io::Write,
    process::exit,
};

use crate::prefix::log_with_prefix;

fn check_project_name(name: &str) -> Option<usize> {
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

pub fn command(project_name: &str) {
    if let Some(e) = check_project_name(project_name) {
        log_with_prefix("error", ": cannot create project with a given name");
        log_with_prefix(
            "note",
            format!(
                ": character `{}` doesn't correspond to the pattern: {}",
                project_name.chars().nth(e).unwrap_or_else(|| panic!(
                    "Cannot get the {}-nth character of project name",
                    e
                )),
                if e != 0 {
                    "`0` to `9`, `a` to `z`, `A` to `Z` or `_`"
                } else {
                    "`a` to `z`, `A` to `Z` or `_`"
                }
            ),
        );
        exit(1);
    }

    fs::create_dir(project_name).unwrap_or_else(|_| {
        log_with_prefix("error", ": cannot create project folder");
        exit(1);
    });

    fs::create_dir(format!("{}/bin", project_name)).unwrap_or_else(|_| {
        log_with_prefix("error", ": cannot create `bin` package folder");
        exit(1);
    });

    let mut main_file =
        File::create(format!("{}/bin/main.ry", project_name)).unwrap_or_else(|_| {
            log_with_prefix("error", ": cannot create `bin/main.ry`");
            exit(1);
        });

    main_file
        .write_all(b"fun main() {\n  println(\"Hello, world!\");\n}")
        .unwrap_or_else(|_| {
            log_with_prefix("error", ": cannot write to `bin/main.ry`");
            exit(1);
        });

    let mut project_file =
        File::create(format!("{}/project.json", project_name)).unwrap_or_else(|_| {
            log_with_prefix("error", ": cannot create `project.json`");
            exit(1);
        });

    project_file
        .write_all(
            format!(
                "{{
\"name\": \"{}\",
\"version\": \"0.0.1\",
\"dependencies\": []
}}",
                project_name
            )
            .as_bytes(),
        )
        .unwrap_or_else(|_| {
            log_with_prefix("error", ": cannot write to `project.json`");
            exit(1);
        });

    log_with_prefix("   Created ", project_name);
}
