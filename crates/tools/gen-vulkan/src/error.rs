#[derive(Debug)]
pub enum Error {
    Cmd(CmdError),
    Parse(xml::common::TextPosition, ParseError),
    Trans(TransError),
    Emit(EmitError),

    Io(std::io::Error),
}

impl Error {
    pub fn parse(pos: &impl xml::common::Position, error: ParseError) -> Self {
        Self::Parse(pos.position(), error)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cmd(error) => error.fmt(f),
            Self::Parse(pos, error) => write!(f, "error at {pos}: {error}"),
            Self::Trans(error) => error.fmt(f),
            Self::Emit(error) => error.fmt(f),

            Self::Io(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl From<CmdError> for Error {
    fn from(value: CmdError) -> Self {
        Self::Cmd(value)
    }
}

impl From<TransError> for Error {
    fn from(value: TransError) -> Self {
        Self::Trans(value)
    }
}

impl From<EmitError> for Error {
    fn from(value: EmitError) -> Self {
        Self::Emit(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug)]
pub enum CmdError {
    BadCmdArg(String),
    ReqCmdArg(String),
}

impl std::fmt::Display for CmdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadCmdArg(arg) => write!(f, "bad command-line argument '{arg}'"),
            Self::ReqCmdArg(arg) => {
                write!(f, "command-line argument '{arg}' unexpectedly terminated")
            }
        }
    }
}

impl std::error::Error for CmdError {}

#[derive(Debug)]
pub enum ParseError {
    EmptyReg,
    BadStart(String),
    BadCont(String),
    BadChild(String, String),
    ReqAttrib(String, String),
    BadAttrib(String, String, String),
    UnreadAttrib(String, String, String),
    BadEnd(String),
    DocEnd,

    ParseBool(std::str::ParseBoolError),

    XmlRead(xml::reader::Error),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyReg => write!(f, "empty registry"),
            Self::BadStart(element) => write!(f, "bad start <{element}>"),
            Self::BadCont(content) => write!(f, "bad content '{content}'"),
            Self::BadChild(element, child) => {
                write!(f, "element <{element}> has invalid child <{child}>")
            }
            Self::ReqAttrib(element, attrib) => {
                write!(f, "element <{element}> missing attribute '{attrib}'")
            }
            Self::BadAttrib(element, attrib, value) => write!(
                f,
                "element <{element}> has bad attribute '{attrib}={value}'"
            ),
            Self::UnreadAttrib(element, attrib, value) => write!(
                f,
                "element <{element}> has unread attribute '{attrib}={value}"
            ),
            Self::BadEnd(element) => write!(f, "bad tag ending </{element}>"),
            Self::DocEnd => write!(f, "document ended"),

            Self::ParseBool(error) => error.fmt(f),

            Self::XmlRead(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<std::str::ParseBoolError> for ParseError {
    fn from(value: std::str::ParseBoolError) -> Self {
        Self::ParseBool(value)
    }
}

impl From<xml::reader::Error> for ParseError {
    fn from(value: xml::reader::Error) -> Self {
        Self::XmlRead(value)
    }
}

#[derive(Debug)]
pub enum TransError {}

impl std::fmt::Display for TransError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl std::error::Error for TransError {}

#[derive(Debug)]
pub enum EmitError {
    Syn(syn::Error),
}

impl std::fmt::Display for EmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Syn(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for EmitError {}

impl From<syn::Error> for EmitError {
    fn from(value: syn::Error) -> Self {
        Self::Syn(value)
    }
}
