use std::fmt::{self, Display};

use crate::frontend::error::FrontendError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmulatorErrorKind {
    MemoryAccessOutOfBounds,
    MemoryAccessReadOnly,
    UnknownOpcode,
    Misc,
}

impl Display for EmulatorErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmulatorErrorKind::MemoryAccessOutOfBounds => {
                write!(f, "attempted out of bounds memory access")
            }
            EmulatorErrorKind::MemoryAccessReadOnly => {
                write!(f, "attempted read only memory access")
            }
            EmulatorErrorKind::UnknownOpcode => {
                write!(f, "attempted execution of unknown opcode")
            }
            EmulatorErrorKind::Misc => write!(f, "misc error"),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    Emulator(EmulatorErrorKind, String),
    Other(String),
}

impl Error {
    pub fn new<S>(msg: S) -> Error
    where
        S: Into<String>,
    {
        Error::Emulator(EmulatorErrorKind::Misc, msg.into())
    }

    pub fn emulator<S>(kind: EmulatorErrorKind, msg: S) -> Error
    where
        S: Into<String>,
    {
        Error::Emulator(kind, msg.into())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Emulator(kind, msg) => write!(f, "Emulator: {} - {}", kind, msg),
            Self::Other(msg) => write!(f, "Other: {}", msg),
        }
    }
}

impl<E: Display> From<FrontendError<E>> for Error {
    fn from(err: FrontendError<E>) -> Self {
        Self::Other(format!("{}", err))
    }
}

impl From<fmt::Error> for Error {
    fn from(err: fmt::Error) -> Self {
        Self::Other(format!("{:?}", err))
    }
}
