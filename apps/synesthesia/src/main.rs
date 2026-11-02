use std::{
    ffi::OsString,
    io::{self, Write},
    path::Path,
    process::ExitCode,
};
use synesthesia_core::pipeline::PipelineConfig;

fn run(args: &[OsString], out: &mut impl Write, err: &mut impl Write) -> u8 {
    match args {
        [] => help(out),
        [arg] if arg == "--help" || arg == "-h" => help(out),
        [arg] if arg == "--version" => {
            let _ = writeln!(out, "synesthesia {}", env!("CARGO_PKG_VERSION"));
            0
        }
        [flag, path] if flag == "--check-config" => match PipelineConfig::load(Path::new(path)) {
            Ok(_) => {
                let _ = writeln!(
                    out,
                    "Pipeline configuration valid (schema 2). Credentials, devices, models and live runtime remain unverified."
                );
                0
            }
            Err(error) => {
                let _ = writeln!(err, "{error}");
                2
            }
        },
        _ => {
            let _ = writeln!(
                err,
                "Unsupported command. Use --help; live desktop runtime is not integrated yet."
            );
            2
        }
    }
}
fn help(out: &mut impl Write) -> u8 {
    let _ = writeln!(
        out,
        "Synesthesia: audio -> speech-to-text -> optional translation -> desktop captions\nUsage: synesthesia --help | --version | --check-config FILE\nSchema 2 selects microphone/system audio and local/Fun-ASR recognition independently.\nRelative model paths resolve from the configuration file directory.\nThis build provides configuration validation; live capture, provider orchestration and transparent windows are not integrated yet."
    );
    0
}
fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    ExitCode::from(run(
        &args,
        &mut io::stdout().lock(),
        &mut io::stderr().lock(),
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unsupported_runtime_does_not_echo_arguments_or_claim_readiness() {
        let mut out = Vec::new();
        let mut err = Vec::new();
        assert_eq!(
            run(
                &["--run".into(), "private-input".into()],
                &mut out,
                &mut err
            ),
            2
        );
        let error = String::from_utf8(err).unwrap();
        assert!(out.is_empty());
        assert!(error.contains("not integrated"));
        assert!(!error.contains("private-input"));
    }
}
