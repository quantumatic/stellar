mod project_template;

use clap::{arg, Command};
use ry_report::ReporterState;

fn cli() -> Command {
    Command::new("lapis")
        .about("Lapis - official Ry package manager client")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("new")
                .about("Create new lapis project")
                .arg(arg!(<NAME> "project name"))
                .arg_required_else_help(true),
        )
}

#[allow(clippy::single_match)]
fn main() {
    let reporter = ReporterState::default();

    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("new", sub_matches)) => {
            let name = sub_matches.get_one::<String>("NAME").unwrap();

            project_template::create_lapis_project(name, &reporter);
        }
        _ => {}
    }
}
