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
    BadElemStart(String, String),
    UnexpEnd(String),
    UnexpCont(String, String),
    UnexpChild(String, String),

    XmlRead(xml::reader::Error),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadElemStart(expected, element) => {
                write!(f, "expected element <{expected}> but found <{element}>")
            }
            Self::UnexpEnd(element) => {
                write!(f, "unexpected eof at element <{element}>")
            }
            Self::UnexpCont(element, content) => {
                write!(f, "element <{element}> has unexpected content '{content}'")
            }
            Self::UnexpChild(element, child) => {
                write!(f, "element <{element}> has unexpected child <{child}>")
            }

            Self::XmlRead(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for ParseError {}

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
