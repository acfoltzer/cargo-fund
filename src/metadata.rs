//! Adapted from the `cargo_tree::metadata` module.

use crate::args::Args;
use anyhow::{anyhow, Context, Error};
use cargo_metadata::Metadata;
use std::env;
use std::ffi::OsString;
use std::process::{Command, Stdio};

pub fn get(args: &Args) -> Result<Metadata, Error> {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));

    let mut command = Command::new(cargo);
    command.arg("metadata").arg("--format-version").arg("1");

    if args.quiet {
        command.arg("-q");
    }

    if let Some(path) = &args.manifest_path {
        command.arg("--manifest-path").arg(path);
    }

    for _ in 0..args.verbose {
        command.arg("-v");
    }

    if let Some(color) = &args.color {
        command.arg("--color").arg(color);
    }

    for flag in &args.unstable_flags {
        command.arg("-Z").arg(flag);
    }

    let output = output(&mut command, "cargo metadata")?;

    serde_json::from_str(&output).context("error parsing cargo metadata output")
}

fn output(command: &mut Command, job: &str) -> Result<String, Error> {
    let output = command
        .stderr(Stdio::inherit())
        .output()
        .with_context(|| format!("error running {}", job))?;

    if !output.status.success() {
        return Err(anyhow!("{} returned {}", job, output.status));
    }

    String::from_utf8(output.stdout).with_context(|| format!("error parsing {} output", job))
}
