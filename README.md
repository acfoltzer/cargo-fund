# cargo-fund

Discover funding links for your project's dependencies.

[![Crates.io][crates-badge]][crates-url]
[![CircleCI][circleci-badge]][circleci-url]

[crates-badge]: https://img.shields.io/crates/v/cargo-fund.svg
[crates-url]: https://crates.io/crates/cargo-fund
[circleci-badge]: https://img.shields.io/circleci/build/github/acfoltzer/cargo-fund/develop
[circleci-url]: https://circleci.com/gh/acfoltzer/cargo-fund

## Installation

To install `cargo-fund`, use `cargo`:

```sh
$ cargo install cargo-fund
```

### Github API token

`cargo-fund` retrieves funding links for any dependencies with a Github URL in its
`[package.repository]` field. To retrieve this information, you must provide a valid Github API
token in the `GITHUB_API_TOKEN` environment variable or the `--github-api-token` command-line
argument. To generate this token, go to <https://github.com/settings/tokens> and create a token
with the `public_repo` and `user` scopes.

## Usage

Run `cargo fund` in your workspace to print funding links. For example:

```text
$ GITHUB_API_TOKEN=... cargo fund
/path/to/cargo-fund (found funding links for 16 out of 138 dependencies)
├─┬─ https://www.buymeacoffee.com/dannyguo
│ ├─ https://www.paypal.me/DannyGuo
│ └─ https://ko-fi.com/dannyguo
│    └─ strsim 0.8.0
├─── https://github.com/sponsors/XAMPPRocky
│    └─ remove_dir_all 0.5.2
├─── https://github.com/sponsors/dtolnay
│    ├─ anyhow 1.0.28
│    ├─ dtoa 0.4.5
│    ├─ itoa 0.4.5
│    ├─ proc-macro-hack 0.5.15
│    ├─ proc-macro-nested 0.1.4
│    ├─ quote 1.0.3
│    ├─ ryu 1.0.4
│    └─ syn 1.0.18
└─── https://github.com/sponsors/seanmonstar
     ├─ httparse 1.3.4
     ├─ num_cpus 1.13.0
     ├─ reqwest 0.10.4
     ├─ try-lock 0.2.2
     ├─ unicase 2.6.0
     └─ want 0.3.0
```

## Including your sponsorship info

`cargo-fund` uses the Github API to get the available funding links for crates. To ensure your
crate's information appears:

1. Make sure that the `[package.repository]` in your `Cargo.toml` contains a valid Github URL.
2. Add your funding information to [`.github/FUNDING.yml`][funding-yml] in your repository.

Currently, Github is the only source of funding information, but please open an issue if you know of
any other structured sources of funding information.

[funding-yml]: https://help.github.com/en/github/administering-a-repository/displaying-a-sponsor-button-in-your-repository
