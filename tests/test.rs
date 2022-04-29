use std::path::Path;
use std::process::Command;

fn sanitize_stderr(bytes: &[u8]) -> String {
    let stderr = std::str::from_utf8(bytes).expect("stderr is valid UTF-8");
    // since we're running a bunch of cargo commands back to back, this message can sometimes show
    // up in the test output
    stderr.replace("    Blocking waiting for file lock on package cache\n", "")
}

#[test]
fn client_package_output_expected() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let client_package = root.join("tests").join("client-package");
    let expected = format!(
        "{} (found funding links for 1 out of 3 dependencies)
──┬─ https://acfoltzer.net/bare_relative_link
  ├─ https://www.acfoltzer.net/
  ├─ https://www.acfoltzer.net/another_url
  ├─ https://issuehunt.io/r/acfoltzer
  ├─ https://ko-fi.com/acfoltzer
  ├─ https://liberapay.com/acfoltzer
  └─ https://patreon.com/acfoltzer
     └─ funding-test 0.1.0\n",
        client_package.display()
    );
    let exe = Path::new(env!("CARGO_BIN_EXE_cargo-fund"));
    let output = Command::new(exe)
        .current_dir(client_package)
        .arg("fund")
        .env(
            "CARGO_FUND_GITHUB_API_TOKEN",
            std::env::var_os("VALID_CARGO_FUND_GITHUB_API_TOKEN").unwrap(),
        )
        .output()
        .expect("cargo-fund runs");

    assert!(output.status.success());
    let stdout = std::str::from_utf8(&output.stdout).expect("stdout is valid UTF-8");
    assert_eq!(stdout, expected, "stdout matches");
    assert_eq!(&sanitize_stderr(&output.stderr), "", "stderr matches");
}

#[test]
fn missing_token() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let expected =
        "Error: Github API token must be provided through the CARGO_FUND_GITHUB_API_TOKEN \
         environment variable or the --github-api-token flag.\n";
    let exe = Path::new(env!("CARGO_BIN_EXE_cargo-fund"));
    let output = Command::new(exe)
        .current_dir(root.join("tests").join("client-package"))
        .arg("fund")
        // not necessary for CI, but makes local testing easier
        .env_remove("CARGO_FUND_GITHUB_API_TOKEN")
        .output()
        .expect("cargo-fund runs");
    assert!(!output.status.success());
    assert_eq!(&output.stdout, b"", "stdout matches");
    assert_eq!(&sanitize_stderr(&output.stderr), expected, "stderr matches");
}

#[test]
fn invalid_token() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let expected =
        "Error: Invalid Github API token. Create a token with the `public_repo` and `user` scopes \
         at https://github.com/settings/tokens.\n";
    let exe = Path::new(env!("CARGO_BIN_EXE_cargo-fund"));
    let mut token = std::env::var("VALID_CARGO_FUND_GITHUB_API_TOKEN").unwrap();
    // remove a character to invalidate the token
    token.pop();

    let output = Command::new(exe)
        .current_dir(root.join("tests").join("client-package"))
        .arg("fund")
        .env("CARGO_FUND_GITHUB_API_TOKEN", token)
        .output()
        .expect("cargo-fund runs");
    assert!(!output.status.success());
    assert_eq!(&output.stdout, b"", "stdout matches");
    assert_eq!(&sanitize_stderr(&output.stderr), expected, "stderr matches");
}

#[test]
fn insufficient_scopes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let expected = "Error: Insufficient Github API token scopes. Modify your token to include the \
                    `public_repo` and `user` scopes at https://github.com/settings/tokens.\n";
    let exe = Path::new(env!("CARGO_BIN_EXE_cargo-fund"));
    let output = Command::new(exe)
        .current_dir(root.join("tests").join("client-package"))
        .arg("fund")
        .env(
            "CARGO_FUND_GITHUB_API_TOKEN",
            std::env::var_os("INSUFFICIENT_SCOPES_CARGO_FUND_GITHUB_API_TOKEN").unwrap(),
        )
        .output()
        .expect("cargo-fund runs");
    assert!(!output.status.success());
    assert_eq!(&output.stdout, b"", "stdout matches");
    assert_eq!(&sanitize_stderr(&output.stderr), expected, "stderr matches");
}
