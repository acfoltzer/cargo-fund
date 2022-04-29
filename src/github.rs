use super::{globals, Link, LinkSource, Platform};
use anyhow::{anyhow, bail, Error};
use cargo_metadata::PackageId;
use http::{StatusCode, Uri};
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::fmt::Write;
use tracing::{debug, info, trace, warn};

const GITHUB_TOKEN_HELP: &str = "Invalid Github API token. \
Create a token with the `public_repo` and `user` scopes at https://github.com/settings/tokens.";

const GITHUB_TOKEN_SCOPES_HELP: &str = "Insufficient Github API token scopes. \
Modify your token to include the `public_repo` and `user` scopes at https://github.com/settings/tokens.";

#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum GithubLinkSource {
    Repo { owner: String, name: String },
    Owner { owner: String },
}

impl GithubLinkSource {
    fn owner(&self) -> &str {
        match self {
            GithubLinkSource::Repo { owner, .. } => owner,
            GithubLinkSource::Owner { owner, .. } => owner,
        }
    }
}

pub(crate) fn try_get_sources(uri: Uri) -> Result<Vec<LinkSource>, Error> {
    let mut path_components = uri.path().split("/").skip(1).take(2);
    let owner = path_components.next();
    let name = path_components.next();
    if let (Some(owner), Some(name)) = (owner, name) {
        let name = name.trim_end_matches(".git");
        Ok(vec![
            LinkSource::Github(GithubLinkSource::Repo {
                owner: owner.to_string(),
                name: name.to_string(),
            }),
            LinkSource::Github(GithubLinkSource::Owner {
                owner: owner.to_string(),
            }),
        ])
    } else {
        bail!("not a full Github URI: {}", uri)
    }
}

pub(crate) async fn resolve_github_links(
    source_map: &HashMap<LinkSource, HashSet<PackageId>>,
    resolved: &mut HashMap<PackageId, HashSet<Link>>,
) -> Result<(), Error> {
    #[derive(Clone, Debug, Eq, PartialEq, Hash)]
    enum Alias {
        Repo(String),
        Owner(String),
    }
    let mut query_map = HashMap::new();
    let mut gensym = 0usize;
    let mut query = "query FundingLinks {".to_string();
    for (source, pkgs) in source_map {
        let alias = format!("_{}", gensym);
        gensym += 1;
        // allow this pattern even though we have no other `LinkSource` variants yet
        #[allow(irrefutable_let_patterns)]
        let source = if let LinkSource::Github(source) = source {
            source
        } else {
            continue;
        };
        match &source {
            GithubLinkSource::Repo { owner, name } => {
                writeln!(
                    &mut query,
                    "
{}: repository(owner: {:?}, name: {:?}) {{
  fundingLinks {{
    platform
    url
  }}
}}",
                    alias, owner, name,
                )
                .unwrap();
                query_map.insert(Alias::Repo(alias), (source, pkgs));
            }
            GithubLinkSource::Owner { owner } => {
                writeln!(
                    &mut query,
                    "
{}: repositoryOwner(login: {:?}) {{
  ... on Organization {{
    sponsorsListing {{
      id
    }}
  }}
  ... on User {{
    sponsorsListing {{
      id
    }}
  }}
}}
",
                    alias, owner
                )
                .unwrap();
                query_map.insert(Alias::Owner(alias), (source, pkgs));
            }
        }
    }
    writeln!(
        &mut query,
        "
  rateLimit {{
    cost
    remaining
  }}
}}"
    )
    .unwrap();

    let query = serde_json::json!({ "query": query });

    let req = globals()
        .client
        .post("https://api.github.com/graphql")
        .bearer_auth(&globals().github_api_token)
        .json(&query);

    trace!("sending Github GraphQL query");

    let resp = req.send().await?;

    trace!("received Github GraphQL query response");

    match resp.status() {
        StatusCode::OK => (),
        StatusCode::UNAUTHORIZED => bail!(GITHUB_TOKEN_HELP),
        status => bail!("Github API returned unexpected status: {}", status),
    }

    trace!("deserializing Github response JSON");

    let res: serde_json::Value = resp.json().await?;

    trace!("deserialized Github response JSON");

    if let serde_json::Value::Array(errors) = &res["errors"] {
        for error in errors {
            let message = error["message"]
                .as_str()
                .ok_or_else(|| anyhow!("Malformed Github API response"))?;
            if let serde_json::Value::String(ty) = &error["type"] {
                match ty.as_str() {
                    "INSUFFICIENT_SCOPES" => bail!(GITHUB_TOKEN_SCOPES_HELP),
                    "NOT_FOUND" => {
                        info!("{}", message);
                        continue;
                    }
                    _ => {
                        eprintln!("{}", error);
                        bail!("Github API response contained error: {}", message)
                    }
                }
            } else {
                bail!("Malformed Github API response");
            }
        }
    }

    for (alias, (source, pkgs)) in query_map {
        trace!("processing {:?}, {:?}", alias, source);
        match alias {
            Alias::Repo(alias) => {
                if let serde_json::Value::Array(links) = &res["data"][alias]["fundingLinks"] {
                    for link in links {
                        trace!("processing {:?}", link);
                        let platform = link["platform"]
                            .as_str()
                            .ok_or_else(|| anyhow!("Malformed Github API response"))?;
                        let uri = link["url"]
                            .as_str()
                            .ok_or_else(|| anyhow!("Malformed Github API response"))?;
                        let link = match Link::try_from((platform, uri)) {
                            Ok(link) => link,
                            Err(e) => {
                                warn!(
                                    platform = %platform,
                                    uri = %uri,
                                    "could not parse Github funding links; skipping: {}",
                                    e
                                );
                                continue;
                            }
                        };
                        for pkg in pkgs.iter() {
                            resolved
                                .entry(pkg.clone())
                                .or_insert_with(HashSet::new)
                                .insert(link.clone());
                        }
                    }
                } else {
                    // no result, probably indicates an invalid or private repo
                    continue;
                }
            }
            Alias::Owner(alias) => {
                if let serde_json::Value::Null = res["data"][alias]["sponsorsListing"] {
                    continue;
                } else {
                    let uri: http::Uri =
                        match format!("https://github.com/sponsors/{}", source.owner()).parse() {
                            Ok(link) => link,
                            Err(e) => {
                                warn!(
                                    owner = %source.owner(),
                                    "could not create valid owner sponsor link; skipping: {}",
                                    e
                                );
                                continue;
                            }
                        };
                    for pkg in pkgs {
                        resolved
                            .entry(pkg.clone())
                            .or_insert_with(HashSet::new)
                            .insert(Link {
                                platform: Platform::Github,
                                uri: uri.clone(),
                            });
                    }
                }
            }
        }
    }

    debug!("finished resolving Github links");

    Ok(())
}
