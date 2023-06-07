use boxer_core::package::PackageClient;

fn main() {
    PackageClient::new().download_package("@devcomp/luau-asm", "0.0.1")
}
