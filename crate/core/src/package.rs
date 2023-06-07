use crate::manifest::ManfiestHandler;
use boxer_utils::meta::get_meta;
use git2::Repository;
use reqwest::blocking::Client as reqwest;
use serde::Deserialize;
use std::{
    collections::VecDeque,
    env::current_dir,
    fs::File,
    io::Write,
    path::Path,
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
                .expect("failed to build HTTP networking client"),
            config: ClientConfig {
                deps_dir: Path::new("./packages"),
            },
        }
    }

    pub fn download_package(&self, package_namespace: &str, package_version: &str) {
        let namespace_api_uri = format!("http://{}/api/{}", self.registry, package_namespace);

        let package_meta = match self.net.get(namespace_api_uri).send() {
            Ok(resp) => resp
                .json::<PackageMetadataResp>()
                .expect("failed to deserialize package metadata"),
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
            let pkg_id = target_pkg_meta.identifier;

            let git_api_uri = format!("http://{}/api/git/{}", self.registry, pkg_id);
            let git_uri = self
                .net
                .get(git_api_uri)
                .send()
                .expect(format!("unable to fetch git repo URL for package {}", pkg_id).as_str())
                .text()
                .unwrap();

            println!("{}", git_uri);

            let repo_path = get_meta().repos_path.join(pkg_id);

            Repository::clone(git_uri.as_str().trim(), &repo_path)
                .expect("failed to clone package git repo");

            let post_install_cmd: String = match ManfiestHandler::new(repo_path).parse_manifest() {
                Ok(parsed) => {
                    let build_scripts = parsed.scripts;
                    let target_script = build_scripts
                        .get("post_install")
                        .map(|s| s.to_owned())
                        .unwrap();

                    target_script.to_string()
                }

                Err(err) => {
                    tracing::error!(
                        "failed to parse config manifest `Boxer.toml`: {}",
                        err.message()
                    );

                    exit(1);
                }
            };

            let mut cmd_argv = post_install_cmd
                .split_whitespace()
                .collect::<VecDeque<&str>>();

            let mut build_cmd = Command::new(
                cmd_argv.pop_front().expect(
                    format!(
                        "failed to get build command executable for package {}",
                        package_namespace
                    )
                    .as_str(),
                ),
            );

            for arg in cmd_argv.into_iter() {
                build_cmd.arg(arg);
            }

            let mut build_cmd_child = build_cmd.spawn().expect(
                format!(
                    "failed to initially spawn build command for package {}",
                    package_namespace
                )
                .as_str(),
            );

            if !build_cmd_child
                .wait()
                .expect(
                    format!(
                        "failed to run spawned build command for package {}",
                        package_namespace
                    )
                    .as_str(),
                )
                .success()
            {
                tracing::error!("build command for package {} failed.", package_namespace);
                tracing::error!("-- stderr:\n{:?}", build_cmd_child.stderr.take().unwrap())
            }
        } else {
            let pkg_bundle_uri = format!("http://{}/dl/{}", self.registry, target_pkg_meta.bundle);

            let mut dl_resp = self.net.get(pkg_bundle_uri).send().expect(
                format!(
                    "failed to pull package bundle for package {}",
                    package_namespace
                )
                .as_str(),
            );

            let mut bundle_chunks = Vec::new();
            dl_resp
                .copy_to(&mut bundle_chunks)
                .expect("failed to write chunks into memory");

            let mut output_file = File::create(
                current_dir()
                    .expect("failed to get current directory; does it exist?")
                    .join(self.config.deps_dir)
                    .join(target_pkg_meta.bundle),
            )
            .expect(
                format!(
                    "failed to create dependency bundle file for package {}",
                    package_namespace
                )
                .as_str(),
            );

            output_file.write(&bundle_chunks as &[u8]).expect(
                format!(
                    "failed to write to dependency bundle file for package {}",
                    package_namespace
                )
                .as_str(),
            );
        }
    }
}
