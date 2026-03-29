use std::error::Error;

use akasha::parser::config::MihomoConfig;

fn main() -> Result<(), Box<dyn Error>> {
    let mc = MihomoConfig::new("resources/config.yaml")?;
    let vec = mc.get_proxy_groups_namevec();
    println!("ProxyGroups names: {:?}", vec);
    Ok(())
}
