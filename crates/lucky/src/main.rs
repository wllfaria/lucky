mod atoms;
mod decorator;
mod event;
mod handlers;
mod keyboard;
mod layout_manager;
mod lucky;
mod screen;
mod screen_manager;

use lucky::Lucky;
use tracing_subscriber::FmtSubscriber;

fn main() -> anyhow::Result<()> {
    let (data_dir, log_file) = config::log_file()?;
    let log_writer = tracing_appender::rolling::daily(data_dir, log_file);
    let (non_blocking, _guard) = tracing_appender::non_blocking(log_writer);
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .with_ansi(false)
        .with_writer(non_blocking)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");

    Lucky::new().run();

    Ok(())
}
