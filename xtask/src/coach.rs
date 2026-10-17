use std::path::{Path, PathBuf};

use crate::tools::{Failure, Workspace};

const APP: &str = "Phrase Gym Coach";
pub const BINARY: &str = "phrase-gym-coach";
const FAKE_CHILD: &str = "phrase-gym-fake-child";
// The oldest macOS that every component supports: Claude Code requires 13.0, everything else less.
const MACOS_MINIMUM: &str = "13.0";

pub fn io(context: &str) -> impl Fn(std::io::Error) -> Failure + '_ {
    move |error| Failure::new(format!("{context}: {error}"))
}

// A development layout, not a distributable package: unsigned, with the fake CLI bundled so the
// packaged process-supervision checks can run from the installed location.
pub fn bundle(workspace: &Workspace, out: &Path) -> Result<PathBuf, Failure> {
    workspace.cargo(&[
        "build",
        "--release",
        "--locked",
        "--package",
        BINARY,
        "--features",
        "desktop",
        "--bins",
    ])?;
    let release = workspace.root.join("target/release");
    let executable = |name: &str| release.join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
    let (launcher, binaries, resources) = layout(out)?;
    for directory in [&binaries, &resources] {
        std::fs::create_dir_all(directory).map_err(io("creating the bundle"))?;
    }
    let installed = binaries.join(executable(BINARY).file_name().unwrap_or_default());
    std::fs::copy(executable(BINARY), &installed).map_err(io("copying Coach"))?;
    let fake = executable(FAKE_CHILD);
    std::fs::copy(&fake, resources.join(fake.file_name().unwrap_or_default()))
        .map_err(io("copying the fake CLI"))?;
    if cfg!(target_os = "macos") {
        std::fs::write(
            out.join(format!("{APP}.app/Contents/Info.plist")),
            info_plist(),
        )
        .map_err(io("writing Info.plist"))?;
    }
    println!(
        "xtask: Coach bundle {} ({} KiB)",
        launcher.display(),
        size(&launcher) / 1024
    );
    Ok(launcher)
}

fn layout(out: &Path) -> Result<(PathBuf, PathBuf, PathBuf), Failure> {
    if cfg!(target_os = "macos") {
        let app = out.join(format!("{APP}.app"));
        let contents = app.join("Contents");
        Ok((app, contents.join("MacOS"), contents.join("Resources")))
    } else if cfg!(windows) {
        let folder = out.join(APP);
        let executable = folder.join(format!("{BINARY}.exe"));
        Ok((executable, folder.clone(), folder.join("resources")))
    } else {
        Err(Failure::new("Coach bundles are built on macOS or Windows"))
    }
}

pub fn info_plist() -> String {
    let version = env!("CARGO_PKG_VERSION");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key><string>en</string>
  <key>CFBundleDisplayName</key><string>{APP}</string>
  <key>CFBundleExecutable</key><string>{BINARY}</string>
  <key>CFBundleIdentifier</key><string>io.github.andgineer.phrase-gym-coach</string>
  <key>CFBundleName</key><string>{APP}</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>{version}</string>
  <key>CFBundleVersion</key><string>{version}</string>
  <key>LSMinimumSystemVersion</key><string>{MACOS_MINIMUM}</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
"#
    )
}

fn size(path: &Path) -> u64 {
    if path.is_dir() {
        std::fs::read_dir(path).map_or(0, |entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| size(&entry.path()))
                .sum()
        })
    } else {
        std::fs::metadata(path).map_or(0, |metadata| metadata.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bundle_declares_identity_executable_and_minimum_macos() {
        let plist = info_plist();
        for expected in [
            "<string>io.github.andgineer.phrase-gym-coach</string>",
            "<key>CFBundleExecutable</key><string>phrase-gym-coach</string>",
            "<key>LSMinimumSystemVersion</key><string>13.0</string>",
            "<key>NSHighResolutionCapable</key><true/>",
        ] {
            assert!(plist.contains(expected), "{expected}");
        }
    }
}
