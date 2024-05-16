mod atoms;
mod clients;
mod decorator;
mod event;
mod handlers;
mod keyboard;
mod layout_manager;
mod lucky;
mod screen_manager;

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
