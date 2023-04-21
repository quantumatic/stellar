use std::{fs::metadata, fs::File, io::Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub(crate) fn log_with_prefix(prefix: &str, message: &str) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true));
    write!(&mut stdout, "{}", prefix);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)));
    writeln!(&mut stdout, "{}", message);
}

pub(crate) fn create_unique_file(name: &str, extension: &str) -> (String, File) {
    let first = true;
    let mut path = name.to_owned() + "." + extension;
    let mut idx = 2;

    while metadata(path.clone()).is_ok() {
        if first {
            path = name.to_owned() + &format!(" ({})", idx) + "." + extension;
        } else {
            path = name.to_owned() + &format!(" ({})", idx) + "." + extension;
        }

        idx += 1;
    }

    (path.clone(), File::create(path).expect("Err"))
}
