use std::path::Path;

pub struct Metadata {
    pub version: f32,
    pub version_hash: &'static str,
    pub repos_path: &'static Path
}

pub fn get_meta() -> Metadata {
    Metadata {
        version: 0.0,
        version_hash: "3ae7fc",
        repos_path: Path::new("./boxer/repos")
    }
}