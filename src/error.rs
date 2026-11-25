//! The crate's error type.

use core::fmt;

/// Everything that can go wrong while reading or writing UDDF.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The input is not well-formed XML, or it carries a document type declaration
    /// (`<!DOCTYPE>`), which UDDF has no use for and which is refused outright.
    #[error("malformed XML at {line}:{column}: {message}")]
    Xml {
        /// What the XML parser reported.
        message: String,
        /// 1-based line of the problem.
        line: u32,
        /// 1-based column of the problem.
        column: u32,
    },
    /// The root element is not `<uddf>` (matched case-insensitively, in any namespace).
    #[error("the root element is <{0}>, not <uddf>")]
    NotUddf(String),
    /// A value could not be read as the type UDDF declares for it.
    #[error(transparent)]
    InvalidValue(#[from] InvalidValue),
    /// Writing to the output failed.
    #[error("I/O error while writing UDDF: {0}")]
    Io(#[from] std::io::Error),
}

/// A value that could not be read as the type UDDF declares for it, such as a `<depth>`
/// holding `deep`.
///
/// [`crate::from_str`] returns it as [`Error::InvalidValue`]; [`crate::from_str_lenient`]
/// collects it as a warning and reads the value as absent.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct InvalidValue {
    /// Where the value sits, as lowercase element names from the root with a 1-based
    /// position where an element has same-named siblings, for example
    /// `/uddf/profiledata/repetitiongroup/dive[2]/samples/waypoint[17]/depth`. An attribute
    /// is appended as `/@name`.
    pub path: String,
    /// 1-based line of the element or attribute.
    pub line: u32,
    /// 1-based column of the element or attribute.
    pub column: u32,
    /// The text as found, trimmed.
    pub value: String,
    /// What was expected instead, such as `"a decimal number"`.
    pub expected: &'static str,
}

impl fmt::Display for InvalidValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at {}:{} is {:?}, expected {}",
            self.path, self.line, self.column, self.value, self.expected
        )
    }
}

impl core::error::Error for InvalidValue {}

impl From<roxmltree::Error> for Error {
    fn from(error: roxmltree::Error) -> Self {
        let pos = error.pos();
        Self::Xml {
            message: error.to_string(),
            line: pos.row,
            column: pos.col,
        }
    }
}
