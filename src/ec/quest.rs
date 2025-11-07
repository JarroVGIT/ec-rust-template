use std::fmt;

/// Represents a quest day (1-25 typically)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Quest(u8);

impl Quest {
    pub const fn new(day: u8) -> Self {
        Self(day)
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        let day = s
            .parse::<u8>()
            .map_err(|_| format!("Invalid quest number: {}", s))?;

        if day == 0 {
            return Err("Quest number must be greater than 0".to_string());
        }

        Ok(Self(day))
    }

    pub fn as_u8(&self) -> u8 {
        self.0
    }
}

impl fmt::Display for Quest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}", self.0)
    }
}

/// Creates a Quest from a number literal
#[macro_export]
macro_rules! quest {
    ($day:expr) => {
        $crate::Quest::new($day)
    };
}
