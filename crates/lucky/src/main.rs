mod cursor;
mod keyboard;
mod keys;
mod lucky;

use lucky::Lucky;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("failed to set global subscriber");
    let config = config::load_config();

    Lucky::new(config).run();

    Ok(())
}
