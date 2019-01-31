//! bender_config is a rust library, that deals with reading, writing and creating \
//! the config for the bender renderfarm. It consists of two parts:
//! - the rust library
//! - a CLI tool for creating and managing the config
//!
//! It can be loaded into a rust project via its git repository by putting this in your Cargo.toml:  
//! ```ignore
//! [dependencies]
//! bender_config = { git = "ssh://git@code.hfbk.net:4242/bendercode/bender-config.git"}
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

use rand::prelude::*;
use rand::distributions::{Alphanumeric};

use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;
use blake2::{Blake2b, Digest};
use uuid::Uuid;


pub type GenError = Box<std::error::Error>;
pub type GenResult<T> = Result<T, GenError>;



// ============================== CONFIG STRUCT ================================

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






// =============================== PATHS STRUCT ================================
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
                    fs::create_dir_all(folder)?;
                }
                let file = fs::OpenOptions::new().append(true)
                                                 .create(true)
                                                 .open(p.clone());

                match file{
                    Ok(_) => {
                        fs::remove_file(p)?;
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
                match fs::create_dir_all(self){
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





// =========================== FLASKBENDER STRUCT ==============================
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




// ============================= RABBITMQ STRUCT ===============================
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


// =========================== JANITOR STRUCT ==============================
#[serde(default)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Janitor{
    pub error_period_minutes: usize,
    pub finish_period_minutes: usize
}


impl Default for Janitor{
    fn default() -> Self{ 
        Self{
            error_period_minutes: 240,
            finish_period_minutes: 240
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
    pub workload: usize
}


impl Default for Worker{
    fn default() -> Self{ 
        Self{
            id: Uuid::new_v4(),       // Worker Random ID asigned uppon config
            disklimit: 200*1_000_000, // in_MB
            grace_period: 60,         // How many seconds to keep blendfiles,
            workload: 1               // How many frames to render at once
        }
    }
}







// ================================ UNIT TESTS =================================

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
