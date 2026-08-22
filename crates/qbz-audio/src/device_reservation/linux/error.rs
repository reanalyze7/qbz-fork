use std::fmt;

#[derive(Debug)]
pub enum ReservationError {
    InvalidDevice(String),
    HigherPriorityHolder {
        holder_name: String,
        holder_priority: i32,
    },
    DbusError(String),
    AlsaError(String),
}

impl fmt::Display for ReservationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDevice(s) => write!(f, "invalid ALSA device string: {}", s),
            Self::HigherPriorityHolder {
                holder_name,
                holder_priority,
            } => write!(
                f,
                "device reserved by '{}' at priority {}",
                holder_name, holder_priority
            ),
            Self::DbusError(s) => write!(f, "D-Bus error: {}", s),
            Self::AlsaError(s) => write!(f, "ALSA error: {}", s),
        }
    }
}

impl std::error::Error for ReservationError {}
