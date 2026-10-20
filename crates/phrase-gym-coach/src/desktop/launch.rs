use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub struct LaunchOptions {
    pub smoke_report: Option<PathBuf>,
    pub smoke_input: Option<super::smoke_intake::SmokeInput>,
}

pub fn parse(arguments: impl IntoIterator<Item = OsString>) -> Result<LaunchOptions, String> {
    let mut arguments = arguments.into_iter();
    let mut options = LaunchOptions {
        smoke_report: None,
        smoke_input: None,
    };
    while let Some(argument) = arguments.next() {
        // Older macOS versions append a process serial number when Finder launches an app.
        if argument.to_string_lossy().starts_with("-psn_") {
            continue;
        }
        if argument == "--smoke-test" {
            let report = arguments.next().ok_or("--smoke-test needs a report path")?;
            options.smoke_report = Some(PathBuf::from(report));
        } else if argument == "--smoke-input" {
            let server = arguments
                .next()
                .ok_or("--smoke-input needs a Server URL")?
                .into_string()
                .map_err(|_| "Smoke Server URL must be Unicode")?;
            let server = crate::transfer::client::server_origin(&server)?;
            let directory = PathBuf::from(
                arguments
                    .next()
                    .ok_or("--smoke-input needs a fresh directory")?,
            );
            options.smoke_input = Some(super::smoke_intake::SmokeInput { server, directory });
        } else {
            return Err(format!("unknown argument {}", argument.to_string_lossy()));
        }
    }
    if options.smoke_input.is_some() && options.smoke_report.is_none() {
        return Err(
            "--smoke-input requires --smoke-test and uses only bundled synthetic input/fake CLI"
                .into(),
        );
    }
    Ok(options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_controlled_smoke_options() {
        assert_eq!(
            parse([]),
            Ok(LaunchOptions {
                smoke_report: None,
                smoke_input: None
            })
        );
        let parsed = parse(["--smoke-test".into(), "/tmp/Ünï report.json".into()]).unwrap();
        assert_eq!(
            parsed.smoke_report,
            Some(PathBuf::from("/tmp/Ünï report.json"))
        );
        assert!(parse(["--smoke-test".into()]).is_err());
        assert_eq!(
            parse(["-psn_0_12345".into()]),
            Ok(LaunchOptions {
                smoke_report: None,
                smoke_input: None
            })
        );
        assert!(parse(["--verbose".into()]).is_err());
        assert!(
            parse([
                "--smoke-input".into(),
                "http://127.0.0.1:8000".into(),
                "/tmp/fresh".into()
            ])
            .is_err()
        );
        let options = parse([
            "--smoke-test".into(),
            "/tmp/report.json".into(),
            "--smoke-input".into(),
            "http://127.0.0.1:8000".into(),
            "/tmp/fresh".into(),
        ])
        .unwrap();
        assert!(options.smoke_input.is_some());
    }
}
