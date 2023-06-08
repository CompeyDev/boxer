use boxer_core::package::PackageClient;
use boxer_utils::fmt::LogFormatter;
use once_cell::unsync::Lazy;
use tracing::Level;

fn main() {
    let tracing_level: Lazy<Level> = Lazy::new(|| {
        if cfg!(debug_assertions) {
            Level::TRACE
        } else {
            Level::INFO
        }
    });

    tracing_subscriber::fmt()
        .with_max_level((&*tracing_level).to_owned())
        .event_format(LogFormatter)
        .init();

    PackageClient::new()
        .download_package(
            "@devcomp/luau-asm", 
            "0.0.1"
        );
}
