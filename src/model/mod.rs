//! The typed UDDF model.
//!
//! Every type mirrors one UDDF element. Optional members are `Option`, repeated ones `Vec`;
//! an absent element is `None` (or an empty `Vec`), never a default value. Values keep
//! UDDF's own SI units, stated on each field: metres, kelvin, pascal, seconds, cubic
//! metres, kilograms, and fractions rather than percent unless a field says otherwise.
//!
//! Cross-references stay as the `id`/`ref` strings the file carries (trimmed of
//! surrounding whitespace); [`Uddf::references`](crate::Uddf::references) resolves them.

macro_rules! record {
    ($(#[$meta:meta])* pub struct $name:ident { $($body:tt)* }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Default, PartialEq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(default))]
        #[non_exhaustive]
        pub struct $name { $($body)* }
    };
}

macro_rules! vocabulary {
    (
        $(#[$meta:meta])* $name:ident {
            $($(#[$vmeta:meta])* $variant:ident = $text:literal,)*
        }
    ) => {
        $(#[$meta])*
        ///
        /// Read ASCII case-insensitively; a spelling outside the vocabulary is kept as
        /// `Unknown`. With the `serde` feature it serializes as its UDDF spelling.
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        #[non_exhaustive]
        pub enum $name {
            $($(#[$vmeta])* #[doc = concat!("UDDF spelling: \"", $text, "\"")] $variant,)*
            /// A value outside UDDF's vocabulary, kept as written.
            Unknown(String),
        }

        impl $name {
            /// The UDDF spelling.
            #[must_use]
            pub fn as_str(&self) -> &str {
                match self {
                    $(Self::$variant => $text,)*
                    Self::Unknown(text) => text,
                }
            }

            /// Reads a UDDF spelling, ASCII case-insensitively.
            #[must_use]
            pub fn from_uddf(text: &str) -> Self {
                $(if text.eq_ignore_ascii_case($text.trim()) {
                    return Self::$variant;
                })*
                Self::Unknown(text.to_owned())
            }
        }

        #[cfg(feature = "serde")]
        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(self.as_str())
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let text = <std::borrow::Cow<'de, str>>::deserialize(deserializer)?;
                Ok(Self::from_uddf(&text))
            }
        }
    };
}

mod common;
mod dive;
mod equipment;
mod people;
mod places;

pub use common::*;
pub use dive::*;
pub use equipment::*;
pub use people::*;
pub use places::*;

record! {
    /// A whole UDDF document, `<uddf>`.
    ///
    /// The top-level containers (`<diver>`, `<divesite>`, `<gasdefinitions>`, ...) are
    /// flattened into the lists they hold.
    pub struct Uddf {
        /// `@version` as read, such as `3.2.1`. Informational only: the writer always
        /// writes `3.2.3`.
        pub version: Option<String>,
        /// `<generator>`: the application that wrote the file.
        pub generator: Generator,
        /// `<maker>/<manufacturer>`: manufacturers of the equipment the file describes.
        pub makers: Vec<Manufacturer>,
        /// `<business>/<shop>`: dive shops.
        pub shops: Vec<Shop>,
        /// `<diver>/<owner>`: the diver whose log this is.
        pub owner: Option<Person>,
        /// `<diver>/<buddy>`: the people the owner dives with.
        pub buddies: Vec<Buddy>,
        /// `<divesite>/<divebase>`: dive centres.
        pub dive_bases: Vec<DiveBase>,
        /// `<divesite>/<site>`: dive sites.
        pub sites: Vec<Site>,
        /// `<divetrip>/<trip>`: trips.
        pub trips: Vec<Trip>,
        /// `<gasdefinitions>/<mix>`: breathing gases.
        pub mixes: Vec<Mix>,
        /// `<decomodel>`: decompression models dives may link.
        pub deco_models: Vec<DecoModel>,
        /// `<profiledata>/<repetitiongroup>`: the dives, grouped by surface interval.
        pub repetition_groups: Vec<RepetitionGroup>,
    }
}

impl Uddf {
    /// Every dive in file order, across repetition groups.
    pub fn dives(&self) -> impl Iterator<Item = &Dive> {
        self.repetition_groups
            .iter()
            .flat_map(|group| group.dives.iter())
    }
}

vocabulary! {
    /// `<generator>/<type>`: what kind of program wrote the file.
    GeneratorType {
        Converter = "converter",
        DiveComputer = "divecomputer",
        Logbook = "logbook",
    }
}

record! {
    /// `<generator>`: the program that wrote the file.
    pub struct Generator {
        /// `<name>`.
        pub name: Option<String>,
        /// `<aliasname>`.
        pub alias_names: Vec<String>,
        /// `<type>`.
        pub kind: Option<GeneratorType>,
        /// `<link ref>`.
        pub link: Option<String>,
        /// `<manufacturer>`.
        pub manufacturer: Option<Manufacturer>,
        /// `<version>`.
        pub version: Option<String>,
        /// `<datetime>`: when the file was written.
        pub datetime: Option<crate::DateTime>,
    }
}
