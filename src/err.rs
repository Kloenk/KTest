use std::str::Utf8Error;
use tracing::*;

#[derive(Debug)]
pub enum ErrorKind {
    None,
    Io(std::io::Error),
    Errno(nix::errno::Errno),
    Utf8(core::str::Utf8Error),

    Clap(clap::Error),
}

#[derive(Debug)]
pub struct Error {
    pub exit_code: Option<i32>,
    pub context: String,
    pub kind: ErrorKind,
}

impl Error {
    pub fn new(context: impl Into<String>) -> Self {
        Self {
            exit_code: None,
            context: context.into(),
            kind: ErrorKind::None,
        }
    }

    pub fn set_exit_code(mut self, exit_code: impl Into<Option<i32>>) -> Self {
        self.exit_code = exit_code.into();
        self
    }

    pub fn exit_code(&self) -> i32 {
        self.exit_code.unwrap_or(1)
    }

    pub fn exit(&self) -> ! {
        println!("Failed to run ktest");
        if !self.context.is_empty() {
            println!("{}", self.context)
        }
        println!("{}", self);

        std::process::exit(self.exit_code())
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.kind {
            ErrorKind::None => write!(f, "{}", &self.context),
            ErrorKind::Io(err) => err.fmt(f),
            ErrorKind::Errno(err) => err.fmt(f),
            ErrorKind::Utf8(err) => err.fmt(f),

            ErrorKind::Clap(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match &self.kind {
            ErrorKind::Io(err) => err,
            ErrorKind::Errno(err) => err,
            ErrorKind::Utf8(err) => err,

            _ => return None,
        })
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        trace!("creating error from std::io::Error: {:?}", value);
        let exit_code = value.raw_os_error().map(i32::from);
        let kind = ErrorKind::Io(value);

        Self {
            exit_code,
            context: String::new(),
            kind,
        }
    }
}

impl From<nix::errno::Errno> for Error {
    fn from(errno: nix::errno::Errno) -> Self {
        let exit_code = (errno as i32).try_into().ok();
        let kind = ErrorKind::Errno(errno);

        Self {
            exit_code,
            context: String::new(),
            kind,
        }
    }
}

impl From<core::str::Utf8Error> for Error {
    fn from(value: Utf8Error) -> Self {
        Self {
            exit_code: None,
            context: String::new(),
            kind: ErrorKind::Utf8(value),
        }
    }
}

impl From<clap::Error> for Error {
    fn from(err: clap::Error) -> Self {
        let kind = ErrorKind::Clap(err);

        Self {
            exit_code: None,
            context: String::new(),
            kind,
        }
    }
}

pub type Result<T = (), E = Error> = core::result::Result<T, E>;

pub trait Context<T> {
    fn context(self, context: impl Into<String>) -> Result<T>;
}

impl<T, E> Context<T> for Result<T, E>
where
    E: Into<Error>,
{
    fn context(self, context: impl Into<String>) -> Result<T> {
        self.map_err(|err| {
            let mut err: Error = err.into();
            err.context = context.into();
            err
        })
    }
}

impl<T> Context<T> for Option<T> {
    fn context(self, context: impl Into<String>) -> Result<T> {
        self.ok_or_else(|| Error {
            exit_code: None,
            context: context.into(),
            kind: ErrorKind::None,
        })
    }
}
