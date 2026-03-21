use std::path::Path;

pub struct PkgInfo {
    name: &'static str,
    version: &'static str,
    config_path: Box<Path>,
    cache_path: Box<Path>,
}

impl PkgInfo {
    pub fn new() -> Self {
        let name = env!("CARGO_PKG_NAME");
        let home_path = Path::new(env!("HOME"));
        let config_path = home_path.join(".config").join(name).into_boxed_path();
        let cache_path = home_path.join(".cache").join(name).into_boxed_path();
        Self {
            name,
            version: env!("CARGO_PKG_VERSION"),
            config_path,
            cache_path,
        }
    }

    pub fn get_name(&self) -> &'static str {
        self.name
    }

    pub fn get_version(&self) -> &'static str {
        self.version
    }
}
