//! Adapted from the `cargo_tree::args` module.

use serde::Deserialize;
use std::path::PathBuf;
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(Deserialize, Debug)]
pub struct Env {
    #[serde(rename = "cargo_fund_github_api_token")]
    pub github_api_token: Option<String>,
}

#[derive(StructOpt)]
#[structopt(bin_name = "cargo")]
pub enum Opts {
    #[structopt(
    name = "fund",
    setting = AppSettings::UnifiedHelpMessage,
    setting = AppSettings::DeriveDisplayOrder,
    setting = AppSettings::DontCollapseArgsInUsage
    )]
    /// Display funding links for workspace dependencies
    Fund(Args),
}

#[derive(StructOpt)]
pub struct Args {
    /// Github API token, which must have the scope `public_repo`. This option overrides the token
    /// provided in the `CARGO_FUND_GITHUB_API_TOKEN` environment variable.
    #[structopt(long = "github-api-token", value_name = "TOKEN")]
    pub github_api_token: Option<String>,
    #[structopt(long = "manifest-path", value_name = "PATH", parse(from_os_str))]
    /// Path to Cargo.toml
    pub manifest_path: Option<PathBuf>,
    #[structopt(long = "verbose", short = "v", parse(from_occurrences))]
    /// Use verbose output (-vv very verbose/build.rs output)
    pub verbose: u32,
    #[structopt(long = "quiet", short = "q")]
    /// No output printed to stdout other than the funding information
    pub quiet: bool,
    #[structopt(long = "color", value_name = "WHEN")]
    /// Coloring: auto, always, never
    pub color: Option<String>,
    #[structopt(short = "Z", value_name = "FLAG")]
    /// Unstable (nightly-only) flags to Cargo
    pub unstable_flags: Vec<String>,
}
