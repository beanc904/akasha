use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand, ValueEnum};

use crate::{app::App, pkginfo::PkgInfo};

pub(super) async fn tui_main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal).await;
    ratatui::restore();
    result
}

pub(super) async fn cli_main() -> color_eyre::Result<()> {
    let args = Args::parse();
    let port = "7890";

    match args.command {
        Some(Command::Init { shell, reverse }) => {
            if !reverse {
                match shell {
                    Some(Shell::Bash) => {
                        print!(
                            r#"
function praxy() {{
    if [[ $1 == "on" ]]; then
        export http_proxy=http://127.0.0.1:{port}
        export https_proxy=http://127.0.0.1:{port}
        export all_proxy=socks5://127.0.0.1:{port}
        echo -e "Terminal proxy is now enabled"
    elif [[ $1 == "off" ]]; then
        unset http_proxy https_proxy all_proxy
        echo -e "Terminal proxy is now disabled"
    else
        echo -e "Usage: praxy [on|off]"
    fi
}}
"#
                        );
                    }
                    Some(Shell::Zsh) => {
                        print!(
                            r#"
function praxy() {{
    if [[ $1 == "on" ]]; then
        export http_proxy=http://127.0.0.1:{port}
        export https_proxy=http://127.0.0.1:{port}
        export all_proxy=socks5://127.0.0.1:{port}
        echo -e "Terminal proxy is now enabled"
    elif [[ $1 == "off" ]]; then
        unset http_proxy https_proxy all_proxy
        echo -e "Terminal proxy is now disabled"
    else
        echo -e "Usage: praxy [on|off]"
    fi
}}
"#
                        );
                    }
                    Some(Shell::Fish) => {
                        print!(
                            r#"
function praxy
    if test "$argv[1]" = "on"
        set -gx http_proxy http://127.0.0.1:{port}
        set -gx https_proxy http://127.0.0.1:{port}
        set -gx all_proxy socks5://127.0.0.1:{port}
        echo "Terminal proxy is now enabled"
    else if test "$argv[1]" = "off"
        set -e http_proxy https_proxy all_proxy
        echo "Terminal proxy is now disabled"
    else
        echo "Usage: praxy [on|off]"
    end
end
"#
                        );
                    }
                    None => {
                        // println!("write the function to your shell...");
                        let _ = setup_shell_function(true).unwrap();
                    }
                }
            } else {
                // println!("reverse the function in your shell...");
                let _ = setup_shell_function(false).unwrap();
            }
        }
        Some(Command::MihomoStart) => {
            let pkginfo = PkgInfo::new();
            println!(
                "akasha-mihomo -d {} -f {} -ext-ctl-unix {}",
                pkginfo.get_app_configdir().to_str().unwrap(),
                pkginfo.get_mihomo_config().to_str().unwrap(),
                pkginfo.get_mihomo_socket().to_str().unwrap()
            );
        }
        // Unreachable branch
        None => {
            panic!("This branch will never be reached.");
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
    MihomoStart,
}

#[derive(ValueEnum, Debug, Clone)]
enum Shell {
    Bash,
    Zsh,
    Fish,
}

fn setup_shell_function(is_install: bool) -> io::Result<()> {
    if is_install {
        match env::var("SHELL") {
            Ok(shell_path) => match shell_path.as_str() {
                "/bin/bash" => install_shell_init(Shell::Bash)?,
                "/bin/zsh" => install_shell_init(Shell::Zsh)?,
                "/bin/fish" => install_shell_init(Shell::Fish)?,
                _ => eprintln!("Unknown shell type"),
            },
            Err(_) => todo!(),
        }
    } else {
        match env::var("SHELL") {
            Ok(shell_path) => match shell_path.as_str() {
                "/bin/bash" => uninstall_shell_init(Shell::Bash)?,
                "/bin/zsh" => uninstall_shell_init(Shell::Zsh)?,
                "/bin/fish" => uninstall_shell_init(Shell::Fish)?,
                _ => eprintln!("Unknown shell type"),
            },
            Err(_) => todo!(),
        }
    }
    Ok(())
}

impl Shell {
    /// Shell name
    fn name(self) -> &'static str {
        match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
        }
    }

    /// Correspond rc file path
    fn rc_path(&self) -> io::Result<PathBuf> {
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "HOME not set"))?;

        let path = match self {
            Shell::Bash => home.join(".bashrc"),
            Shell::Zsh => home.join(".zshrc"),
            Shell::Fish => home.join(".config/fish/config.fish"),
        };

        Ok(path)
    }

    /// Init line
    fn init_line(self) -> String {
        format!(r#"eval "$(/usr/local/bin/akasha init {})""#, self.name())
    }
}

/// Write into init line (write into when not exist)
fn install_shell_init(shell: Shell) -> io::Result<()> {
    let path = shell.rc_path()?;
    let init_line = shell.init_line();

    ensure_parent_dir(&path)?;

    let mut content = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        String::new()
    };

    // Avoid duplicating addition
    if !content.lines().any(|line| line.trim() == init_line) {
        if !content.ends_with('\n') && !content.is_empty() {
            content.push('\n');
        }

        content.push_str(&init_line);
        content.push('\n');

        fs::write(path, content)?;
    }

    Ok(())
}

/// Delete init line
fn uninstall_shell_init(shell: Shell) -> io::Result<()> {
    let path = shell.rc_path()?;

    if !path.exists() {
        return Ok(());
    }

    let init_line = shell.init_line();

    let content = fs::read_to_string(&path)?;

    let filtered = content
        .lines()
        .filter(|line| line.trim() != init_line)
        .collect::<Vec<_>>()
        .join("\n");

    let final_content = if filtered.is_empty() {
        String::new()
    } else {
        format!("{filtered}\n")
    };

    fs::write(path, final_content)?;

    Ok(())
}

/// Ensure father dir exist (fish will use it)
fn ensure_parent_dir(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    Ok(())
}
