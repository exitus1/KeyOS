// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::File,
    io::{self, IsTerminal, Read, Write},
    path::PathBuf,
    process::{Command, ExitStatus, Stdio},
    sync::LazyLock,
};

use anyhow::Context;
use once_cell::sync::Lazy;

use crate::builder::cargo;
use crate::project_root;
use crate::TARGET_TRIPLE_KEYOS;

pub static GIT_TIMESTAMP: Lazy<String> = Lazy::new(|| {
    let git_timestamp = std::process::Command::new("git")
        .args(["log", "-1", "--pretty=%ct"])
        .output()
        .expect("Could not get last commit date");
    if !git_timestamp.status.success() {
        panic!("Git log unsuccesful: {:?}", git_timestamp);
    }
    String::from_utf8(git_timestamp.stdout.trim_ascii().to_vec()).unwrap()
});

const TOOLCHAIN_RELEASE_URL_KEYOS: &str =
    "https://api.github.com/repos/Foundation-Devices/rust-keyos/releases";

static TOOLCHAIN_RELEASE_URLS: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    HashMap::from([(TARGET_TRIPLE_KEYOS.to_owned(), TOOLCHAIN_RELEASE_URL_KEYOS.to_owned())])
});

/// Since we use the same TARGET for all calls to `build()`,
/// cache it inside an atomic boolean. If this is `true` then
/// it means we can assume the check passed already.
static DONE_COMPILER_CHECK: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Ensure we have a compatible compiler toolchain. We use a new Target,
/// and we want to give the user a friendly way of installing the latest
/// Rust toolchain.
pub(crate) fn ensure_compiler(target: Option<&str>, force_install: bool, remove_existing: bool) {
    if DONE_COMPILER_CHECK.load(std::sync::atomic::Ordering::SeqCst) {
        return;
    }

    // No need to do anything when targeting x86
    let Some(target) = target else {
        return;
    };
    // If the sysroot exists, then we're good.
    if let Some(path) = get_sysroot(Some(target)) {
        let mut version_path = PathBuf::from(&path);
        version_path.push("lib");
        version_path.push("rustlib");
        version_path.push(target);
        if remove_existing {
            println!("Target path exists, removing it");
            std::fs::remove_dir_all(version_path)
                .unwrap_or_else(|e| panic!("unable to remove existing toolchain: {}", e));
        } else {
            DONE_COMPILER_CHECK.store(true, std::sync::atomic::Ordering::SeqCst);
            return;
        }
    }

    // Since no sysroot exists, we must download a new one.
    let toolchain_path =
        PathBuf::from(get_sysroot(None).unwrap_or_else(|| panic!("default toolchain not installed")));
    // If the terminal is a tty, or if toolchain installation is forced,
    // download the latest toolchain.
    if !std::io::stdin().is_terminal() && !force_install {
        panic!("Toolchain for {target} not found");
    }

    let ver = rustc_version::version_meta().unwrap();
    let is_nightly = matches!(ver.channel, rustc_version::Channel::Nightly);

    // Ask the user if they want to install the toolchain.
    let mut buffer = String::new();
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    if force_install {
        println!("Downloading toolchain");
    } else {
        println!("Error: Toolchain for {target} was not found on this system!");
        loop {
            print!("Would you like this program to attempt to download and install it?   [Y/n] ");
            stdout.flush().unwrap();
            buffer.clear();
            stdin.read_line(&mut buffer).unwrap();

            let trimmed = buffer.trim();

            if trimmed == "n" || trimmed == "N" {
                panic!("Please install the {target} toolchain");
            }

            if trimmed == "y" || trimmed == "Y" || trimmed.is_empty() {
                break;
            }
            println!();
        }
    }

    let url = TOOLCHAIN_RELEASE_URLS
        .get(target)
        .unwrap_or_else(|| panic!("Can't find toolchain URL for target {target}"));
    let j: serde_json::Value = ureq::get(url)
        .set("User-Agent", "xous-core")
        .set("Accept", "application/vnd.github.v3+json")
        .call()
        .unwrap()
        .into_json()
        .unwrap();
    let releases = j.as_array().unwrap();
    let target_prefix = format!(
        "{}.{}.{}{}",
        ver.semver.major,
        ver.semver.minor,
        ver.semver.patch,
        if is_nightly { "-nightly" } else { "" }
    );
    let Some((_release, toolchain_url)) = releases
        .iter()
        .filter_map(|r| {
            let keys = r.as_object()?;
            let release = keys.get("tag_name")?.as_str()?;
            if !release.starts_with(&target_prefix) {
                return None;
            }
            let download_url = keys.get("assets")?.as_array()?.first()?.get("url")?.as_str()?;
            Some((release.to_owned(), download_url.to_owned()))
        })
        .max()
    else {
        panic!("No toolchains found for Rust {target_prefix}");
    };

    println!("Attempting to install toolchain for {target} into {toolchain_path:?}");
    println!("Downloading toolchain from {toolchain_url}...");

    print!("Download in progress...");
    stdout.flush().unwrap();
    let mut zip_data = vec![];
    {
        let agent = ureq::builder().build();
        let mut freader =
            agent.get(&toolchain_url).set("Accept", "application/octet-stream").call().unwrap().into_reader();
        freader.read_to_end(&mut zip_data).unwrap();
        println!();
    }
    println!("Download successful. Total data size is {} bytes", zip_data.len());

    // Clean to avoid conflicts with newly downloaded stdlib
    let status = Command::new(cargo()).current_dir(project_root()).args(["clean"]).status().unwrap();
    if !status.success() {
        panic!("`cargo clean` failed");
    }

    println!("Extracting toolchain to {toolchain_path:?}...");
    let archive_data = std::io::Cursor::new(zip_data);
    let mut archive = zip::ZipArchive::new(archive_data).unwrap();
    for i in 0..archive.len() {
        let mut entry_in_archive = archive.by_index(i).unwrap();
        let output_path = toolchain_path.join(entry_in_archive.enclosed_name().unwrap());
        if entry_in_archive.is_dir() {
            std::fs::create_dir_all(&output_path).unwrap();
        } else {
            // Create the parent directory if necessary
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            let mut outfile = std::fs::File::create(&output_path).unwrap();
            std::io::copy(&mut entry_in_archive, &mut outfile).unwrap();
        }
    }

    println!("Toolchain successfully installed");

    DONE_COMPILER_CHECK.store(true, std::sync::atomic::Ordering::SeqCst);
}

fn get_target_toolchain(target: Option<&str>) -> (bool, String, Option<PathBuf>, Option<PathBuf>) {
    let mut args = vec!["--print", "sysroot"];
    if let Some(target) = target {
        args.push("--target");
        args.push(target);
    }

    let sysroot_cmd = Command::new("rustc")
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .args(&args)
        .spawn()
        .expect("could not run rustc");
    let sysroot_output = sysroot_cmd.wait_with_output().unwrap();
    let have_toolchain = sysroot_output.status.success();

    let toolchain_path = String::from_utf8(sysroot_output.stdout).unwrap().trim().to_owned();

    let target_path = target.map(|target| {
        let mut target_path = PathBuf::from(&toolchain_path);
        target_path.push("lib");
        target_path.push("rustlib");
        target_path.push(target);

        target_path
    });

    let version_path = target_path.as_ref().cloned().map(|mut target_path| {
        target_path.push("RUST_VERSION");
        target_path
    });

    (have_toolchain, toolchain_path, target_path, version_path)
}

/// Return the sysroot for the given target. If the target does not exist,
/// return None.
fn get_sysroot(target: Option<&str>) -> Option<String> {
    let (have_toolchain, toolchain_path, _, version_path) = get_target_toolchain(target);

    // Look for the "RUST_VERSION" file to ensure it's compatible with this version.
    if let Some(version_path) = version_path {
        if let Ok(mut vp) = File::open(&version_path) {
            let mut version_str = String::new();
            vp.read_to_string(&mut version_str).expect("Unable to get version string");

            let rustc_version = rustc_version::version_meta().unwrap();
            let rustc_version_str = format!("{}", rustc_version.semver);
            if version_str.trim() != rustc_version_str.trim() {
                println!(
                    "Version upgrade. Compiler is version {}, the installed toolchain is for {}",
                    version_str.trim(),
                    rustc_version_str.trim()
                );
                return None;
            }
        } else {
            println!("Outdated toolchain installed.");
            return None;
        }
    }

    if have_toolchain {
        Some(toolchain_path)
    } else {
        None
    }
}

pub(crate) fn is_target_installed(target: &str) -> bool {
    let (is_installed, _, target_path, _) = get_target_toolchain(Some(target));
    is_installed && target_path.map(|pb| pb.exists()).unwrap_or(false)
}

/// dynamic link editor paths
/// return tuple with values: (
///     - env variable name similar to <prefix>PATH, this name is different for different OS
///     - path to the libraries, if we can find it without reading env var <prefix>PATH
/// )
pub(crate) fn get_dl_path() -> anyhow::Result<(String, String)> {
    match std::env::consts::OS {
        "macos" => {
            let xcode_cmd = Command::new("xcode-select")
                .stderr(Stdio::null())
                .stdout(Stdio::piped())
                .args(["--print-path"])
                .spawn()
                .context("Unable to run xcode-select")?;
            let dl_output = xcode_cmd.wait_with_output().context("Unable to read xcode-select output")?;
            let dl_path =
                String::from_utf8(dl_output.stdout).context("Unable to find dl path")?.trim().to_owned();

            let mut paths = vec![
                format!("{}/Toolchains/XcodeDefault.xctoolchain/usr/lib", dl_path),
                format!("{}/usr/lib", dl_path),
            ];

            if let Ok(existing) = std::env::var("DYLD_FALLBACK_LIBRARY_PATH") {
                if !existing.is_empty() {
                    paths.push(existing);
                }
            }

            Ok(("DYLD_FALLBACK_LIBRARY_PATH".into(), paths.join(":")))
        }
        "linux" => Ok(("LD_LIBRARY_PATH".into(), std::env::var("LD_LIBRARY_PATH").unwrap_or_default())),
        "windows" => Ok(("PATH".into(), std::env::var("PATH").unwrap_or_default())),
        _ => Err(anyhow::anyhow!("this OS is not supported")),
    }
}

pub(crate) struct Cosign2 {
    config_path: Option<PathBuf>,
}

impl Cosign2 {
    pub(crate) fn new(config_path: Option<PathBuf>) -> io::Result<Self> {
        // Make sure that the cosign2 can be executed.
        Command::new("cosign2").stdout(Stdio::null()).stderr(Stdio::null()).status().inspect_err(|_| {
            eprintln!("Couldn't run `cosign2`. Is `cosign2` tool installed?");
            eprintln!("Visit https://github.com/Foundation-Devices/cosign2 for more info");
        })?;
        if let Some(ref config_path) = config_path {
            let config_path_str = config_path.to_str().expect("Path should be convertible to str");
            if !std::fs::exists(config_path_str)? {
                eprintln!("Could not find `cosign2` config at {config_path_str}");
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Could not find `cosign2` config at {config_path_str}",
                ));
            }
        }

        Ok(Self { config_path })
    }

    pub(crate) fn sign<A>(&self, args: A) -> io::Result<ExitStatus>
    where
        A: IntoIterator,
        A::Item: AsRef<OsStr>,
    {
        self.run(Cosign2Cmd::Sign, args)
    }

    #[allow(dead_code)]
    pub(crate) fn dump<A>(&self, args: A) -> io::Result<ExitStatus>
    where
        A: IntoIterator,
        A::Item: AsRef<OsStr>,
    {
        self.run(Cosign2Cmd::Dump, args)
    }

    pub(crate) fn run<A>(&self, cmd: Cosign2Cmd, args: A) -> io::Result<ExitStatus>
    where
        A: IntoIterator,
        A::Item: AsRef<OsStr>,
    {
        let mut command = Command::new("cosign2");
        let command = command.arg(&cmd.to_string()).args(args);

        let command = if let Some(ref config_path) = self.config_path {
            let config_path_str = config_path.to_str().expect("Path should be convertible to str");
            command.args(["-c", config_path_str])
        } else {
            command
        };

        command.status()
    }
}

pub(crate) enum Cosign2Cmd {
    Sign,
    Dump,
}

impl std::fmt::Display for Cosign2Cmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cmd = match self {
            Cosign2Cmd::Sign => "sign",
            Cosign2Cmd::Dump => "dump",
        };
        write!(f, "{}", cmd)
    }
}
