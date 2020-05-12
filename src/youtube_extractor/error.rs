use failure::Error;
use failure::Fail;

#[derive(Debug, Fail, Clone)]
pub enum ParsingError {

    #[fail(display = "Parsing Error : {}", cause)]
    ParsingError {
        cause: String,
    },

    #[fail(display = "Age restricted video not supported")]
    AgeRestricted,

    #[fail(display = "Download Error : {}", cause)]
    DownloadError{
        cause: String
    }
}


impl ParsingError{
    pub fn parsing_error_from_str(cause:&str)->Self{
        ParsingError::ParsingError {
            cause:cause.to_string()
        }
    }
}

impl From<&str> for ParsingError{
    fn from(cause: &str) -> Self {
        ParsingError::parsing_error_from_str(cause)
    }
}

impl From<String> for ParsingError{
    fn from(cause: String) -> Self {
        ParsingError::ParsingError {
            cause
        }
    }
}