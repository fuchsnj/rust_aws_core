use xmlutil::XmlParseError;

#[derive(Debug)]
pub enum AWSError{
	XmlParse(String),
	NoCredentials,
	ProtocolError(String)
}
impl AWSError{
	pub fn protocol_error(msg: &str) -> AWSError{
		AWSError::ProtocolError(msg.to_owned())
	}
}

impl From<XmlParseError> for AWSError {
	fn from(err: XmlParseError) -> AWSError {
		AWSError::XmlParse(format!("{:?}", err))
	}
}
