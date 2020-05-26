//! `cargo fund`: find out how to financially support the authors of your dependencies.
//!
//! For example, when run on this project, `cargo fund` produces:
//!
//! ```text
//! % CARGO_FUND_GITHUB_API_TOKEN=... cargo fund
//! $HOME/cargo-fund (found funding links for 16 out of 138 dependencies)
//! ├─┬─ https://www.buymeacoffee.com/dannyguo
//! │ ├─ https://www.paypal.me/DannyGuo
//! │ └─ https://ko-fi.com/dannyguo
//! │    └─ strsim 0.8.0
//! ├─── https://github.com/sponsors/XAMPPRocky
//! │    └─ remove_dir_all 0.5.2
//! ├─── https://github.com/sponsors/dtolnay
//! │    ├─ anyhow 1.0.28
//! │    ├─ dtoa 0.4.5
//! │    ├─ itoa 0.4.5
//! │    ├─ proc-macro-hack 0.5.15
//! │    ├─ proc-macro-nested 0.1.4
//! │    ├─ quote 1.0.3
//! │    ├─ ryu 1.0.4
//! │    └─ syn 1.0.18
//! └─── https://github.com/sponsors/seanmonstar
//!      ├─ httparse 1.3.4
//!      ├─ num_cpus 1.13.0
//!      ├─ reqwest 0.10.4
//!      ├─ try-lock 0.2.2
//!      ├─ unicase 2.6.0
//!      └─ want 0.3.0
//! ```
//!
//! # Github API token
//!
//! `cargo fund` retrieves funding links for any dependencies with a Github URL in its
//! `[package.repository]` field. To retrieve this information, you must provide a valid Github API
//! token in the `CARGO_FUND_GITHUB_API_TOKEN` environment variable or the `--github-api-token` command-line
//! argument. To generate this token, go to <https://github.com/settings/tokens> and create a token
//! with the `public_repo` and `user` scopes.
use crate::args::Opts;
use anyhow::{anyhow, bail, Error};
use cargo_metadata::{Metadata, Package, PackageId};
use lazy_static::lazy_static;
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::convert::{TryFrom, TryInto};
use structopt::StructOpt;

mod args;
mod github;
mod metadata;

lazy_static! {
    static ref GLOBALS: RwLock<Option<Globals>> = RwLock::new(None);
}

struct Globals {
    github_api_token: String,
    client: reqwest::Client,
}

fn initialize_globals(github_api_token: &str) -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ))
        .build()?;
    *GLOBALS.write() = Some(Globals {
        github_api_token: github_api_token.to_string(),
        client,
    });
    Ok(())
}

fn globals() -> MappedRwLockReadGuard<'static, Globals> {
    RwLockReadGuard::map(GLOBALS.read(), |o| {
        o.as_ref().expect("globals must be initialized first")
    })
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
enum LinkSource {
    Github(github::GithubLinkSource),
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Platform {
    CommunityBridge,
    Custom,
    Github,
    IssueHunt,
    Kofi,
    Liberapay,
    OpenCollective,
    Otechie,
    Patreon,
    Tidelift,
    Other(String),
}

impl From<&str> for Platform {
    fn from(platform: &str) -> Self {
        match platform.to_ascii_uppercase().as_str() {
            "COMMUNITY_BRIDGE" => Self::CommunityBridge,
            "CUSTOM" => Self::Custom,
            "GITHUB" => Self::Github,
            "ISSUEHUNT" => Self::IssueHunt,
            "KO_FI" => Self::Kofi,
            "LIBERAPAY" => Self::Liberapay,
            "OPEN_COLLECTIVE" => Self::OpenCollective,
            "OTECHIE" => Self::Otechie,
            "PATREON" => Self::Patreon,
            "TIDELIFT" => Self::Tidelift,
            _ => Self::Other(platform.to_string()),
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct Link {
    platform: Platform,
    uri: http::Uri,
}

impl Ord for Link {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.platform.cmp(&other.platform) {
            std::cmp::Ordering::Equal => self.uri.to_string().cmp(&other.uri.to_string()),
            other => other,
        }
    }
}

impl PartialOrd for Link {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl TryFrom<(&str, &str)> for Link {
    type Error = Error;

    fn try_from((platform, url): (&str, &str)) -> Result<Self, Self::Error> {
        let platform = platform.try_into()?;
        let mut uri: http::Uri = url.parse()?;
        if let Platform::Github = platform {
            // fix up the URI for github sponsors 🤷
            let mut parts = uri.into_parts();
            parts.path_and_query = Some(
                format!(
                    "/sponsors{}",
                    parts
                        .path_and_query
                        .ok_or_else(|| anyhow!("Github URL missing path"))?
                        .as_str()
                )
                .as_str()
                .try_into()?,
            );
            uri = http::Uri::from_parts(parts)?;
        }
        Ok(Link { platform, uri })
    }
}

/// Try to get sources for a single package.
fn try_get_sources<'a>(package: &Package) -> Result<Vec<LinkSource>, Error> {
    let uri: http::Uri = if let Some(repo) = package.repository.as_ref() {
        repo.parse()?
    } else {
        return Ok(vec![]);
    };
    match uri.authority().map(|a| a.as_str()) {
        Some("github.com") | Some("www.github.com") => github::try_get_sources(uri),
        _ => Ok(vec![]),
    }
}

/// Get the sources for all dependencies in the workspace.
fn collect_sources(metadata: &Metadata) -> Result<HashMap<LinkSource, HashSet<PackageId>>, Error> {
    let mut source_map = HashMap::new();
    for pkg in &metadata.packages {
        if metadata.workspace_members.contains(&pkg.id) {
            // skip packages within our own workspace
            continue;
        }
        for source in try_get_sources(pkg)? {
            source_map
                .entry(source)
                .or_insert_with(HashSet::new)
                .insert(pkg.id.clone());
        }
    }
    Ok(source_map)
}

/// Turn the sources into a mapping between packages and sets of funding links.
async fn resolve_links(
    source_map: &HashMap<LinkSource, HashSet<PackageId>>,
) -> Result<HashMap<PackageId, HashSet<Link>>, Error> {
    // only one source for now, but other resolvers can add to this mapping later
    let mut resolved = HashMap::new();
    github::resolve_github_links(source_map, &mut resolved).await?;
    Ok(resolved)
}

/// Invert the mapping between packages and sets of funding links.
///
/// This allows us to group the output by unique sets of funding links.
fn invert_mapping(
    resolved: HashMap<PackageId, HashSet<Link>>,
) -> BTreeMap<BTreeSet<Link>, BTreeSet<PackageId>> {
    let mut inverted = BTreeMap::new();
    for (pkg, links) in resolved {
        let links: BTreeSet<Link> = links.into_iter().collect();
        inverted
            .entry(links)
            .or_insert_with(BTreeSet::new)
            .insert(pkg);
    }
    inverted
}

/// Print the results in a pretty tree.
///
/// TODO: support non-Unicode, perhaps add colors?
fn print_results(
    metadata: &Metadata,
    inverted: &BTreeMap<BTreeSet<Link>, BTreeSet<PackageId>>,
    num_found: usize,
) {
    println!(
        "{} (found funding links for {} out of {} dependencies)",
        metadata.workspace_root.display(),
        num_found,
        metadata.packages.len() - metadata.workspace_members.len()
    );
    let last_mapping_ix = inverted.len() - 1;
    for (mapping_ix, (links, pkgs)) in inverted.into_iter().enumerate() {
        let last_link_ix = links.len() - 1;
        for (link_ix, link) in links.into_iter().enumerate() {
            // first two characters of each link line
            match (mapping_ix, link_ix) {
                (0, 0) if last_mapping_ix == 0 => {
                    // first line of first and only link section
                    print!("──");
                }
                (mapping_ix, 0) if mapping_ix < last_mapping_ix => {
                    // first line of a link section
                    print!("├─");
                }
                (mapping_ix, _) if mapping_ix < last_mapping_ix => {
                    // non-first line of non-final link section
                    print!("│ ");
                }
                (mapping_ix, 0) if mapping_ix == last_mapping_ix => {
                    // first line of last link section of many
                    print!("└─");
                }
                // non-first line of final link section
                _ => print!("  "),
            }
            // second two characters of each link line
            match link_ix {
                0 if last_link_ix > 0 => {
                    // first link line of many
                    print!("┬─");
                }
                0 if last_link_ix == 0 => {
                    // first and only link line
                    print!("──");
                }
                link_ix if link_ix < last_link_ix => {
                    // non-first, non-final link line
                    print!("├─");
                }
                link_ix if link_ix == last_link_ix => {
                    // final link line of many
                    print!("└─");
                }
                _ => print!("  "),
            }
            println!(" {:?}", link.uri);
        }
        let last_pkg_ix = pkgs.len() - 1;
        for (pkg_ix, pkg) in pkgs.into_iter().enumerate() {
            if mapping_ix < last_mapping_ix {
                print!("│    ");
            } else {
                print!("     ");
            }
            if pkg_ix == last_pkg_ix {
                print!("└─");
            } else {
                print!("├─");
            }
            let pkg = &metadata[&pkg];
            println!(" {} {}", pkg.name, pkg.version);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let env = envy::from_env::<args::Env>()?;
    let Opts::Fund(args) = Opts::from_args();
    let github_api_token = if let Some(token) = args
        .github_api_token
        .as_ref()
        .or(env.github_api_token.as_ref())
    {
        token
    } else {
        bail!(
            "Github API token must be provided through the CARGO_FUND_GITHUB_API_TOKEN environment \
             variable or the --github-api-token flag."
        );
    };
    initialize_globals(github_api_token)?;

    let metadata = metadata::get(&args)?;

    let source_map = collect_sources(&metadata)?;
    let resolved = resolve_links(&source_map).await?;
    let num_found = resolved.len();
    let inverted = invert_mapping(resolved);
    print_results(&metadata, &inverted, num_found);

    Ok(())
}
