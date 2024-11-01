#[derive(Debug)]
pub enum Error {
    Cmd(CmdError),
    Emit(EmitError),

    Io(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cmd(error) => error.fmt(f),
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
