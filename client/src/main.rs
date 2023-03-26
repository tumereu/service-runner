use model::config::ServiceConfig;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

fn main() {
    let config = read_config(Path::new("../sampleConfig.yml"));
    println!("Hello, config: {config:?}");
}

fn read_config<P: AsRef<Path>>(path: P) -> Result<ServiceConfig, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    Ok(serde_yaml::from_reader(reader)?)
}