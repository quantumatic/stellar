use std::io::Write;

use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[allow(dead_code)]
pub fn log_info(prefix: impl AsRef<str>, message: impl AsRef<str>) {
    log_with_prefix(format!("{:>width$}", prefix.as_ref(), width = 12), message);
}

#[allow(dead_code)]
pub fn log_error(message: impl AsRef<str>) {
    let mut stdout = StandardStream::stderr(ColorChoice::Always);

    stdout
        .set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Red)))
        .unwrap();
    write!(&mut stdout, "error: ").unwrap();
    stdout.set_color(ColorSpec::new().set_fg(None)).unwrap();
    write!(&mut stdout, "{}", message.as_ref()).unwrap();
}

#[allow(dead_code)]
fn log_with_prefix(prefix: impl AsRef<str>, message: impl AsRef<str>) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    stdout
        .set_color(
            ColorSpec::new()
                .set_bold(true)
                .set_italic(true)
                .set_fg(Some(Color::Green)),
        )
        .unwrap();
    write!(&mut stdout, "{} ", prefix.as_ref()).unwrap();
    stdout
        .set_color(ColorSpec::new().set_fg(Some(Color::White)))
        .unwrap();
    writeln!(&mut stdout, "{}", message.as_ref()).unwrap();
    stdout.set_color(ColorSpec::new().set_fg(None)).unwrap();
}
