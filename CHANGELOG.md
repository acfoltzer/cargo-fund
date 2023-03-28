# 0.2.3

## Changed

- Updated to `clap` 4.

## Fixed

- Updated some dependencies to clear Dependabot warnings. None of the warnings were applicable to how `cargo-fund` used those dependencies directly.

# 0.2.2

## Fixed

- Updated some dependencies to clear Dependabot warnings. None of the warnings were applicable to how `cargo-fund` used those dependencies.

# 0.2.1

## Fixed

- ([#8](https://github.com/acfoltzer/cargo-fund/issues/8)) `cargo-fund` will now try to prepend an `https://` scheme to bare relative URLs, and failure to parse a URL will lead to that entry being ignored rather than crashing.

# 0.2.0

## Changed

- ([#5](https://github.com/acfoltzer/cargo-fund/pull/5)) The program now looks for the Github API token in the `CARGO_FUND_GITHUB_API_TOKEN` environment variable rather than `GITHUB_API_TOKEN` in order to better support privilege separation.
- ([#6](https://github.com/acfoltzer/cargo-fund/pull/6)) Increased timeout on HTTP client operations to 60 seconds.

# 0.1.0

Initial release.
