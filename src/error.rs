use xmlutil::XmlParseError;

#[derive(Debug)]
pub enum AWSError{
	XmlParse(String),
	NoCredentials
}

impl From<XmlParseError> for AWSError {
	fn from(err: XmlParseError) -> AWSError {
		AWSError::XmlParse(format!("{:?}", err))
	}
}
