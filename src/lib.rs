extern crate hyper;
extern crate regex;
extern crate xml;
extern crate url;
extern crate openssl;
extern crate rustc_serialize;
extern crate time;

mod credentials;
mod error;
mod xmlutil;
mod signature;
mod params;
mod regions;
mod request;

pub use credentials::Credentials;
pub use signature::SignedRequest;
pub use regions::Region;
pub use error::AWSError;

pub type AWSResult<T> = Result<T, AWSError>;
