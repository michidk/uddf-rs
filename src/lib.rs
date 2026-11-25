#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod datetime;
mod error;
mod model;
mod read;
mod refs;
mod write;

pub use datetime::{DateTime, Offset, ParseDateTimeError};
pub use error::{Error, InvalidValue};
pub use model::*;
pub use refs::{References, Target};
pub use write::{NAMESPACE, VERSION};

/// Reads a UDDF document, failing on the first value that cannot be read as its UDDF type.
///
/// # Errors
///
/// [`Error::Xml`] for text that is not well-formed XML or carries a `<!DOCTYPE>`,
/// [`Error::NotUddf`] when the root element is not `<uddf>`, and
/// [`Error::InvalidValue`] for a malformed value such as a non-numeric `<depth>`.
pub fn from_str(text: &str) -> Result<Uddf, Error> {
    read::read(text, false).map(|(uddf, _)| uddf)
}

/// Reads a UDDF document, reading malformed values as absent and returning them as
/// warnings, in document order.
///
/// # Errors
///
/// [`Error::Xml`] and [`Error::NotUddf`], as for [`from_str`]; never
/// [`Error::InvalidValue`].
pub fn from_str_lenient(text: &str) -> Result<(Uddf, Vec<InvalidValue>), Error> {
    read::read(text, true)
}

/// Writes `uddf` as an indented UDDF 3.2.3 document in schema order.
///
/// # Errors
///
/// [`Error::Io`] if writing to `writer` fails.
pub fn to_writer<W: std::io::Write>(writer: W, uddf: &Uddf) -> Result<(), Error> {
    write::write(writer, uddf)
}

/// Writes `uddf` as an indented UDDF 3.2.3 document in schema order.
///
/// # Errors
///
/// None in practice; the signature matches [`to_writer`].
pub fn to_string(uddf: &Uddf) -> Result<String, Error> {
    let mut out = Vec::new();
    write::write(&mut out, uddf)?;
    String::from_utf8(out)
        .map_err(|error| Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, error)))
}
