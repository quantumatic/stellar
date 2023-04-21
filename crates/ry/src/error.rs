#[macro_export]
macro_rules! mytry {
    ($e:expr) => {{
        let e0 = $e;

        #[allow(unused_qualifications)]
        if let Err(e) = e0 {
            println!(
                "
>===============================================================<

We are sorry :(

This is an internal compiler error.
Feel free to report it on https://github.com/abs0luty/ry/issues.

Info:
    [Trace]: happened in source file: {}, line: {}
    [Err object]: {:?}
    [Version]: {}
    [Dir]: {}
    [Exe path]: {}
    [OS]: {}

>===============================================================<",
                file!(),
                line!(),
                e,
                env!("CARGO_PKG_VERSION"),
                std::env::current_dir()
                    .map(|d| d.display().to_string())
                    .unwrap_or("error when getting current dir".to_owned()),
                std::env::current_exe()
                    .map(|d| d.display().to_string())
                    .unwrap_or("error when getting current exe".to_owned()),
                std::env::consts::OS
            );
            std::process::exit(1);
        } else {
            e0.unwrap()
        }
    }};
}
