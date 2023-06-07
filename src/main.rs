mod pull_package;
pub mod utils;

fn main() {
    pull_package::PackageClient::new().download_package("@devcomp/real", "0.0.1")
}
