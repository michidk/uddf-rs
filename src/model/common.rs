//! Types shared by many elements.

record! {
    /// `<address>`.
    pub struct Address {
        /// `<street>`.
        pub street: Option<String>,
        /// `<city>`.
        pub city: Option<String>,
        /// `<postcode>`.
        pub postcode: Option<String>,
        /// `<country>` (required by the schema).
        pub country: Option<String>,
        /// `<province>`.
        pub province: Option<String>,
    }
}

record! {
    /// `<contact>`: ways to reach a person or organisation, each list in file order.
    pub struct Contact {
        /// `<language>`.
        pub languages: Vec<String>,
        /// `<phone>`.
        pub phones: Vec<String>,
        /// `<mobilephone>`.
        pub mobile_phones: Vec<String>,
        /// `<fax>`.
        pub faxes: Vec<String>,
        /// `<email>`.
        pub emails: Vec<String>,
        /// `<homepage>`.
        pub homepages: Vec<String>,
    }
}

/// One child of `<notes>`, in file order.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[non_exhaustive]
pub enum NoteItem {
    /// `<para>`: a paragraph of text, trimmed.
    Para(String),
    /// `<link ref>`: a reference to another element, typically `<mediadata>`.
    Link(String),
}

/// Paragraphs of a `<notes>` element joined with blank lines, or `None` if it has none.
#[must_use]
pub fn notes_text(notes: &[NoteItem]) -> Option<String> {
    let paras: Vec<&str> = notes
        .iter()
        .filter_map(|item| match item {
            NoteItem::Para(text) => Some(text.as_str()),
            NoteItem::Link(_) => None,
        })
        .collect();
    (!paras.is_empty()).then(|| paras.join("\n\n"))
}

record! {
    /// A price, such as `<priceperdive>`.
    pub struct Price {
        /// The amount.
        pub value: f64,
        /// `@currency`: an ISO 4217 code.
        pub currency: Option<String>,
    }
}

record! {
    /// `<pricedivepackage>`: the price of a package of dives.
    pub struct DivePackagePrice {
        /// The amount.
        pub value: f64,
        /// `@currency`: an ISO 4217 code.
        pub currency: Option<String>,
        /// `@noofdives`: how many dives the package holds.
        pub number_of_dives: Option<u32>,
    }
}

record! {
    /// `<rating>`.
    pub struct Rating {
        /// `<datetime>`: when the rating was given.
        pub datetime: Option<crate::DateTime>,
        /// `<ratingvalue>`: 1 (worst) to 10 (best) per the schema; kept as written.
        pub value: Option<u8>,
    }
}

record! {
    /// `<manufacturer>` (also `<maker>/<manufacturer>`).
    pub struct Manufacturer {
        /// `@id`.
        pub id: Option<String>,
        /// `<name>`.
        pub name: Option<String>,
        /// `<aliasname>`.
        pub alias_names: Vec<String>,
        /// `<address>`.
        pub address: Option<Address>,
        /// `<contact>`.
        pub contact: Option<Contact>,
    }
}
