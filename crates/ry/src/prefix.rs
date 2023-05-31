use std::{fs::metadata, fs::File, io::Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub(crate) fn log_with_prefix<P, M>(prefix: P, message: M)
where
    P: AsRef<str>,
    M: AsRef<str>,
{
    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    stdout
        .set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Blue)))
        .expect("Cannot set fg color to blue for current log");
    write!(&mut stdout, "{}", prefix.as_ref()).expect("Cannot write the current log message");
    stdout
        .set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::White)))
        .expect("Cannot set fg color to white for current log");
    writeln!(&mut stdout, "{}", message.as_ref()).expect("Cannot write the current log message");
    stdout
        .set_color(ColorSpec::new().set_fg(None))
        .expect("Cannot set fg color to white for current log");
}

pub(crate) fn create_unique_file(name: &str, extension: &str) -> (String, File) {
    let mut path = name.to_owned() + "." + extension;
    let mut idx = 2;

    while metadata(path.clone()).is_ok() {
        path = name.to_owned() + &format!(" ({})", idx) + "." + extension;
        idx += 1;
    }

    (path.clone(), File::create(path).expect("Err"))
}
