use ansi_term::Colour::{Green, Red, Yellow};

#[derive(Clone, Copy, Debug)]
pub enum ColorMode {
    Colored,
    // No colors
    Plain,
}

impl ColorMode {
    pub fn yellow(&self, s: impl Into<String>) -> String {
        match self {
            ColorMode::Colored => Yellow.paint(s.into()).to_string(),
            ColorMode::Plain => s.into(),
        }
    }

    pub fn red(&self, s: impl Into<String>) -> String {
        match self {
            ColorMode::Colored => Red.paint(s.into()).to_string(),
            ColorMode::Plain => s.into(),
        }
    }

    pub fn green(&self, s: impl Into<String>) -> String {
        match self {
            ColorMode::Colored => Green.paint(s.into()).to_string(),
            ColorMode::Plain => s.into(),
        }
    }
}
