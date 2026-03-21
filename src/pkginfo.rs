pub struct PkgInfo {
    name: &'static str,
    version: &'static str,
    config_path: &'static str,
    cache_path: &'static str,
}

impl PkgInfo {
    pub fn new() -> Self {
        Self {
            name: env!("CARGO_PKG_NAME"),
            version: env!("CARGO_PKG_VERSION"),
            config_path: env!("HOME"),
            cache_path: env!("HOME"),
        }
    }
}
