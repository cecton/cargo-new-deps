use anyhow::{ensure, Context, Result};
use cargo_metadata::{Metadata, MetadataCommand, Package, PackageId};
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use structopt::StructOpt;

/// List the newly added dependencies and their features.
#[derive(StructOpt)]
pub struct Cli {
    /// Read cargo metadata from JSON file to compare from.
    #[structopt(long)]
    from_json: Option<PathBuf>,

    /// Read cargo metadata from JSON file to compare to.
    #[structopt(long)]
    to_json: Option<PathBuf>,

    /// Commit or branch or branch to compare from.
    #[structopt(long)]
    from: Option<String>,

    /// Commit or branch or branch to compare to.
    #[structopt(long)]
    to: Option<String>,
}

impl Cli {
    pub fn run(self) -> Result<()> {
        let from_metadata = if let Some(path) = self.from_json {
            Self::read_metadata_from_json(path)?
        } else if let Some(commit) = self.from.as_deref() {
            Self::read_metadata_from_commit(commit)?
        } else {
            let commit = Self::git_default_branch()?;
            Self::read_metadata_from_commit(commit)?
        };

        let to_metadata = if let Some(path) = self.to_json {
            Self::read_metadata_from_json(path)?
        } else if let Some(commit) = self.to.as_deref() {
            Self::read_metadata_from_commit(commit)?
        } else {
            MetadataCommand::new()
                .exec()
                .context("could not parse metadata")?
        };

        let diff = MetadataDiff::new(&from_metadata, &to_metadata);
        let new_packages = diff.collect_new_dependencies();

        use ansi_term::Color::*;

        for ((dep_id, features), parents) in new_packages {
            print!("{}", Green.bold().paint(&diff.new_map[dep_id].name));
            if !features.is_empty() {
                for feature in features.iter() {
                    print!(" +{}", Red.bold().paint(&**feature));
                }
            }
            let mut it = parents.iter();
            print!(
                " pulled by: {}",
                Yellow.bold().paint(&diff.new_map[*it.next().unwrap()].name)
            );
            for parent_id in it {
                print!(", {}", Yellow.bold().paint(&diff.new_map[*parent_id].name));
            }
            println!();
        }

        Ok(())
    }

    fn read_metadata_from_json(path: impl AsRef<Path>) -> Result<Metadata> {
        let path = path.as_ref();
        Ok(MetadataCommand::parse(
            std::fs::read_to_string(path)
                .with_context(|| format!("could not read file: {}", path.display()))?,
        )
        .with_context(|| format!("could not parse metadata from file: {}", path.display()))?)
    }

    fn read_metadata_from_commit(commit: impl AsRef<str>) -> Result<Metadata> {
        let tmp_dir = tempfile::tempdir().context("could not create temporary directory")?;

        ensure!(
            Command::new("git")
                .args(&["worktree", "add"])
                .arg(tmp_dir.path())
                .arg(commit.as_ref())
                .stdout(Stdio::null())
                .status()
                .context("could not start command git")?
                .success(),
            "git working tree creation failed"
        );

        let metadata = MetadataCommand::new()
            .current_dir(tmp_dir.path())
            .exec()
            .context("could not parse metadata")?;

        let _ = Command::new("git")
            .args(&["worktree", "remove", "-f"])
            .arg(tmp_dir.path())
            .status();

        Ok(metadata)
    }

    fn git_default_branch() -> Result<String> {
        let output = Command::new("git")
            .args(&["symbolic-ref", "refs/remotes/origin/HEAD"])
            .stderr(Stdio::inherit())
            .output()
            .context("could not start command git")?;
        ensure!(output.status.success(), "could not get default branch");

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

fn main() -> Result<()> {
    let mut args = std::env::args().peekable();
    let command = args.next();
    args.next_if(|x| x.as_str() == "new-deps");
    Cli::from_iter(command.into_iter().chain(args)).run()
}

struct MetadataDiff<'a> {
    old_metadata: &'a Metadata,
    new_metadata: &'a Metadata,
    old_map: HashMap<&'a PackageId, &'a Package>,
    new_map: HashMap<&'a PackageId, &'a Package>,
}

impl<'a> MetadataDiff<'a> {
    pub fn new(old_metadata: &'a Metadata, new_metadata: &'a Metadata) -> Self {
        let old_map = Self::collect_packages_map(old_metadata);
        let new_map = Self::collect_packages_map(new_metadata);

        Self {
            old_metadata,
            new_metadata,
            old_map,
            new_map,
        }
    }

    pub fn collect_new_dependencies(
        &'a self,
    ) -> IndexMap<(&'a PackageId, Vec<&'a str>), Vec<&'a PackageId>> {
        let old_deps = Self::collect_dependencies(&self.old_metadata, &self.old_map);
        let new_deps = Self::collect_dependencies(&self.new_metadata, &self.new_map);
        let diff = new_deps
            .into_iter()
            .filter(|(_, new_features, new_dep_id)| {
                !old_deps.iter().any(|(_, old_features, old_dep_id)| {
                    self.old_map[old_dep_id].name == self.new_map[new_dep_id].name
                        && old_features.is_superset(new_features)
                })
            })
            .collect::<Vec<_>>();
        let mut new_packages =
            diff.into_iter()
                .fold(IndexMap::new(), |mut acc, (parent_id, features, dep_id)| {
                    let mut features = features.into_iter().collect::<Vec<_>>();
                    features.sort_unstable();
                    acc.entry((dep_id, features))
                        .or_insert(Vec::new())
                        .push(parent_id);
                    acc
                });
        new_packages.sort_keys();

        new_packages
    }

    fn collect_packages_map(metadata: &'a Metadata) -> HashMap<&'a PackageId, &'a Package> {
        metadata.packages.iter().map(|x| (&x.id, x)).collect()
    }

    fn collect_dependencies(
        metadata: &'a Metadata,
        map: &'a HashMap<&'a PackageId, &'a Package>,
    ) -> Vec<(&'a PackageId, HashSet<&'a str>, &'a PackageId)> {
        let first_level_dependencies = metadata
            .resolve
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .filter(|node| metadata.workspace_members.contains(&node.id))
            .flat_map(|node| &node.dependencies)
            .collect::<HashSet<_>>();

        metadata
            .resolve
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .flat_map(|node| {
                let package = map[&node.id];
                package
                    .dependencies
                    .iter()
                    .map(move |x| (package, &node.features, x))
            })
            .filter_map(|(parent_package, parent_features, dep)| {
                metadata
                    .packages
                    .iter()
                    .filter(|x| {
                        x.name == dep.name
                            && x.source
                                .as_ref()
                                .map(|s| strip_fragment(url::Url::parse(&s.repr).unwrap()))
                                == dep
                                    .source
                                    .as_ref()
                                    .map(|s| strip_fragment(url::Url::parse(&s).unwrap()))
                            && dep.req.matches(&x.version)
                    })
                    .reduce(|a, b| if a.version > b.version { a } else { b })
                    .map(|dep_package| (parent_package, parent_features, dep_package))
            })
            .map(|(parent_package, parent_features, dep_package)| {
                let dep_features = parent_package
                    .features
                    .iter()
                    .filter(|(k, _)| parent_features.contains(k))
                    .flat_map(|(_, dep_features)| dep_features)
                    .filter_map(|x| x.split_once('/'))
                    .filter(|(crate_name, _)| crate_name == &&dep_package.name)
                    .map(|(_, feature)| feature)
                    .collect::<HashSet<_>>();

                (&parent_package.id, dep_features, &dep_package.id)
            })
            .filter(|(parent_id, _, _)| first_level_dependencies.contains(parent_id))
            .collect()
    }
}

fn strip_fragment(mut url: url::Url) -> url::Url {
    url.set_fragment(None);
    url
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn scenario_1() {
        const OUTPUT_PATH: &str = "tests/scenario_1.stdout";

        let main = MetadataCommand::parse(include_str!("../tests/fixtures/main.json")).unwrap();
        let router_3 =
            MetadataCommand::parse(include_str!("../tests/fixtures/router-3.json")).unwrap();
        let diff = MetadataDiff::new(&main, &router_3);
        let got = format!("{:#?}", diff.collect_new_dependencies());

        if std::env::var("OVERWRITE").is_ok() || !Path::new(OUTPUT_PATH).exists() {
            fs::write(&OUTPUT_PATH, got).unwrap();
        } else {
            let expected = fs::read_to_string(&OUTPUT_PATH).unwrap();
            println!("{}", prettydiff::diff_lines(&expected, &got));
            assert!(got == expected);
        }
    }
}
