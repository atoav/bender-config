#[macro_use]
extern crate serde_derive;
extern crate docopt;

use docopt::Docopt;

const USAGE: &'static str = "
bender-config

A cli to the bender-configuration file

Usage:
  bender-config config
  bender-config config show
  bender-config config get <key>
  bender-config config set <key>
  bender-config config path
  bender-config config reset

  bender-config (-h | --help)
  bender-config --version

Commands:
  config . . . . . . . .  Run the configuration wizard

  config show  . . . . .  Show the configuration file

  config get <key> . . .  Get a key from the configuration file
                          e.g. Paths will return the paths set

  config set <key> . . .  Set a key in the configuration file

  config path  . . . . .  Return the path of the configuration file

  config reset . . . . .  Reset the configuration to its default values




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
    arg_key: String,
    arg_job: Vec<String>,
    arg_x: Option<i32>,
    arg_y: Option<i32>,
    cmd_config: bool,
    cmd_set: bool,
    cmd_reset: bool,
    cmd_delete: bool,
    cmd_abort: bool,
    cmd_restart: bool,
    cmd_pause: bool,
    cmd_job: bool,
    cmd_all: bool,
    cmd_all_except: bool,
    cmd_list: bool,
    cmd_show: bool,
    cmd_info: bool,
    cmd_get: bool,
    cmd_path: bool,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.deserialize())
                            .unwrap_or_else(|e| e.exit());

    // Run configuration wizard if config is the sole command
    if args.cmd_config 
        && !args.cmd_set 
        && !args.cmd_reset 
        && !args.cmd_get 
        && !args.cmd_show 
        && !args.cmd_path{
        println!("Command: Config Wizard");
    }

    // Print the config
    if args.cmd_config && args.cmd_show{
        println!("Command: Print the config");
    }

    // Get individual key value pairs
    if args.cmd_config && args.cmd_get{
        println!("Command: Get config values");
    }

    // Set individual key value pairs
    if args.cmd_config && args.cmd_set{
        println!("Command: Set config values");
    }

    // Get the config path
    if args.cmd_config && args.cmd_path{
        println!("Command: get the config path");
    }

    // Reset the config to its initial state
    if args.cmd_config && args.cmd_reset{
        println!("Command: reset the config");
    }


}