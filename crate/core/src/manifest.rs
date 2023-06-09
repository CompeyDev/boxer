use serde::Deserialize;
use std::{
    fs::File,
    io::{ErrorKind, Read},
    path::PathBuf,
    process::exit,
};
use toml::{de::Error, Table};

#[derive(Deserialize, Clone)]
pub struct ManifestSchema {
    pub package: PackageSection,
    pub lib: LibSection,
    pub scripts: Table,
    pub dependencies: Table,
}

#[derive(Deserialize, Clone)]
pub struct PackageSection {
    pub name: String,
    pub version: String,
    pub author: Option<Vec<String>>,
    pub git: String,
}

#[derive(Deserialize, Clone)]
pub struct LibSection {
    pub release_type: String,
}

pub struct ManfiestHandler {
    manifest_contents: String,
}

impl ManfiestHandler {
    pub fn new(proj_dir: PathBuf) -> Self {
        let mut manifest_contents = String::new();

        match File::open(proj_dir.join("Boxer.toml")) {
            Ok(mut contents) => contents
                .read_to_string(&mut manifest_contents)
                .unwrap_or_else(|err| {
                    tracing::error!("failed to read manifest contents to memory");
                    tracing::trace!("memory write error that occured {}", err);

                    exit(1);
                }),
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    tracing::error!(
                        "unable to find `Boxer.toml` manifest file in directory {:?}",
                        proj_dir
                    );

                    exit(1);
                } else {
                    tracing::error!(
                        "unable to read `Boxer.toml` manifest file in directory {:?} due to error {}",
                        proj_dir,
                        err.kind()
                    );

                    exit(1);
                }
            }
        };

        Self { manifest_contents }
    }

    pub fn parse_manifest(&self) -> Result<ManifestSchema, Error> {
        Ok(toml::from_str::<ManifestSchema>(
            self.manifest_contents.as_str(),
        )?)
    }
}
