use std::fs::{metadata, File};

pub(crate) fn create_unique_file(name: &str, extension: &str) -> (String, File) {
    let mut path = name.to_owned() + "." + extension;
    let mut idx = 2;

    while metadata(path.clone()).is_ok() {
        path = name.to_owned() + &format!(" ({})", idx) + "." + extension;
        idx += 1;
    }

    (path.clone(), File::create(path).expect("Err"))
}
