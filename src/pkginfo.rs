use std::env;
#[allow(unused)]
use std::path::{Path, PathBuf};

const MIHOMO_CONFIG_NAME: &'static str = "config.yaml";
const MIHOMO_SOCKET_NAME: &'static str = "mihomo.sock";
const AKASHA_CONFIG_NAME: &'static str = "akasha.toml";
const DEBUG_CONFIG_DIR: &'static str = "config";

pub struct PkgInfo {
    name: &'static str,
    version: &'static str,
    authors: &'static str,
    config_path: Box<Path>,
    cache_path: Box<Path>,
    tmp_path: Box<Path>,
}

impl PkgInfo {
    pub fn new() -> Self {
        Self {
            name: env!("CARGO_PKG_NAME"),
            version: env!("CARGO_PKG_VERSION"),
            authors: env!("CARGO_PKG_AUTHORS"),
            config_path: dirs::config_dir().unwrap().into_boxed_path(),
            cache_path: dirs::cache_dir().unwrap().into_boxed_path(),
            // delete target after beta version
            #[cfg(target_os = "linux")]
            tmp_path: std::env::temp_dir().into_boxed_path(),
            #[cfg(target_os = "macos")]
            tmp_path: PathBuf::from("/tmp").into_boxed_path(),
        }
    }

    pub fn get_name(&self) -> &'static str {
        self.name
    }

    pub fn get_version(&self) -> &'static str {
        self.version
    }

    pub fn get_authors(&self) -> &'static str {
        self.authors
    }

    pub fn get_app_configdir(&self) -> Box<Path> {
        if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
            PathBuf::from(manifest_dir)
                .join(DEBUG_CONFIG_DIR)
                .into_boxed_path()
        } else {
            self.config_path.join(self.name).into_boxed_path()
        }
    }

    pub fn get_app_cachedir(&self) -> Box<Path> {
        self.cache_path.join(self.name).into_boxed_path()
    }

    pub fn get_mihomo_socket(&self) -> Box<Path> {
        self.tmp_path
            .join(self.name)
            .join(MIHOMO_SOCKET_NAME)
            .into_boxed_path()
    }

    pub fn get_mihomo_config(&self) -> Box<Path> {
        self.get_app_configdir()
            .join(MIHOMO_CONFIG_NAME)
            .into_boxed_path()
    }

    pub fn get_akasha_config(&self) -> Box<Path> {
        self.get_app_configdir()
            .join(AKASHA_CONFIG_NAME)
            .into_boxed_path()
    }
}
