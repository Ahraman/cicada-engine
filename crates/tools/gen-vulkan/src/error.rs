#[derive(Debug)]
pub enum Error {
    BadArg(String, String),

    BadElemStart(String),
    BadElemEnd(String),
    DocEnd,

    Io(std::io::Error),

    XmlRead(xml::reader::Error),

    Ureq(ureq::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BadArg(arg, value) => write!(f, "bad value '{value}' for argument '{arg}'"),

            Error::BadElemStart(tag) => write!(f, "bad element start <{tag}>"),
            Error::BadElemEnd(tag) => write!(f, "bad element end <{tag}>"),
            Error::DocEnd => write!(f, "document ended unexpectedly"),

            Error::XmlRead(error) => error.fmt(f),

            Error::Io(error) => error.fmt(f),

            Error::Ureq(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<xml::reader::Error> for Error {
    fn from(value: xml::reader::Error) -> Self {
        Self::XmlRead(value)
    }
}

impl From<ureq::Error> for Error {
    fn from(value: ureq::Error) -> Self {
        Self::Ureq(value)
    }
}
