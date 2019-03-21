//! bender_config is a rust library, that deals with reading, writing and creating \
//! the config for the bender renderfarm. It consists of two parts:
//! - the rust library
//! - a CLI tool for creating and managing the config
//!
//! It can be loaded into a rust project via its git repository by putting this in your Cargo.toml:  
//! ```ignore
//! [dependencies]
//! bender_config = { git = "https://github.com/atoav/bender-config.git"}
//! ```
//! To update this run
//! ```ignore
//! cargo clean
//! cargo update
//! ```
//!
//! ## Testing
//! The libary is implemented with a extensive amount of tests to make
//! sure that repeated deserialization/serialization won't introduce
//! losses or glitches to the config file. The tests can be run with
//! ```ignore
//! cargo test
//! ```
//!
//! ## Documentation
//! If you want to view the documentation run
//! ```ignore
//! cargo doc --no-deps --open
//! ```
//! 
//! ## Installation
//! To run cargo, make sure you have rust installed. Go to [rustup.rs](http://rustup.rs) and follow the instructions there
//! To install the CLI tool `bender-config` just execute `./install.sh` for a guided setup



#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate rand;
extern crate blake2;
extern crate hex;
extern crate uuid;
extern crate dialoguer;
extern crate console;
extern crate colored;

use rand::prelude::*;
use rand::distributions::{Alphanumeric};

use std::process::Command;
use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;
use blake2::{Blake2b, Digest};
use uuid::Uuid;
use dialoguer::{Select, Input};
use std::fs::DirBuilder;

#[cfg(unix)]
use std::os::unix::fs::DirBuilderExt;



pub mod wizard;
use wizard::{Dialog, print_sectionlabel, print_block};


pub type GenError = Box<dyn std::error::Error>;
pub type GenResult<T> = Result<T, GenError>;



/// Return the path of the configuration by running `bender-cli config path`
pub fn path() -> GenResult<String>{
    let out = Command::new("bender-cli")
                       .arg("config")
                       .arg("path")
                       .output()?;
    let out = String::from_utf8_lossy(&out.stdout).to_string();
    let out = out.trim().to_string();
    if !out.contains("Error"){
        if std::path::PathBuf::from(out.clone()).exists(){
            Ok(out)
        }else{
            let errmsg = format!("config.toml doesn't exist at path {}", out);
            Err(From::from(errmsg))
        }
    }else{
        Ok(out)
    }
}




// ============================= CONFIG STRUCT ===============================

#[serde(default)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Config{
    pub servername: String,
    pub paths: Paths,
    pub flaskbender: Flaskbender,
    pub rabbitmq: RabbitMQ,
    pub janitor: Janitor,
    pub worker: Worker
}



impl Default for Config {
    fn default() -> Self { 
        Self{
            servername: "bender.render".to_string(),
            paths: Paths::default(),
            flaskbender: Flaskbender::default(),
            rabbitmq: RabbitMQ::default(),
            janitor: Janitor::default(),
            worker: Worker::default()
        }
    }
}


impl Config{
    /// Deserialize a Config from a string of text
    pub fn deserialize<S>(string: S) -> GenResult<Self> where S: Into<String>{
        let string = string.into();
        let config: Self = toml::from_str(string.as_str())?;
        Ok(config)
    }

    /// Deserialize a Config from a slice of bytes
    pub fn deserialize_from_u8(v: &[u8]) -> GenResult<Self>{
        let config: Self = toml::from_slice(v)?;
        Ok(config)
    }

    /// Serialize the Config to a pretty string
    pub fn serialize(&self) -> GenResult<String>{
        let serialized: String = toml::to_string_pretty(self)?;
        Ok(serialized)
    }

    /// Serialize the Config to a vector of bytes
    pub fn serialize_to_u8(&self) -> GenResult<Vec<u8>>{
        let serialized: Vec<u8> = toml::to_vec(self)?;
        Ok(serialized)
    }

    /// Deserialize the Config from a file
    pub fn from_file<S>(path: S) -> GenResult<Self> where S: Into<String>{
        let path = path.into();
        let mut file = fs::File::open(path.trim())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let deserialized = Self::deserialize(contents.as_str())?;
        Ok(deserialized)
    }

    /// Serialize the Config to a file
    pub fn to_file<S>(&self, path:S) -> GenResult<()> where S: Into<String>{
        let path = path.into();
        let mut file = fs::File::create(path.as_str())?;
        let serialized = self.serialize_to_u8()?;
        file.write_all(&serialized)?;
        Ok(())
    }

    /// Serialize the Config to the location specified in `self.paths.config`
    pub fn write_changes(&self) -> GenResult<()>{
        self.to_file(self.paths.config.clone())?;
        Ok(())
    }

    /// Update the Config from the location specified in `self.paths.config`
    pub fn read_changes(&mut self) -> GenResult<()>{
        let mut file = fs::File::open(self.paths.config.as_str())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let deserialized = Self::deserialize(contents.as_str())?;
        *self = deserialized;
        Ok(())
    }

    /// The goto method to get a config file. This is what other services should
    /// use. This relies on `bender-cli config path` to get the config path and
    /// fails horribly if no config is present or no bender-cli is in PATH
    pub fn get() -> Self{
        let configpath = match path(){
            Ok(c)     =>  c,
            Err(_err) =>  {
                eprintln!("Error: Didn't find a server configuration. Install bender-cli and run bender-cli setup! {}", _err);
                std::process::exit(1);
            }
        };

        // Check if bender-cli config path returned an error
        if configpath.contains("There is no config.toml at"){
            eprintln!("Error: There is no config.toml, use bender-cli to generate one");
            std::process::exit(1);
        }

        // Double check if the thing is really there
        if !std::path::PathBuf::from(configpath.clone()).exists(){
            eprintln!("Error: There is no config.toml, use bender-cli to generate one");
            std::process::exit(1);
        }

        // Finally try to deserialize the dame thing
        match Config::from_file(configpath){
            Ok(config) => config,
            Err(err)   =>{
                eprintln!("Error: Error while deserializing the configuration: {}", err);
                std::process::exit(1);
            }
        }
    }
}


impl Dialog for Config{
    fn ask() -> Self{
        let servername = Input::<String>::new()
                                        .with_prompt("The name of the server (displayed in the header of the website)")
                                        .default("bender.render".to_string())
                                        .interact()
                                        .expect("Couldn't display dialog.");
        
        Self{
            servername,
            paths: Paths::ask(),
            flaskbender: Flaskbender::ask(),
            rabbitmq: RabbitMQ::ask(),
            janitor: Janitor::ask(),
            worker: Worker::ask()
        }
    }

    fn compare(&self, other: Option<&Self>) -> Self{
        match other{
            Some(o) => {
                print_block(" The server name (shows up in frontend) ");
                let servername = wizard::differ(self.servername.clone(), Some(o.servername.clone()));
                Self{
                    servername,
                    paths: self.paths.compare(Some(&o.paths)),
                    flaskbender: self.flaskbender.compare(Some(&o.flaskbender)),
                    rabbitmq: self.rabbitmq.compare(Some(&o.rabbitmq)),
                    janitor: self.janitor.compare(Some(&o.janitor)),
                    worker: self.worker.compare(Some(&o.worker))
                }
            },
            None => {
                print_block(" The server name (shows up in frontend) ");
                let servername = wizard::differ(self.servername.clone(), None);
                Self{
                    servername,
                    paths: self.paths.compare(None),
                    flaskbender: self.flaskbender.compare(None),
                    rabbitmq: self.rabbitmq.compare(None),
                    janitor: self.janitor.compare(None),
                    worker: self.worker.compare(None)
                }
            }
        }
    }
}


impl Config{
    /// Returns true if the Config has the default values
    pub fn is_default(&self) -> bool{
        self == &Self::default()
    }

    /// Returns the path of the configuration
    pub fn location() -> String{
        Self::default().paths.config
    }
}


impl Config {
    /// Generates a 256 byte random Alphanumeric appsecret
    pub fn generate_appsecret() -> String{
        thread_rng().sample_iter(&Alphanumeric).take(256).collect()
    }

    /// Reads the appsecret from its path
    pub fn read_appsecret(&self) -> GenResult<String>{
        let mut file = fs::File::open(self.get_appsecret_path().as_str())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }

    /// Writes the appsecret to its path
    pub fn write_appsecret(&self) -> GenResult<()>{
        let mut file = fs::File::create(self.get_appsecret_path().as_str())?;
        let appsecret = Self::generate_appsecret();
        let appsecret = appsecret.as_bytes();
        file.write_all(&appsecret)?;
        Ok(())
    }

    /// Gets the appsecret path (basically push app.secret to the private path)
    pub fn get_appsecret_path(&self) -> String{
        let mut p = PathBuf::from(self.paths.private.clone());
        p.push("app.secret");
        p.to_str().unwrap().to_string()
    }

    /// Returns true if the app secret exists
    pub fn appsecret_exists(&self) -> bool{
        PathBuf::from(self.get_appsecret_path()).exists()

    }


    /// Return a salt to be use for private fields. The salt is a blake2 hashed
    /// version of the appsecret
    pub fn get_salt(&self) -> GenResult<String>{
        // Try to read the appsecret
        match self.read_appsecret(){
            Ok(appsecret) => {
                let mut hash = Blake2b::new();
                hash.input(&appsecret.clone().into_bytes());
                let x = hash.result();
                Ok(hex::encode(&x))
            },
            Err(err) => Err(err)
        }
    }
}






// ============================== PATHS STRUCT ===============================
#[serde(default)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Paths{
    pub config: Path,
    pub private: Path,
    pub upload: Path
}

impl Paths{
    /// Return a Path to blendfiles
    pub fn blend(&self) -> String{
        self.upload.push("blendfiles")
    }

    /// Return a Path to frames
    pub fn frames(&self) -> String{
        self.upload.push("frames")
    }
}


impl Default for Paths{
    fn default() -> Self{ 
        Self{
            config: "/etc/bender/config.toml".to_string(),
            private: "/var/lib/flask/private".to_string(),
            upload: "/data/bender".to_string()
        }
    }
}


type Path = String;

pub trait PathMethods{
    fn is_writeable(&self) -> GenResult<bool>;
    fn exists(&self) -> bool;
    fn push<S>(&self, s: S) -> String where S: Into<String>;
}

impl PathMethods for Path{
    /// Returns Ok(true) if the path is writeable and returns Ok(false) if not.
    /// For every other reason a write could have failed return a Error
    fn is_writeable(&self) -> GenResult<bool>{
        let p = PathBuf::from(self.clone());
        // Naive check: if this thing has a dot in it it must be a file
        match p.extension(){
            Some(_) => {
                let mut folder = p.clone();
                folder.pop();
                if !folder.exists(){
                    println!("Trying to create path to {}", folder.clone().to_str().unwrap());

                    // Create frames directory with 775 permissions on Unix
                    let mut builder = DirBuilder::new();

                    // Set the permissions to 775
                    #[cfg(unix)]
                    builder.mode(0o2775);
                    
                    builder.recursive(true)
                           .create(&folder)?;
                }
                let file = fs::OpenOptions::new().append(true)
                                                 .create(true)
                                                 .open(p.clone());

                match file{
                    Ok(f) => {
                        match f.metadata(){
                            Ok(metadata) => {
                                if let 0 = metadata.len() { fs::remove_file(p)?}
                            },
                            Err(err) => eprintln!("Error while retrieving metadata: {}", err)
                        }
                        Ok(true)
                    },
                    Err(err) => match err.kind(){
                        std::io::ErrorKind::PermissionDenied => Ok(false),
                        std::io::ErrorKind::AlreadyExists => Ok(true),
                        _ => Err(From::from(err))
                    }
                }
            }
            None => {
                // Create frames directory with 775 permissions on Unix
                let mut builder = DirBuilder::new();

                // Set the permissions to 775
                #[cfg(unix)]
                builder.mode(0o2775);
                
                match builder.recursive(true).create(&self) { 
                    Ok(_) => Ok(true),
                    Err(err) => match err.kind(){
                        std::io::ErrorKind::PermissionDenied => Ok(false),
                        std::io::ErrorKind::AlreadyExists => Ok(true),
                        _ => Err(From::from(err))
                    }
                }
            }
        }
    }

    /// Return true if the path exists
    fn exists(&self) -> bool{
        let p = PathBuf::from(self.clone());
        p.exists()
    }

    /// Push onto self
    fn push<S>(&self, s: S) -> String where S: Into<String>{
        let s = s.into();
        let mut p = PathBuf::from(self.clone());
        p.push(s.as_str());
        p.to_str().unwrap().to_string()
    }
}



impl Dialog for Paths{
    fn ask() -> Self{
        println!();
        print_sectionlabel("Paths");
        let config = "/etc/bender/config.toml".to_string();

        let private = Input::<String>::new().with_prompt("Specify the directory where the app.secret for flaskbender should be stored")
                                           .default("/var/lib/flask/private".to_string())
                                           .interact()
                                           .expect("Couldn't display dialog.");

        let upload = Input::<String>::new().with_prompt("Specify the directory where the uploaded blendfiles and the rendered frames will be stored")
                                           .default("/data/bender".to_string())
                                           .interact()
                                           .expect("Couldn't display dialog.");
        
        Self{
            config,
            private,
            upload,
        }
    }

    fn compare(&self, other: Option<&Self>) -> Self{
        println!();
        print_sectionlabel("Paths");
        match other{
            Some(o) => {
                let config = "/etc/bender/config.toml".to_string();
                print_block("\n config.paths.private (where the app.secret is stored) ");
                let private = wizard::differ(self.private.clone(), Some(o.private.clone()));
                print_block("\n config.paths.upload (where the both the uploaded blendfiles and the rendered frames are stored) ");
                let upload = wizard::differ(self.upload.clone(), Some(o.upload.clone()));
                Self{
                    config,
                    private,
                    upload,
                }
            },
            None => {
                let config = "/etc/bender/config.toml".to_string();
                print_block("\n config.paths.private (where the app.secret is stored) ");
                let private = wizard::differ(self.private.clone(), None);
                print_block("\n config.paths.upload (where the both the uploaded blendfiles and the rendered frames are stored) ");
                let upload = wizard::differ(self.upload.clone(), None);
                Self{
                    config,
                    private,
                    upload,
                }
            }
        }
    }
}





// ========================== FLASKBENDER STRUCT =============================
#[serde(default)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Flaskbender{
    pub upload_limit: usize,
    pub upload_url: String,
    pub job_cookie_name: String
}


impl Default for Flaskbender{
    fn default() -> Self{ 
        Self{
            upload_limit: 2,
            upload_url: "http://localhost:5000/blendfiles/".to_string(),
            job_cookie_name: "bender-renderjobs".to_string(),
        }
    }
}

impl Dialog for Flaskbender{
    fn ask() -> Self{
        println!();
        print_sectionlabel("Flaskbender");
        let upload_limit = Input::<usize>::new().with_prompt("The maximum upload size in GB")
                                                .default(2)
                                                .interact()
                                                .expect("Couldn't display dialog.");
        // let upload_url = Input::<usize>::new().with_prompt("The upload URL").default("http://localhost:5000/blendfiles/".to_string()).interact().expect("Couldn't display dialog.");
        let job_cookie_name = Input::<String>::new().with_prompt("The name of the secure cookie, where the users job IDs are stored")
                                                    .default("bender-renderjobs".to_string())
                                                    .interact()
                                                    .expect("Couldn't display dialog.");
        
        Self{
            upload_limit,
            upload_url: "http://localhost:5000/blendfiles/".to_string(),
            job_cookie_name,
        }
    }

    fn compare(&self, other: Option<&Self>) -> Self{
        println!();
        print_sectionlabel("Flaskbender");
        match other{
            Some(o) => {
                print_block("\n The upload limit (max file size) in GB ");
                let upload_limit = wizard::differ(self.upload_limit, Some(o.upload_limit));
                // let upload_url = wizard::differ(self.upload_url.clone(), Some(o.upload_url.clone()));
                print_block("\n The name of the secure cookie in which the client stores it's job ids ");
                let job_cookie_name = wizard::differ(self.job_cookie_name.clone(), Some(o.job_cookie_name.clone()));
                Self{
                    upload_limit,
                    upload_url: "http://localhost:5000/blendfiles/".to_string(),
                    job_cookie_name,
                }
            },
            None => {
                print_block("\n The upload limit (max file size) in GB ");
                let upload_limit = wizard::differ(self.upload_limit, None);
                // let upload_url = wizard::differ(self.upload_url.clone(), Some(o.upload_url.clone()));
                print_block("\n The name of the secure cookie in which the client stores it's job ids ");
                let job_cookie_name = wizard::differ(self.job_cookie_name.clone(), None);
                Self{
                    upload_limit,
                    upload_url: "http://localhost:5000/blendfiles/".to_string(),
                    job_cookie_name,
                }
            }
        }
    }
}




// ============================ RABBITMQ STRUCT ==============================
#[serde(default)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RabbitMQ{
    pub url: String
}


impl Default for RabbitMQ{
    fn default() -> Self{ 
        Self{
            url: "amqp://localhost//".to_string()
        }
    }
}

impl Dialog for RabbitMQ{
    fn ask() -> Self{
        println!();
        print_sectionlabel("RabbitMQ");
        let url = Input::<String>::new().with_prompt("RabbitMQ URL").default( "amqp://localhost//".to_string()).interact().expect("Couldn't display dialog.");
        
        Self{
            url
        }
    }

    fn compare(&self, other: Option<&Self>) -> Self{
        println!();
        print_sectionlabel("RabbitMQ");
        match other{
            Some(o) => {
                print_block("\n The AMQP URL for e.g. RabbitMQ ");
                let url = wizard::differ(self.url.clone(), Some(o.url.clone()));
                Self{
                    url
                }
            },
            None => {
                print_block("\n The AMQP URL for e.g. RabbitMQ ");
                let url = wizard::differ(self.url.clone(), None);
                Self{
                    url
                }
            }
        }
    }
}



// =========================== JANITOR STRUCT ==============================
#[serde(default)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Janitor{
    pub checking_period_seconds:       usize,
    pub error_deletion_min_minutes:    usize,
    pub error_deletion_max_minutes:    usize,
    pub finish_deletion_min_minutes:   usize,
    pub finish_deletion_max_minutes:   usize,
    pub cancel_deletion_min_minutes:   usize,
    pub cancel_deletion_max_minutes:   usize
}


impl Default for Janitor{
    fn default() -> Self{ 
        Self{
            checking_period_seconds: 60,
            error_deletion_min_minutes: 60*24,
            error_deletion_max_minutes: 60*24*14,
            finish_deletion_min_minutes: 60*24,
            finish_deletion_max_minutes: 60*24*14,
            cancel_deletion_min_minutes: 15,
            cancel_deletion_max_minutes: 15
        }
    }
}

impl Dialog for Janitor{
    fn ask() -> Self{
        println!();
        print_sectionlabel("bender-janitor");
        println!("The bender-janitor service cleans up jobs and job files that have somehow ended (e.g. canceled, errored, finished etc)");
        let checking_period_seconds = Input::<usize>::new().with_prompt("How frequenctly should the janitor check for cleaning? (in seconds)").default(60).interact().expect("Couldn't display dialog.");
        
        println!("\nThe bender-janitor will dynamically decide when to keep a job around for longer (e.g. when there is a lot of free disk space) and when to delete these jobs. You can specify minimum and maximum times:");
        let error_deletion_min_minutes    = Input::<usize>::new().with_prompt("Minimum grace period for deletion after error (in minutes)").default(60*24).interact().expect("Couldn't display dialog.");
        let error_deletion_max_minutes    = Input::<usize>::new().with_prompt("Maximum grace period for deletion after error (in minutes)").default(60*24*14).interact().expect("Couldn't display dialog.");
        let finish_deletion_min_minutes   = Input::<usize>::new().with_prompt("Minimum grace period for jobs finished, but not downloaded (in minutes)").default(60*24).interact().expect("Couldn't display dialog.");
        let finish_deletion_max_minutes   = Input::<usize>::new().with_prompt("Maximum grace period for jobs finished, but not downloaded (in minutes)").default(60*24*14).interact().expect("Couldn't display dialog.");
        let cancel_deletion_min_minutes   = Input::<usize>::new().with_prompt("Minimum grace period for canceled jobs (in minutes)").default(15).interact().expect("Couldn't display dialog.");
        let cancel_deletion_max_minutes   = Input::<usize>::new().with_prompt("Maximum grace period for canceled jobs (in minutes)").default(15).interact().expect("Couldn't display dialog.");

        Self{
            checking_period_seconds,
            error_deletion_min_minutes,
            error_deletion_max_minutes,
            finish_deletion_min_minutes,
            finish_deletion_max_minutes,
            cancel_deletion_min_minutes,
            cancel_deletion_max_minutes
        }
    }

    fn compare(&self, other: Option<&Self>) -> Self{
        println!();
        print_sectionlabel("bender-janitor");
        match other{
            Some(o) => {
                print_block("\n How often should the janitor check for cleanup? (in seconds) ");
                let checking_period_seconds = wizard::differ(self.checking_period_seconds, Some(o.checking_period_seconds));

                print_block("\n Minimum: How long to keep jobs after Error? (in minutes) ");
                let error_deletion_min_minutes    = wizard::differ(self.error_deletion_min_minutes, Some(o.error_deletion_min_minutes));
                print_block("\n Maximum: How long to keep jobs after Error? (in minutes) ");
                let error_deletion_max_minutes    = wizard::differ(self.error_deletion_max_minutes, Some(o.error_deletion_max_minutes));
                print_block("\n Minimum: How long to keep jobs after finish? (in minutes) ");
                let finish_deletion_min_minutes   = wizard::differ(self.finish_deletion_min_minutes, Some(o.finish_deletion_min_minutes));
                print_block("\n Maximum: How long to keep jobs after finish? (in minutes) ");
                let finish_deletion_max_minutes   = wizard::differ(self.finish_deletion_max_minutes, Some(o.finish_deletion_max_minutes));
                print_block("\n Minimum: How long to keep jobs after cancelation? (in minutes) ");
                let cancel_deletion_min_minutes   = wizard::differ(self.cancel_deletion_min_minutes, Some(o.cancel_deletion_min_minutes));
                print_block("\n Maximum: How long to keep jobs after cancelation? (in minutes) ");
                let cancel_deletion_max_minutes   = wizard::differ(self.cancel_deletion_max_minutes, Some(o.cancel_deletion_max_minutes));
                
                Self{
                    checking_period_seconds,
                    error_deletion_min_minutes,
                    error_deletion_max_minutes,
                    finish_deletion_min_minutes,
                    finish_deletion_max_minutes,
                    cancel_deletion_min_minutes,
                    cancel_deletion_max_minutes
                }
            },
            None => {
                print_block("\n How often should the janitor check for cleanup? (in seconds) ");
                let checking_period_seconds = wizard::differ(self.checking_period_seconds, None);

                print_block("\n Minimum: How long to keep jobs after Error? (in minutes) ");
                let error_deletion_min_minutes    = wizard::differ(self.error_deletion_min_minutes, None);
                print_block("\n Maximum: How long to keep jobs after Error? (in minutes) ");
                let error_deletion_max_minutes    = wizard::differ(self.error_deletion_max_minutes, None);
                print_block("\n Minimum: How long to keep jobs after finish? (in minutes) ");
                let finish_deletion_min_minutes   = wizard::differ(self.finish_deletion_min_minutes, None);
                print_block("\n Maximum: How long to keep jobs after finish? (in minutes) ");
                let finish_deletion_max_minutes   = wizard::differ(self.finish_deletion_max_minutes, None);
                print_block("\n Minimum: How long to keep jobs after cancelation? (in minutes) ");
                let cancel_deletion_min_minutes   = wizard::differ(self.cancel_deletion_min_minutes, None);
                print_block("\n Maximum: How long to keep jobs after cancelation? (in minutes) ");
                let cancel_deletion_max_minutes   = wizard::differ(self.cancel_deletion_max_minutes, None);
                
                Self{
                    checking_period_seconds,
                    error_deletion_min_minutes,
                    error_deletion_max_minutes,
                    finish_deletion_min_minutes,
                    finish_deletion_max_minutes,
                    cancel_deletion_min_minutes,
                    cancel_deletion_max_minutes
                }
            }
        }
    }
}





// =========================== WORKER STRUCT ==============================
#[serde(default)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Worker{
    pub id: Uuid,
    pub disklimit: u64,
    pub grace_period: u64,
    pub workload: usize,
    pub heart_rate_seconds: isize
}


impl Default for Worker{
    fn default() -> Self{ 
        Self{
            id: Uuid::new_v4(),       // Worker Random ID asigned uppon config
            disklimit: 2,             // in GB
            grace_period: 60,         // How many seconds to keep blendfiles,
            workload: 1,              // How many frames to render at once,
            heart_rate_seconds: 10    // How often to send out a heart beat
        }
    }
}


impl Dialog for Worker{
    fn ask() -> Self{
        println!();
        print_sectionlabel("bender-worker");
        println!("The bender-worker is the client that actually executes tasks from the queue. It can run on the server or on a client. This configuration is only relevant for workers running on the server.\n");
        let disklimit = Input::<u64>::new().with_prompt("How much disk space should the worker keep free? (in GB)").default(2).interact().expect("Couldn't display dialog.");
        let grace_period = Input::<u64>::new().with_prompt("How long should downloaded blendfiles be kept around (ireelevant on server)? (in secs)").default(60).interact().expect("Couldn't display dialog.");
        let workload = Input::<usize>::new().with_prompt("How many frames should the worker render at once?").default(1).interact().expect("Couldn't display dialog.");
        let heart_rate_seconds = Input::<isize>::new().with_prompt("How often should the worker send a heartbeat message to bender-qu at max (in seconds)?").default(10).interact().expect("Couldn't display dialog.");
        
        Self{
            id: Uuid::new_v4(),
            disklimit,
            grace_period,
            workload,
            heart_rate_seconds
        }
    }

    fn compare(&self, other: Option<&Self>) -> Self{
        println!();
        print_sectionlabel("bender-worker");
        match other{
            Some(o) => {
                print_block("\n The Workers disklimit in GB (if exceeded don't accept new jobs) ");
                let disklimit = wizard::differ(self.disklimit, Some(o.disklimit));
                print_block("\n The Workers grace period (how long downloaded blendfiles are kept around in seconds - irrelevant for server ");
                let grace_period = wizard::differ(self.grace_period, Some(o.grace_period));
                print_block("\n How many frames should a worker accept at once? ");
                let workload = wizard::differ(self.workload, Some(o.workload));
                print_block("\nHow often should the worker send a heartbeat message to bender-qu at max (in seconds)? ");
                let heart_rate_seconds = wizard::differ(self.heart_rate_seconds, Some(o.heart_rate_seconds));

                Self{
                    id: Uuid::new_v4(),
                    disklimit,
                    grace_period,
                    workload,
                    heart_rate_seconds
                }
            },
            None => {
                print_block("\n The Workers disklimit in GB (if exceeded don't accept new jobs) ");
                let disklimit = wizard::differ(self.disklimit, None);
                print_block("\n The Workers grace period (how long downloaded blendfiles are kept around in seconds - irrelevant for server ");
                let grace_period = wizard::differ(self.grace_period, None);
                print_block("\n How many frames should a worker accept at once? ");
                let workload = wizard::differ(self.workload, None);
                print_block("\nHow often should the worker send a heartbeat message to bender-qu at max (in seconds)? ");
                let heart_rate_seconds = wizard::differ(self.heart_rate_seconds, None);


                Self{
                    id: Uuid::new_v4(),
                    disklimit,
                    grace_period,
                    workload,
                    heart_rate_seconds
                }
            }
        }
    }
}







// =============================== UNIT TESTS ================================

#[cfg(test)]
mod unit_tests {
    use ::*;

    #[test]
    fn is_default() {
        let c = Config::default();
        assert_eq!(c.is_default(), true);
    }

    #[test]
    fn serialize_deserialize() {
        let c = Config::default();
        match c.serialize(){
            Ok(serialized) => {
                match Config::deserialize(serialized){
                    Ok(deserialized) => assert_eq!(c, deserialized),
                    Err(err) => println!("Error while deserializing serialized: {:?}", err)
                }
            },
            Err(err) => println!("Error while serializing c: {:?}", err)
        }
    }

    #[test]
    fn serialize_deserialize_u8() {
        let c = Config::default();
        match c.serialize_to_u8(){
            Ok(serialized) => {
                match Config::deserialize_from_u8(&serialized){
                    Ok(deserialized) => assert_eq!(c, deserialized),
                    Err(err) => println!("Error while deserializing serialized: {:?}", err)
                }
            },
            Err(err) => println!("Error while serializing c: {:?}", err)
        }
    }
}
