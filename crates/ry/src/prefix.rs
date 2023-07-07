use leftpad_str::leftpad;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub(crate) fn log_with_left_padded_prefix<P, M>(prefix: P, message: M)
where
    P: AsRef<str>,
    M: AsRef<str>,
{
    log_with_prefix(leftpad(&format!("{} ", prefix.as_ref()), 15, ' '), message);
}

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
