use boxer_core::package::PackageClient;
use boxer_utils::fmt::LogFormatter;
use colored::Colorize;
use once_cell::unsync::Lazy;
use std::time::Instant;
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

    let start = Instant::now();

    {
        PackageClient::new().download_package("@devcomp/luau-asm", "0.0.1");
    }

    println!(
        "       {} in {:.2?}",
        "Finished".green().bold(),
        start.elapsed()
    );
}
