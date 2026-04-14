pub mod app;
pub mod pkginfo;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    env_logger::init();
    let terminal = ratatui::init();
    let result = app::App::new().run(terminal).await;
    ratatui::restore();
    result
}
