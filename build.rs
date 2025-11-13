// Copyright 2025 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use std::{cmp::Ordering, env, process::Command, str::FromStr};

const MIN_SPAN_LOCATIONS_VER: Version = Version::new(1, 88, 0);

fn main() {
    println!("cargo::rerun-if-changed=README.md");
    if matches!(rustc_version(), Ok(version) if version >= MIN_SPAN_LOCATIONS_VER) {
        println!("cargo::rustc-cfg=span_locations");
    }
}

fn rustc_version() -> Result<Version, Box<dyn std::error::Error>> {
    let output = Command::new(env::var("RUSTC")?).arg("--version").output()?;
    let stdout = String::from_utf8(output.stdout)?;
    let mut words = stdout.split_whitespace();
    words.next().ok_or("expected `rustc`")?;

    let version: Version = words.next().ok_or("expected version")?.parse()?;
    Ok(version)
}

#[derive(Debug, Default, Eq)]
struct Version {
    major: u16,
    minor: u16,
    patch: u16,
}

impl Version {
    const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl FromStr for Version {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // cspell:ignore splitn
        let mut values = s.splitn(3, ".").map(str::parse::<u16>);
        Ok(Self {
            major: values
                .next()
                .ok_or("no major version")?
                .map_err(|err| err.to_string())?,
            minor: values
                .next()
                .ok_or("no minor version")?
                .map_err(|err| err.to_string())?,
            patch: values
                .next()
                .ok_or("no patch version")?
                .map_err(|err| err.to_string())?,
        })
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major && self.minor == other.minor && self.patch == other.patch
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        let cmp = self.major.cmp(&other.major);
        if cmp != Ordering::Equal {
            return cmp;
        }
        let cmp = self.minor.cmp(&other.minor);
        if cmp != Ordering::Equal {
            return cmp;
        }
        self.patch.cmp(&other.patch)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
