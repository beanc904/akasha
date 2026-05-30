mod alib;
mod app;
mod pkginfo;

use std::env;

use crate::alib::{cli_main, tui_main};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    if env::args().len() == 1 {
        // Running without any args.
        // default enter the tui mode
        tui_main().await
    } else {
        // Running with one arg at least.
        cli_main().await
    }
}
