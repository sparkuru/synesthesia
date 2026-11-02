//! Shared honest command entry: validation is available; product runtime is not.
use crate::config::{Role, RuntimeRoot};
use std::{ffi::OsString, io::Write, path::PathBuf};

/// Returns a process exit code without logging user arguments or configuration.
pub fn run<I>(
    role: Role,
    arguments: I,
    executable: PathBuf,
    out: &mut impl Write,
    err: &mut impl Write,
) -> i32
where
    I: IntoIterator<Item = OsString>,
{
    let args: Vec<OsString> = arguments.into_iter().collect();
    if args.is_empty() || args == [OsString::from("--help")] || args == [OsString::from("-h")] {
        let _ = writeln!(
            out,
            "synesthesia {role:?}\nUsage: --help | --version | --check-config FILE [--runtime-root ABSOLUTE_DIRECTORY]\nRelative paths resolve from the executable directory (or explicit runtime root).\nConfiguration validation only: capture, inference, windows and WS are not integrated yet."
        );
        return 0;
    }
    if args == [OsString::from("--version")] {
        let _ = writeln!(out, "synesthesia {} {role:?}", env!("CARGO_PKG_VERSION"));
        return 0;
    }
    if !(args.len() == 2 || args.len() == 4)
        || args[0] != "--check-config"
        || (args.len() == 4 && args[2] != "--runtime-root")
    {
        let _ = writeln!(
            err,
            "Unsupported command. Use --help; product runtime is not implemented."
        );
        return 2;
    }
    let root = if args.len() == 4 {
        RuntimeRoot::new(PathBuf::from(&args[3]))
    } else {
        RuntimeRoot::from_executable(&executable)
    };
    let config = root.and_then(|root| {
        let config = root.load(&PathBuf::from(&args[1]), role)?;
        if let Some(inference) = config.inference() {
            root.resolve(&inference.asr_model)?;
            root.resolve(&inference.translation_model)?;
        }
        Ok(config)
    });
    match config {
        Ok(_) => {
            let _ = writeln!(
                out,
                "Configuration valid for {role:?}; model availability and runtime readiness are unverified."
            );
            0
        }
        Err(error) => {
            let _ = writeln!(err, "{error}");
            2
        }
    }
}

pub fn entry(role: Role) -> std::process::ExitCode {
    let executable = match std::env::current_exe() {
        Ok(path) => path,
        Err(_) => {
            eprintln!("Cannot determine runtime root.");
            return std::process::ExitCode::from(2);
        }
    };
    std::process::ExitCode::from(run(
        role,
        std::env::args_os().skip(1),
        executable,
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    ) as u8)
}
