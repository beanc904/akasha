use std::error::Error;

use akasha::parser::config::MihomoConfig;

fn main() -> Result<(), Box<dyn Error>> {
    let mc = MihomoConfig::from_file("resources/config.yaml")?;
    let vec = mc.groups_name();
    println!("ProxyGroups names: {:?}", vec);
    Ok(())
}
