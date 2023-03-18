use std::{process::exit, fs::{self, File}, path::Path, env, io::Write};
use ry_report::ReporterState;

fn check_project_name(name: &str, reporter: &ReporterState) {
    name.chars().for_each(|c| {
        if !(c.is_ascii_digit() || c.is_ascii_lowercase() || c == '_') {
            reporter.emit_global_error(&format!("project name must consist only of ascii lowercase, ascii digit or `_` characters! but somewhy i see `{c}`."));
            exit(1);
        }
    })
}

fn create_file(name: &str, reporter: &ReporterState) -> File {
	match File::create(name) {
		Ok(file) => file,
		Err(_) => {
			reporter.emit_global_error(
		  	  &format!("cannot create file `{name}`."));
				exit(1);
		}
	}
}

pub fn create_lapis_project(name: &str, reporter: &ReporterState) {
	check_project_name(name, reporter);

	if Path::new(name).exists() {
		reporter.emit_global_error(
		    "folder with this name of this project already exists.");
		exit(1);
	}

	fs::create_dir_all(name).unwrap_or_else(|_| {
    reporter.emit_global_error(
        &format!("cannot create project directory `{name}`."));
    exit(1);
	});

	env::set_current_dir(name).unwrap_or_else(|_| {
    reporter.emit_global_error(
        "cannot go into project dir.");
    exit(1);
	});

	let mut lapis_config = create_file(".lapis.json", reporter);
	write!(lapis_config, "{{\n  \"project_name\": \"{name}\"\n}}\n").unwrap();

	["test", "bin", "lib"].iter().for_each(|f| {
		fs::create_dir_all(f).unwrap();
	});

	let mut main_file = create_file("bin/main.ry", reporter);
	write!(main_file, "pub fun main() {{\n\n}}").unwrap();
}