# Unreleased

## Changed

- ([#5](https://github.com/acfoltzer/cargo-fund/pull/5)) The program now looks for the Github API token in the `CARGO_FUND_GITHUB_API_TOKEN` environment variable rather than `GITHUB_API_TOKEN` in order to better support privilege separation.
- ([#6](https://github.com/acfoltzer/cargo-fund/pull/6)) Increased timeout on HTTP client operations to 60 seconds.

# 0.1.0

Initial release.
