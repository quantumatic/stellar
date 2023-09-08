#![cfg(feature = "debug")]

use std::fs;

use stellar_manifest::parse_manifest;

use crate::log::log_error;

pub fn command(filepath: &str) {
    match fs::read_to_string(filepath) {
        Err(..) => {
            log_error(format!("cannot read the file {filepath}"));
        }
        Ok(source) => match parse_manifest(source) {
            Err(err) => {
                log_error(format!(
                    "cannot parse the manifest file due to the error: {err}"
                ));
            }
            Ok(manifest) => {
                println!("{manifest:?}");
            }
        },
    }
}
