/// a simple struct(String) for reporting hcid errors
#[derive(Debug, PartialEq, Clone)]
pub struct HcidError(String);

impl std::fmt::Display for HcidError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for HcidError {
    fn description(&self) -> &str {
        &self.0
    }
    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}

impl From<data_encoding::SpecificationError> for HcidError {
    fn from(error: data_encoding::SpecificationError) -> Self {
        Self(format!("{:?}", error))
    }
}

impl From<data_encoding::DecodeError> for HcidError {
    fn from(error: data_encoding::DecodeError) -> Self {
        Self(format!("{:?}", error))
    }
}

impl From<reed_solomon::DecoderError> for HcidError {
    fn from(error: reed_solomon::DecoderError) -> Self {
        Self(format!("{:?}", error))
    }
}

impl From<std::num::ParseIntError> for HcidError {
    fn from(error: std::num::ParseIntError) -> Self {
        Self(format!("{:?}", error))
    }
}
