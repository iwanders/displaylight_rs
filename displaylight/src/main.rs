use displaylight::{Config, DisplayLight};
use std::error::Error;
use std::path::{Path, PathBuf};

extern crate clap;
use clap::{App, Arg, SubCommand};

fn main() -> Result<(), Box<dyn Error>> {
    let app = App::new("DisplayLight")
        .about("Controls leds behind the monitor.")
        .arg(
            Arg::with_name("config")
                .short("c")
                .help("Override the config, instead of autoselecting based on the OS.."),
        )
        .subcommand(
            SubCommand::with_name("list_ports").about("List serial ports / com ports and quit."),
        );

    let matches = app.clone().get_matches();

    if let Some(_matches) = matches.subcommand_matches("list_ports") {
        println!("Ports:\n{:#?}", lights::available_ports()?);
        return Ok(());
    }

    // Determine what config to load.
    let mut config_path: Option<PathBuf> = None;
    if let Some(v_in) = matches.value_of("config") {
        let path = Path::new(v_in);
        if path.exists() {
            config_path = Some(path.to_path_buf())
        }
    } else {
        // No config specified, try to find it in the current directories' config folder.
        // This is mostly just for development conveniency.
        let path = PathBuf::from("config").join(format!("{}.yaml", std::env::consts::OS));
        if path.exists() {
            config_path = Some(path);
        }
    }

    let mut config: Config = Default::default();
    if config_path.is_some() {
        let path = config_path
            .as_ref()
            .unwrap()
            .to_str()
            .expect("Path is always valid.");
        println!("Config: {}", path);

        use std::fs::File;
        use std::io::Read;
        match File::open(path) {
            Ok(mut file) => {
                let mut content = String::new();
                file.read_to_string(&mut content)
                    .expect("Should be able to read the file.");
                match serde_yaml::from_str(&content) {
                    Ok(parsed_config) => config = parsed_config,
                    Err(failure_message) => {
                        println!("Something went wrong parsing the configuration file:");
                        return Err(Box::new(failure_message));
                    }
                }
            }
            Err(error) => {
                return Err(Box::new(error));
            }
        }
    } else {
        println!("No config specified or found.");
    }

    let mut d = DisplayLight::new(config)?;
    d.run()
}
