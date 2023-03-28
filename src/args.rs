//! Adapted from the `cargo_tree::args` module.

use clap::{ArgAction, Parser};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
pub struct Env {
    #[serde(rename = "cargo_fund_github_api_token")]
    pub github_api_token: Option<String>,
}

#[derive(Parser)]
#[clap(bin_name = "cargo")]
pub enum Opts {
    #[clap(name = "fund", dont_collapse_args_in_usage = true)]
    /// Display funding links for workspace dependencies
    Fund(Args),
}

#[derive(Parser)]
pub struct Args {
    /// Github API token, which must have the scope `public_repo`. This option overrides the token
    /// provided in the `CARGO_FUND_GITHUB_API_TOKEN` environment variable.
    #[clap(long = "github-api-token", value_name = "TOKEN")]
    pub github_api_token: Option<String>,
    #[clap(long = "manifest-path", value_name = "PATH", value_parser)]
    /// Path to Cargo.toml
    pub manifest_path: Option<PathBuf>,
    #[clap(long = "verbose", short = 'v', action = ArgAction::Count)]
    /// Use verbose output (-vv very verbose/build.rs output)
    pub verbose: u8,
    #[clap(long = "quiet", short = 'q')]
    /// No output printed to stdout other than the funding information
    pub quiet: bool,
    #[clap(long = "color", value_name = "WHEN")]
    /// Coloring: auto, always, never
    pub color: Option<String>,
    #[clap(short = 'Z', value_name = "FLAG")]
    /// Unstable (nightly-only) flags to Cargo
    pub unstable_flags: Vec<String>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Opts::command().debug_assert();
    }
}
