#[derive(Debug)]
pub enum Error {
    Cmd(CmdError),
    Parse(xml::common::TextPosition, ParseError),
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
            Self::Emit(error) => error.fmt(f),
            Self::Parse(pos, error) => write!(f, "error at {pos}: {error}"),

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
    BadStart(String, String),
    UnexpStart(String, String),
    UnexpEnd(String, String),
    DocEnd(String),
    UnexpCont(String, String),
    MissingAttrib(String, String),
    BadAttrb(String, String, String),
    BadDeprAttrib(String, String),
    TypeNotFound(String),
    BadType,

    ParseBool(std::str::ParseBoolError),

    XmlRead(xml::reader::Error),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadStart(expected, element) => {
                write!(f, "expected element <{expected}> but found <{element}>")
            }
            Self::UnexpStart(element, child) => {
                write!(f, "element <{element}> has unexpected child <{child}>")
            }
            Self::UnexpEnd(element, end) => {
                write!(f, "element <{element}> contains unexpected end </{end}>")
            }
            Self::DocEnd(element) => {
                write!(f, "unexpected eof at element <{element}>")
            }
            Self::UnexpCont(element, content) => {
                write!(f, "element <{element}> has unexpected content '{content}'")
            }
            Self::MissingAttrib(element, attrib) => {
                write!(f, "element <{element}> missing required attrib '{attrib}'")
            }
            Self::BadAttrb(element, attrib, value) => write!(
                f,
                "element <{element}> has bad attribute '{attrib}={value}'"
            ),
            Self::BadDeprAttrib(attrib, value) => {
                write!(f, "attrib '{attrib}' has invalid value '{value}'")
            }
            Self::TypeNotFound(ty) => {
                write!(f, "type '{ty}' required but not found")
            }
            Self::BadType => {
                write!(f, "type with no name")
            }

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
pub enum EmitError {
    BadStructMember(String, String),

    Syn(syn::Error),
}

impl std::fmt::Display for EmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadStructMember(struct_name, member_name) => {
                write!(f, "bad member in struct '{struct_name}->{member_name}'")
            }

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
