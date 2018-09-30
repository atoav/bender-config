#[macro_use]
extern crate serde_derive;
extern crate toml;

use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;


pub type GenError = Box<std::error::Error>;
pub type GenResult<T> = Result<T, GenError>;



// ============================== CONFIG STRUCT ================================

#[serde(default)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Config{
    pub paths: Paths,
    pub limits: Limits,
}



impl Default for Config {
    fn default() -> Self { 
        Self{
            paths: Paths::default(),
            limits: Limits::default()
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
        let mut file = fs::File::open(path.as_str())?;
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
}






// =============================== PATHS STRUCT ================================
#[serde(default)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Paths{
    pub config: Path,
    pub private: Path,
    pub upload: Path
}


impl Default for Paths{
    fn default() -> Self{ 
        Self{
            config: "/etc/bender/config.toml".to_string(),
            private: "./private".to_string(),
            upload: "/data".to_string()
        }
    }
}

type Path = String;

pub trait PathMethods{
    fn is_writeable(&self) -> GenResult<bool>;
    fn exists(&self) -> bool;
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
                    Err(err) => Err(From::from(err))
                }
            }
        }
    }

    /// Return true if the path exists
    fn exists(&self) -> bool{
        let p = PathBuf::from(self.clone());
        p.exists()
    }
}





// ============================== LIMITS STRUCT ================================
#[serde(default)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Limits{
    pub upload: usize
}


impl Default for Limits{
    fn default() -> Self{ 
        Self{
            upload: 2
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
