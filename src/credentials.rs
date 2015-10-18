use std::env::*;
use std::env;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::error::Error;
use std::io::prelude::*;
use std::io::BufReader;
use std::ascii::AsciiExt;
use std::collections::HashMap;
use hyper::Client;
use hyper::header::Connection;
use error::*;
use regex::Regex;


extern crate rustc_serialize;
use self::rustc_serialize::json::*;

extern crate chrono;
use self::chrono::*;

#[derive(Clone)]
pub struct Credentials{
	pub id: String,
	pub secret: String,
	pub token: Option<String>
}
impl Credentials{
	pub fn get_token(&self) -> Option<String>{
        self.token.clone()
    }
	pub fn get_id(&self) -> String{
        self.id.clone()
    }
	pub fn get_secret(&self) -> String{
        self.secret.clone()
    }
}

/*
#[derive(Clone, Debug)]
pub struct AWSCredentials {
    key: String,
    secret: String,
    token: Option<String>,
    expires_at: DateTime<UTC>
}*/
/*
impl AWSCredentials {
    pub fn new<K, S>(key:K, secret:S, token:Option<String>, expires_at:DateTime<UTC>) -> AWSCredentials where K:Into<String>, S:Into<String> {
        AWSCredentials {
            key: key.into(),
            secret: secret.into(),
            token: token,
            expires_at: expires_at,
        }
    }

    pub fn get_aws_access_key_id(&self) -> &str {
    	&self.key
    }

    pub fn get_aws_secret_key(&self) -> &str {
    	&self.secret
    }

    pub fn get_expires_at(&self) -> &DateTime<UTC> {
        &self.expires_at
    }

    pub fn get_token(&self) -> &Option<String> {
        &self.token
    }

    fn credentials_are_expired(&self) -> bool {
        //println!("Seeing if creds of {:?} are expired compared to {:?}", self.expires_at, UTC::now() + Duration::seconds(20));
        // This is a rough hack to hopefully avoid someone requesting creds then sitting on them
        // before issuing the request:
        if self.expires_at < (UTC::now() + Duration::seconds(20)) {
            return true;
        }
        return false;
    }
}*/
/*
pub trait AWSCredentialsProvider {
	fn get_credentials(&mut self) -> Result<&AWSCredentials, AWSError>;
}

pub struct EnvironmentCredentialsProvider {
    credentials: Option<AWSCredentials>
}

impl AWSCredentialsProvider for EnvironmentCredentialsProvider {

	fn get_credentials(&mut self) -> Result<&AWSCredentials, AWSError> {
        if self.credentials.is_none() || self.credentials.as_ref().unwrap().credentials_are_expired() {
           self.credentials = Some(try!(get_credentials_from_environment()));
        }
        Ok(self.credentials.as_ref().unwrap())
	}
}

impl EnvironmentCredentialsProvider {
    fn new() -> EnvironmentCredentialsProvider {
        EnvironmentCredentialsProvider { credentials: None }
    }

}

fn get_credentials_from_environment<'a>() -> Result<AWSCredentials, AWSError> {
    let env_key = match var("AWS_ACCESS_KEY_ID") {
        Ok(val) => val,
        Err(_) => return Err(AWSError::NoCredentials)
    };
    let env_secret = match var("AWS_SECRET_ACCESS_KEY") {
        Ok(val) => val,
        Err(_) => return Err(AWSError::NoCredentials)
    };

    if env_key.is_empty() || env_secret.is_empty() {
        return Err(AWSError::NoCredentials)
    }

    Ok(AWSCredentials::new(env_key, env_secret, None, in_ten_minutes()))
}

pub struct ProfileCredentialsProvider {
    profile: String,
    file_name: String,
    credentials: Option<AWSCredentials>
}

impl ProfileCredentialsProvider {
    pub fn with_configuration(profile: &str, file_name: &str) -> ProfileCredentialsProvider {
        ProfileCredentialsProvider { credentials: None, profile: profile.to_string(), file_name: file_name.to_string() }
    }

    pub fn with_profile(&mut self, profile: &str) -> &mut ProfileCredentialsProvider {
        self.profile = profile.to_string();
        self
    }

    pub fn get_profile(&self) -> &str {
        &self.profile
    }
}

impl AWSCredentialsProvider for ProfileCredentialsProvider {
    fn get_credentials(&mut self) -> Result<&AWSCredentials, AWSError> {
        if self.credentials.is_none() || self.credentials.as_ref().unwrap().credentials_are_expired() {
            match parse_credentials_file(&self.file_name) {
                Ok(mut profiles) => {
                    let default_profile = profiles.remove(&self.profile);
                    if default_profile.is_none() {
                        return Err(AWSError::NoCredentials)
                    }
                    self.credentials = default_profile;
                },
                Err(e) => {
					println!("file parse err: {:?}", e);
					return Err(AWSError::NoCredentials)
				}
            };
       }
       Ok(self.credentials.as_ref().unwrap())
   }
}

impl ProfileCredentialsProvider {
   pub fn new() -> ProfileCredentialsProvider {
        // Default credentials file location:
        // ~/.aws/credentials (Linux/Mac)
        // %USERPROFILE%\.aws\credentials  (Windows)
        let profile_location;
        match env::home_dir() {
            Some(ref p) => profile_location = p.display().to_string() + "/.aws/credentials",
            None => panic!("Couldn't get your home dir.")
        }

        ProfileCredentialsProvider { credentials: None, profile: "default".to_string(), file_name: profile_location }
    }
}

fn parse_credentials_file(file_with_path: &str) -> Result<HashMap<String, AWSCredentials>, AWSError> {
	println!("parsing credentials file: {:?}", file_with_path);
    let path = Path::new(&file_with_path);
    let display = path.display();

    match fs::metadata(&path) {
        Err(_) => return Err(AWSError::NoCredentials),
        Ok(metadata) => {
            if !metadata.is_file() {
                return Err(AWSError::NoCredentials)
            }
        }
    };

    let file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, Error::description(&why)),
        Ok(opened_file) => opened_file,
    };

    let profile_regex = Regex::new(r"^\[([^\]]+)\]$").unwrap();
    let mut profiles: HashMap<String, AWSCredentials> = HashMap::new();
    let mut access_key: Option<String> = None;
    let mut secret_key: Option<String> = None;
    let mut profile_name: Option<String> = None;

    let file_lines = BufReader::new(&file);
    for line in file_lines.lines() {
		println!("line: {:?}", line);
        let unwrapped_line : String = line.unwrap();

        // skip comments
        if unwrapped_line.starts_with('#') {
            continue;
        }

        // handle the opening of named profile blocks
        if profile_regex.is_match(&unwrapped_line) {

            if profile_name.is_some() && access_key.is_some() && secret_key.is_some() {
                let creds = AWSCredentials::new(access_key.unwrap(), secret_key.unwrap(), None, in_ten_minutes());
                profiles.insert(profile_name.unwrap(), creds);
            }

            access_key = None;
            secret_key = None;

            let caps = profile_regex.captures(&unwrapped_line).unwrap();
            profile_name = Some(caps.at(1).unwrap().to_string());
			println!("profile name: {:?}", profile_name);
            continue;
        }

        // otherwise look for key=value pairs we care about
        let lower_case_line = unwrapped_line.to_ascii_lowercase().to_string();

        if lower_case_line.contains("aws_access_key_id") {
			println!("found access_key_id line");
            if access_key.is_none() {
                let v: Vec<&str> = unwrapped_line.split("=").collect();
                if v.len() > 0 {
                    access_key = Some(v[1].trim_matches(' ').to_string());
                }
            }
        } else if lower_case_line.contains("aws_secret_access_key") {
            if secret_key.is_none() {
                let v: Vec<&str> = unwrapped_line.split("=").collect();
                if v.len() > 0 {
                    secret_key = Some(v[1].trim_matches(' ').to_string());
                }
            }
        }

        // we could potentially explode here to indicate that the file is invalid

    }

    if profile_name.is_some() && access_key.is_some() && secret_key.is_some() {
        let creds = AWSCredentials::new(access_key.unwrap(), secret_key.unwrap(), None, in_ten_minutes());
        profiles.insert(profile_name.unwrap(), creds);
    }

    if profiles.is_empty() {
        return Err(AWSError::NoCredentials)
    }

    Ok(profiles)
}

pub struct IAMRoleCredentialsProvider {
    credentials: Option<AWSCredentials>
}

impl IAMRoleCredentialsProvider {
    fn new() -> IAMRoleCredentialsProvider {
        IAMRoleCredentialsProvider { credentials: None }
    }
}

impl AWSCredentialsProvider for IAMRoleCredentialsProvider {

    fn get_credentials(&mut self) -> Result<&AWSCredentials, AWSError> {
        if self.credentials.is_none() || self.credentials.as_ref().unwrap().credentials_are_expired() {
            // TODO: backoff and retry on failure.

            //println!("Calling IAM metadata");
            // for "real" use: http://169.254.169.254/latest/meta-data/iam/security-credentials/
            let mut address : String = "http://169.254.169.254/latest/meta-data/iam/security-credentials".to_string();
            let client = Client::new();
            let mut response;
            match client.get(&address)
                .header(Connection::close()).send() {
                    Err(_) => return Err(AWSError::NoCredentials),//couldn't connect to meta-data service
                    Ok(received_response) => response = received_response
                };

            let mut body = String::new();
            match response.read_to_string(&mut body) {
                Err(_) => return Err(AWSError::NoCredentials),
                Ok(_) => (),
            };

            address.push_str("/");
            address.push_str(&body);
            body = String::new();
            match client.get(&address)
			.header(Connection::close()).send() {
				Err(_) => return Err(AWSError::NoCredentials),
				Ok(received_response) => response = received_response
			};

            match response.read_to_string(&mut body) {
                Err(_) => return Err(AWSError::NoCredentials),
                Ok(_) => {},
            };

            let json_object : Json;
            match Json::from_str(&body) {
                Err(_) => return Err(AWSError::NoCredentials),
                Ok(val) => json_object = val
            };

            let access_key;
            match json_object.find("AccessKeyId") {
                None => return Err(AWSError::NoCredentials),
                Some(val) => access_key = val.to_string().replace("\"", "")
            };

            let secret_key;
            match json_object.find("SecretAccessKey") {
                None => return Err(AWSError::NoCredentials),
                Some(val) => secret_key = val.to_string().replace("\"", "")
            };

            let expiration;
            match json_object.find("Expiration") {
                None => return Err(AWSError::NoCredentials),
                Some(val) => expiration = val.to_string().replace("\"", "")
            };

            let expiration_time;
            match expiration.parse::<DateTime<UTC>>() {
                Err(why) => return Err(AWSError::NoCredentials),
                Ok(val) => expiration_time = val
            };

            let token_from_response;
            match json_object.find("Token") {
                None => return Err(AWSError::NoCredentials),
                Some(val) => token_from_response = val.to_string().replace("\"", "")
            };

            self.credentials = Some(AWSCredentials::new(access_key, secret_key, Some(token_from_response.to_string()), expiration_time));
        }

		Ok(&self.credentials.as_ref().unwrap())
	}
}

#[derive(Debug, Clone)]
pub struct DefaultAWSCredentialsProviderChain {
    credentials: Option<AWSCredentials>,
    profile: String
}

// Chain the providers:
impl AWSCredentialsProvider for DefaultAWSCredentialsProviderChain {

    fn get_credentials(&mut self) -> Result<&AWSCredentials, AWSError> {
		println!("get credentials called");
        if self.credentials.is_none() || self.credentials.as_ref().unwrap().credentials_are_expired() {
            // fetch creds in order: env, file, IAM
			println!("don't have credentials... fetching from env");
			match EnvironmentCredentialsProvider::new().get_credentials(){
				Ok(creds) => {
					self.credentials = Some(creds.clone());
					return Ok(self.credentials.as_ref().unwrap())
				},
				Err(err) => println!("err: {:?}", err)
			}
			match ProfileCredentialsProvider::new().with_profile(&self.profile).get_credentials(){
				Ok(creds) => {
					self.credentials = Some(creds.clone());
					return Ok(self.credentials.as_ref().unwrap())
				},
				Err(err) => println!("err: {:?}", err)
			}
			/*match IAMRoleCredentialsProvider::new().get_credentials(){
				Ok(creds) => {
					self.credentials = Some(creds.clone());
					return Ok(self.credentials.as_ref().unwrap())
				},
				Err(err) => println!("err: {:?}", err)
			}*/
           return Err(AWSError::NoCredentials)
        }
        Ok(self.credentials.as_ref().unwrap())
    }
}

impl DefaultAWSCredentialsProviderChain {
    pub fn new() -> DefaultAWSCredentialsProviderChain {
        DefaultAWSCredentialsProviderChain { credentials: None, profile: "default".to_string() }
    }

    pub fn set_profile<S>(&mut self, profile: S) where S: Into<String> {
        self.profile = profile.into();
    }

    pub fn get_profile(&self) -> &str {
        &self.profile
    }
}

fn in_ten_minutes() -> DateTime<UTC> {
    UTC::now() + Duration::seconds(600)
}
*/