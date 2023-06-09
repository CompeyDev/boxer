use crate::manifest::ManfiestHandler;
use boxer_utils::meta::get_meta;
use colored::Colorize;
use git2::{ErrorCode, MergeOptions, ObjectType, Repository};
use reqwest::blocking::Client as reqwest;
use serde::Deserialize;
use std::{
    collections::VecDeque,
    env::current_dir,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::{exit, Command},
};

pub struct PackageClient {
    registry: &'static str,
    net: reqwest,
    config: ClientConfig,
}

#[derive(Deserialize)]
pub struct PackageMetadataResp {
    pub owner: String,
    pub name: String,
    pub latest_version: String,
    available_versions: Vec<PackageVersionMetadataResp>,
}

#[allow(dead_code)]
#[derive(Deserialize, Clone)]
struct PackageVersionMetadataResp {
    version: String,
    needs_build: bool,
    bundle: String,
    identifier: String,
    published_at: String,
}

pub struct ClientConfig {
    deps_dir: &'static Path,
}

impl PackageClient {
    pub fn new() -> Self {
        Self {
            registry: "localhost:8080",
            net: reqwest::builder()
                .timeout(std::time::Duration::new(10, 0))
                .user_agent(format!(
                    "boxer-rbx / {} boxer-rbx package manager http client",
                    get_meta().version_hash
                ))
                .build()
                .unwrap_or_else(|_| {
                    tracing::error!("failed to build HTTP networking client");

                    exit(1);
                }),
            config: ClientConfig {
                deps_dir: Path::new("./packages"),
            },
        }
    }

    fn build_from_source(repo_path: PathBuf, package_namespace: &str) -> ! {
        let post_install_cmd: String = match ManfiestHandler::new(repo_path).parse_manifest() {
            Ok(parsed) => {
                let build_scripts = parsed.scripts;
                let target_script = build_scripts
                    .get("post_install")
                    .map(|s| s.to_owned())
                    .unwrap();

                let target_script_string = target_script.to_string();
                let mut target_script_trimmed = target_script_string.chars();

                target_script_trimmed.next();
                target_script_trimmed.next_back();
                target_script_trimmed.as_str().to_string()
            }

            Err(err) => {
                tracing::error!(
                    "failed to parse config manifest `Boxer.toml`: {}",
                    err.message()
                );

                exit(1);
            }
        };

        tracing::debug!(
            "post_install command to be executed: `{}`",
            post_install_cmd
        );

        let shell: &str = if cfg!(target_os = "windows") {
            "cmd"
        } else {
            "sh"
        };

        let mut cmd_argv = post_install_cmd
            .split_whitespace()
            .collect::<VecDeque<&str>>();

        cmd_argv.push_front(shell);

        #[cfg(not(target_os = "windows"))]
        cmd_argv.insert(1, "-c");

        tracing::debug!("args to be passed to command instance: {:#?}", cmd_argv);

        let mut build_cmd = Command::new(cmd_argv.pop_front().unwrap_or_else(|| {
            tracing::error!(
                "failed to get build command executable for package {}",
                package_namespace
            );

            exit(1);
        }));

        for arg in cmd_argv.into_iter() {
            build_cmd.arg(arg);
        }

        tracing::trace!("current command instance: {:#?}", build_cmd);

        let mut build_cmd_child = build_cmd.spawn().unwrap_or_else(|err| {
            tracing::error!(
                "failed to initially spawn build command for package {}",
                package_namespace
            );
            tracing::error!("build command error: {}", err);

            exit(1);
        });

        if !build_cmd_child
            .wait()
            .unwrap_or_else(|_| {
                tracing::error!(
                    "failed to run spawned build command for package {}",
                    package_namespace
                );

                exit(1);
            })
            .success()
        {
            tracing::error!("build command for package {} failed.", package_namespace);
            tracing::error!("-- stderr:\n{:?}", build_cmd_child.stderr);

            exit(1);
        }

        exit(0);
    }

    pub fn download_package(&self, package_namespace: &str, package_version: &str) {
        tracing::info!(
            "{} {}@{}...",
            "Downloading".green().bold(),
            package_namespace,
            package_version
        );

        let namespace_api_uri = format!("http://{}/api/meta/{}", self.registry, package_namespace);

        let package_meta = match self.net.get(namespace_api_uri).send() {
            Ok(resp) => resp.json::<PackageMetadataResp>().unwrap_or_else(|_| {
                tracing::error!("failed to deserialize package metadata");

                exit(1);
            }),
            Err(err) => {
                tracing::error!(
                    "failed to fetch package metadata for {} with error {}",
                    package_namespace,
                    err
                );
                exit(1);
            }
        };

        let mut target_pkg_meta: PackageVersionMetadataResp =
            (&package_meta.available_versions[0]).to_owned();

        for pkg_meta in package_meta.available_versions {
            if pkg_meta.version == package_version.to_string() {
                target_pkg_meta = pkg_meta;
            }
        }

        if target_pkg_meta.needs_build {
            tracing::info!(
                "    {} {}@{}...",
                "Building".purple(),
                package_namespace,
                package_version
            );

            let pkg_id = target_pkg_meta.identifier;

            let git_api_uri = format!("http://{}/api/git/{}", self.registry, pkg_id);
            let git_uri = self
                .net
                .get(git_api_uri)
                .send()
                .unwrap_or_else(|_| {
                    tracing::error!("unable to fetch git repo URL for package {}", pkg_id);

                    exit(1);
                })
                .text()
                .unwrap();

            tracing::debug!(
                "fetched git URI {} for package {}",
                git_uri,
                package_namespace
            );

            let repo_path = get_meta().repos_path.join(pkg_id);

            Repository::clone(git_uri.as_str().trim(), &repo_path).unwrap_or_else(|err| {
                tracing::error!("failed to clone package git repo");
                tracing::trace!("cloning error that occurred: {}", err);

                // in case the directory already exists, that means we have 
                // previously cached it. 
                // in that case, we just attempt to fetch from origin and 
                // merge into HEAD.
                if err.code() == ErrorCode::Exists {
                    let repo = Repository::open(&repo_path).unwrap();

                    repo.find_remote("origin")
                        .unwrap()
                        // in practice, we should first discover what the available branches are,
                        // rather than assuming main exists.
                        .fetch(&["main"], None, None)
                        .unwrap();

                    let head_commit = repo
                        .head()
                        .unwrap_or_else(|_| {
                            tracing::warn!(
                                "failed to update cached package source, using current HEAD for {}",
                                package_namespace
                            );

                            Self::build_from_source(repo_path.clone(), package_namespace);
                        })
                        .resolve()
                        .unwrap_or_else(|_| {
                            tracing::warn!(
                                "failed to update cached package source, using current HEAD for {}",
                                package_namespace
                            );

                            Self::build_from_source(repo_path.clone(), package_namespace);
                        })
                        .peel(ObjectType::Commit)
                        .unwrap_or_else(|_| {
                            tracing::warn!(
                                "failed to update cached package source, using current HEAD for {}",
                                package_namespace
                            );

                            Self::build_from_source(repo_path.clone(), package_namespace);
                        })
                        .into_commit()
                        .unwrap_or_else(|_| {
                            tracing::warn!(
                                "failed to update cached package source, using current HEAD for {}",
                                package_namespace
                            );

                            Self::build_from_source(repo_path.clone(), package_namespace);
                        });

                    let fetch_head = repo
                        .find_reference("FETCH_HEAD")
                        .unwrap_or_else(|_| {
                            tracing::warn!(
                                "failed to update cached package source, using current HEAD for {}",
                                package_namespace
                            );

                            Self::build_from_source(repo_path.clone(), package_namespace);
                        })
                        .peel_to_commit()
                        .unwrap_or_else(|_| {
                            tracing::warn!(
                                "failed to update cached package source, using current HEAD for {}",
                                package_namespace
                            );

                            Self::build_from_source(repo_path.clone(), package_namespace);
                        });

                    repo.merge_commits(&head_commit, &fetch_head, Some(&MergeOptions::new()))
                        .unwrap_or_else(|_| {
                            tracing::warn!(
                                "failed to update cached package source, using current HEAD for {}",
                                package_namespace
                            );

                            Self::build_from_source(repo_path.clone(), package_namespace);
                        });
                }

                exit(1);
            });

            tracing::debug!(
                "cloned git repo {} for package {}",
                git_uri,
                package_namespace
            );

            tracing::debug!(
                "proceeding to parse `Boxer.toml` manifest for post_install script for package {}",
                package_namespace
            );
        } else {
            tracing::info!(
                "    {} {}@{}...",
                "Unpacking".purple(),
                package_namespace,
                package_version
            );

            let pkg_bundle_uri = format!("http://{}/dl/{}", self.registry, target_pkg_meta.bundle);

            let mut dl_resp = self.net.get(pkg_bundle_uri).send().unwrap_or_else(|_| {
                tracing::error!(
                    "failed to pull package bundle for package {}",
                    package_namespace
                );

                exit(1);
            });

            let mut bundle_chunks = Vec::new();
            dl_resp.copy_to(&mut bundle_chunks).unwrap_or_else(|_| {
                tracing::error!(
                    "failed to write response bundle chunks into memory for package {}",
                    package_namespace
                );

                exit(1);
            });

            let mut output_file = File::create(
                current_dir()
                    .unwrap_or_else(|_| {
                        tracing::error!("failed to get current directory; does it exist?");

                        exit(1);
                    })
                    .join(self.config.deps_dir)
                    .join(target_pkg_meta.bundle),
            )
            .unwrap_or_else(|_| {
                tracing::error!(
                    "failed to create dependency bundle file for package {}",
                    package_namespace
                );

                exit(1);
            });

            output_file
                .write(&bundle_chunks as &[u8])
                .unwrap_or_else(|_| {
                    tracing::error!(
                        "failed to write to dependency bundle file for package {}",
                        package_namespace
                    );

                    exit(1);
                });
        }

        tracing::info!(
            "{} {}@{}",
            "Downloaded".blue().bold(),
            package_namespace,
            package_version
        );
    }
}
