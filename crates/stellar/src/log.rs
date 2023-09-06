use std::io::Write;

use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[allow(dead_code)]
pub fn log(prefix: impl AsRef<str>, message: impl AsRef<str>) {
    log_with_prefix(format!("{:>width$}", prefix.as_ref(), width = 12), message);
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
        .expect("Cannot set fg color to cyan for current log");
    write!(&mut stdout, "{} ", prefix.as_ref()).expect("Cannot write the current log message");
    stdout
        .set_color(ColorSpec::new().set_fg(Some(Color::White)))
        .expect("Cannot set fg color to white for current log");
    writeln!(&mut stdout, "{}", message.as_ref()).expect("Cannot write the current log message");
    stdout
        .set_color(ColorSpec::new().set_fg(None))
        .expect("Cannot set fg color to white for current log");
}
