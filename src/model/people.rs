//! `<diver>`: the owner, buddies and what UDDF records about a person.

use super::{Address, Contact, Equipment, NoteItem};
use crate::DateTime;

record! {
    /// `<owner>`, and the person part of a `<buddy>` (UDDF's `personType`).
    pub struct Person {
        /// `@id`.
        pub id: Option<String>,
        /// `<personal>` (required by the schema; written even when empty).
        pub personal: Personal,
        /// `<address>`.
        pub address: Option<Address>,
        /// `<contact>`.
        pub contact: Option<Contact>,
        /// `<equipment>`: the person's own kit.
        pub equipment: Equipment,
        /// `<education>/<certification>`.
        pub certifications: Vec<Certification>,
        /// `<diveinsurances>/<insurance>`.
        pub insurances: Vec<Insurance>,
        /// `<notes>`.
        pub notes: Vec<NoteItem>,
    }
}

record! {
    /// `<buddy>`.
    pub struct Buddy {
        /// Everything a buddy shares with an owner.
        #[cfg_attr(feature = "serde", serde(flatten))]
        pub person: Person,
        /// `<student/>`: the buddy is the owner's student.
        pub student: bool,
    }
}

vocabulary! {
    /// `<sex>`.
    Sex {
        Undetermined = "undetermined",
        Male = "male",
        Female = "female",
        Hermaphrodite = "hermaphrodite",
    }
}

record! {
    /// `<personal>`.
    pub struct Personal {
        /// `<firstname>`.
        pub first_name: Option<String>,
        /// `<middlename>`.
        pub middle_name: Option<String>,
        /// `<lastname>`.
        pub last_name: Option<String>,
        /// `<birthname>`.
        pub birth_name: Option<String>,
        /// `<honorific>`.
        pub honorific: Option<String>,
        /// `<sex>`.
        pub sex: Option<Sex>,
        /// `<birthdate>/<datetime>`.
        pub birth_date: Option<DateTime>,
        /// `<passport>`.
        pub passport: Option<String>,
        /// `<membership>`: an organisation the person belongs to.
        pub membership: Option<Membership>,
        /// `<height>`, metres.
        pub height: Option<f64>,
        /// `<weight>`, kilograms.
        pub weight: Option<f64>,
        /// `<bloodgroup>`.
        pub blood_group: Option<String>,
        /// `<smoking>`: cigarettes per day as one of UDDF's bands (`0`, `0-3`, `4-10`,
        /// `11-20`, `21-40`, `40+`).
        pub smoking: Option<String>,
    }
}

record! {
    /// `<membership>`.
    pub struct Membership {
        /// `@organisation`.
        pub organisation: Option<String>,
        /// `@memberid`.
        pub member_id: Option<String>,
    }
}

record! {
    /// `<certification>`.
    pub struct Certification {
        /// `<level>`.
        pub level: Option<String>,
        /// `<specialty>`.
        pub specialty: Option<String>,
        /// `<certificatenumber>`.
        pub certificate_number: Option<String>,
        /// `<organization>`: the certifying agency, as text.
        pub organization: Option<String>,
        /// `<instructor>`.
        pub instructor: Option<Instructor>,
        /// `<issuedate>/<datetime>`.
        pub issue_date: Option<DateTime>,
        /// `<validdate>/<datetime>`.
        pub valid_date: Option<DateTime>,
        /// `<notes>`.
        pub notes: Vec<NoteItem>,
    }
}

record! {
    /// `<instructor>` of a certification.
    pub struct Instructor {
        /// `@id`.
        pub id: Option<String>,
        /// `<personal>`.
        pub personal: Personal,
        /// `<address>`.
        pub address: Option<Address>,
        /// `<contact>`.
        pub contact: Option<Contact>,
        /// `<notes>`.
        pub notes: Vec<NoteItem>,
    }
}

record! {
    /// `<insurance>`.
    pub struct Insurance {
        /// `<name>`: the insurer.
        pub name: Option<String>,
        /// `<aliasname>`.
        pub alias_names: Vec<String>,
        /// `<issuedate>/<datetime>`.
        pub issue_date: Option<DateTime>,
        /// `<validdate>/<datetime>`: the last day of cover.
        pub valid_date: Option<DateTime>,
        /// `<notes>`.
        pub notes: Vec<NoteItem>,
    }
}
