#[macro_use]
extern crate serde_derive;
extern crate docopt;
extern crate dialoguer;
extern crate colored;
extern crate bender_config;



use docopt::Docopt;
use dialoguer::Confirmation;
use colored::*;
use bender_config::{Config, PathMethods};

const USAGE: &'static str = "
bender-config

A cli to the bender-configuration file

Usage:
  bender-config new
  bender-config new default
  bender-config new appsecret
  bender-config validate
  bender-config show
  bender-config path

  bender-config (-h | --help)
  bender-config --version

Commands:
  new  . . . . . . . . .  Run the configuration wizard

  new default  . . . . .  Write a default config to the default location

  new appsecret  . . . .  Generate a appsecret and put it to the private folder

  show . . . . . . . . .  Show the configuration file

  validate . . . . . . .  Check for validity

  path . . . . . . . . .  Return the path of the configuration file






Options:
  -h --help               Show this screen.
  --version               Show version.
  -s --simple             Just list the job IDs
  --status=<status>       Filter listed jobs by status
";



// -g            => flag_g
// --group       => flag_group
// --group <arg> => flag_group
// FILE          => arg_FILE
// <file>        => arg_file
// build         => cmd_build

#[derive(Debug, Deserialize)]
struct Args {
    cmd_new: bool,
    cmd_default: bool,
    cmd_appsecret: bool,
    cmd_show: bool,
    cmd_validate: bool,
    cmd_path: bool
    
}

pub type GenError = Box<std::error::Error>;
pub type GenResult<T> = Result<T, GenError>;


/// This is just a nice colorful wrapper around the path's is_writeable() method
fn check_config_permission() -> GenResult<bool>{
    let c = Config::default();
        match c.paths.config.is_writeable(){
            Ok(is_writeable) => {
                match is_writeable{
                    true => {
                        Ok(true)
                    },
                    false => {
                        let label = " Error ".on_red().bold();
                        let error_message = format!("you don't have the permissions to write to {}", c.paths.config);
                        println!("    {} {}", label, error_message);
                        Ok(false)
                    }
                }
            },
            Err(err) => {
                let label = " Error ".on_red().bold();
                println!("    {} while checking permissions on {}: {}", label, c.paths.config, err);
                Err(err)
            }
        }
}




/// Create a new default config.toml at the path specified in the bender_config
/// library. This uses the Structs default values.
fn new_default(){
    let c = Config::default();
    match check_config_permission(){
        Ok(is_writeable) if is_writeable => {
            let message = match c.paths.config.exists(){
                true => {
                    let overwrite = "overwrite".red();
                    format!("Do you want to {} the config at {} with the defaults?", overwrite, c.paths.config)
                },
                false => format!("Do you want to write the default config to {}?", c.paths.config)
            };
            if Confirmation::new(message.as_str()).interact().expect("Failed"){
                match c.write_changes(){
                    Ok(_) => {
                        let label = "  OK  ".on_green().bold();
                        println!("    {} Wrote default config to {}", label, c.paths.config)
                    },
                    Err(err) => {
                        let label = " Error ".on_red().bold();
                        println!("    {} Couldn't write default config to {}. Error: {}", label, c.paths.config, err)
                    }
                }
            }
        },
        _ => ()
    }
}




/// Print a config if it exists
fn show(){
    let c = Config::default();
    let p = c.paths.config;
    if p.exists(){
        match Config::from_file(p){
            Ok(c) => {
                match c.serialize(){
                    Ok(s) => println!("{}", s),
                    Err(err) => {
                        let label = " Error ".on_red().bold();
                        println!("    {} Couldn't read the config. Serialization failed with Error: {}", label, err);
                    }
                }
                
            },
            Err(err) => {
                let label = " Error ".on_red().bold();
                println!("    {} Couldn't read the config. Deserialization failed with Error: {}", label, err);
            }
        }
    }else{
        let label = " Error ".on_red().bold();
        println!("    {} there is no config at {}.\n    Create with bender-config new or bender-config new default", label, p);
    }
}




/// Print the configs path if it exists
fn path(){
    let c = Config::default();
    let p = c.paths.config;
    if p.exists(){
        println!("{}", p);
    }else{
        let label = " Error ".on_red().bold();
        println!("    {} there is no config at {}.\n    Create with bender-config new or bender-config new default", label, p);
    }
}




/// Validate the config
fn validate(){
    let c = Config::default();
    let p = c.paths.config;
    if p.exists(){
        match Config::from_file(p){
            Ok(c) => {
                match c.serialize(){
                    Ok(_) => {
                        let label = "  OK  ".on_green().bold();
                        println!("    {} the config at {} is valid TOML and is a valid bender config", label, c.paths.config)
                    },
                    Err(err) => {
                        let label = " Error ".on_red().bold();
                        println!("    {} Couldn't read the config. Serialization failed with Error: {}", label, err);
                    }
                }
                
            },
            Err(err) => {
                let label = " Error ".on_red().bold();
                println!("    {} Couldn't read the config. Deserialization failed with Error: {}", label, err);
            }
        }
    }else{
        let label = " Error ".on_red().bold();
        println!("    {} there is no config at {}.\n    Create with bender-config new or bender-config new default", label, p);
    }
}


// TODO: Implement Wizard
// TODO: Check for more values needed to be stored



/// Generate a new appsecret and put it into the private path. If there is already
/// a app.secret, prompt before attempting a overwrite
fn new_appsecret(){
    let c = Config::default();
    let p = c.paths.config;
    if p.exists(){
        match Config::from_file(p){
            Ok(c) => {
                match c.paths.private.is_writeable(){
                    Ok(is_writable) => match is_writable{
                        true => {
                            let message = match c.appsecret_exists(){
                                true => {
                                    let overwrite = "overwrite".red();
                                    format!("Do you want to {} the appsecret at {} with the defaults?", overwrite, c.get_appsecret_path())
                                },
                                false => format!("Do you want to write the appsecret to {}?", c.get_appsecret_path())
                            };
                            if Confirmation::new(message.as_str()).interact().expect("Failed"){
                                match c.write_appsecret(){
                                    Ok(_) => {
                                        let label = "  OK  ".on_green().bold();
                                        println!("    {} Wrote appsecret to {}", label, c.get_appsecret_path())
                                    },
                                    Err(err) => {
                                        let label = " Error ".on_red().bold();
                                        println!("    {} Couldn't write appsecret to {}. Error: {}", label, c.get_appsecret_path(), err)
                                    }
                                }
                            }
                        },
                        false => {
                            let label = " Error ".on_red().bold();
                            let error_message = format!("you don't have the permissions to write to {}", c.get_appsecret_path());
                            println!("    {} {}", label, error_message);
                        }
                    },
                    Err(err) => {
                        let label = " Error ".on_red().bold();
                        println!("    {} while checking permissions on {}: {}", label, c.get_appsecret_path(), err);
                    }
                }
                
            },
            Err(err) => {
                let label = " Error ".on_red().bold();
                println!("    {} Couldn't read the config. Deserialization failed with Error: {}", label, err);
            }
        }
    }else{
        let label = " Error ".on_red().bold();
        println!("    {} there is no config at {}.\n    Create with bender-config new or bender-config new default", label, p);
    }
}



fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.deserialize())
                            .unwrap_or_else(|e| e.exit());

    // Run configuration wizard if config is the sole command
    if args.cmd_new && !args.cmd_default && !args.cmd_appsecret{
        
    }

    // Create a new default config at the default path
    if args.cmd_new && args.cmd_default{
        new_default();
    }

    // Generate a new appsecret
    if args.cmd_new && args.cmd_appsecret{
        new_appsecret();
    }

    // Print the config if it exists
    if args.cmd_show{
        show();
    }

    // Get the config path if the config exists
    if args.cmd_path{
        path(); 
    }

    // Get the config path if the config exists
    if args.cmd_validate{
        validate(); 
    }



}