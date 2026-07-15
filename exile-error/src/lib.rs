use std::fmt::{self};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoveModifierError<E> {
    ModifierNotFound,
    Validation(E),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReplaceModifierError<E> {
    ModifierNotFound,
    Validation(E),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationError {
    InvalidModifierId,
    InvalidModifierInstanceId,
    InvalidRemoveModifierError,
}

#[derive(Debug)]
pub enum ExileError {
    DeviceError { error: CpalDeviceError },
}

#[derive(Debug)]
pub enum CpalDeviceError {
    Kind { kind: cpal::ErrorKind },
}

impl ExileError {
    pub fn device_error(error: CpalDeviceError) -> Self {
        Self::DeviceError { error }
    }
}

impl fmt::Display for ExileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeviceError { error } => write!(f, "Device error: {}", error),
        }
    }
}

impl fmt::Display for CpalDeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Kind { kind } => write!(f, "CPAL error kind: {:?}", kind),
        }
    }
}

impl std::error::Error for ExileError {}
