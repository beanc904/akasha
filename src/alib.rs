use clap::{Parser, Subcommand, ValueEnum};

use crate::app::App;

pub(super) async fn tui_main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal).await;
    ratatui::restore();
    result
}

pub(super) async fn args_parse() -> color_eyre::Result<()> {
    let args = Args::parse();

    match args.command {
        Some(Command::Init { shell, reverse }) => {
            if !reverse {
                match shell {
                    Some(Shell::Zsh) => {
                        println!("echo zsh praxy function...");
                    }
                    Some(Shell::Fish) => {
                        println!("echo fish praxy function...");
                    }
                    None => {
                        println!("write the function to your shell...");
                    }
                }
            } else {
                println!("reverse the function in your shell...");
            }
        }
        // Unreachable branch
        None => {
            println!("unreachable place?");
        }
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[command(version = concat!(
    " ", env!("CARGO_PKG_VERSION"),
    " (", env!("GIT_HASH"), ")",
    " ", env!("BUILD_TIME")
), about)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Initialize the function(praxy) of setup proxy env for your shell(zsh, fish).
    Init {
        #[arg(value_enum)]
        shell: Option<Shell>,

        #[arg(long)]
        reverse: bool,
    },
}

#[derive(ValueEnum, Debug, Clone)]
enum Shell {
    Zsh,
    Fish,
}
