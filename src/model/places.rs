//! Shops, dive bases, dive sites and trips.

use super::{Address, Contact, DivePackagePrice, NoteItem, Price, Rating};
use crate::DateTime;

record! {
    /// `<shop>`: under `<business>`, or inline in a `<purchase>`.
    pub struct Shop {
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
        /// `<notes>`.
        pub notes: Vec<NoteItem>,
    }
}

record! {
    /// `<divebase>`: a dive centre.
    pub struct DiveBase {
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
        /// `<priceperdive>`.
        pub price_per_dive: Option<Price>,
        /// `<pricedivepackage>`.
        pub price_dive_package: Option<DivePackagePrice>,
        /// `<guide>`: buddies guiding for this base.
        pub guides: Vec<Guide>,
        /// `<rating>`.
        pub ratings: Vec<Rating>,
        /// `<link ref>`.
        pub link: Option<String>,
        /// `<notes>`.
        pub notes: Vec<NoteItem>,
    }
}

record! {
    /// `<guide>`: a buddy leading dives for a dive base. A dive may link the guide's id.
    pub struct Guide {
        /// `@id`.
        pub id: Option<String>,
        /// `<link ref>`: the `<buddy>` who guides.
        pub link: Option<String>,
    }
}

vocabulary! {
    /// `<environment>` of a site.
    Environment {
        /// The schema spells it with a trailing space, which is written; `unknown` is read too.
        Undetermined = "unknown ",
        OceanSea = "ocean-sea",
        LakeQuarry = "lake-quarry",
        RiverSpring = "river-spring",
        CaveCavern = "cave-cavern",
        Pool = "pool",
        HyperbaricChamber = "hyperbaric-chamber",
        UnderIce = "under-ice",
        Other = "other",
    }
}

record! {
    /// `<site>`: a dive site.
    pub struct Site {
        /// `@id`.
        pub id: Option<String>,
        /// `<name>`.
        pub name: Option<String>,
        /// `<aliasname>`.
        pub alias_names: Vec<String>,
        /// `<environment>`.
        pub environment: Option<Environment>,
        /// `<geography>`.
        pub geography: Option<Geography>,
        /// `<sitedata>`.
        pub site_data: Option<SiteData>,
        /// `<rating>`.
        pub ratings: Vec<Rating>,
        /// `<notes>`.
        pub notes: Vec<NoteItem>,
    }
}

record! {
    /// `<geography>`: where a site or trip part is.
    pub struct Geography {
        /// `<location>`: the locality's name (required by the schema).
        pub location: Option<String>,
        /// `<address>`.
        pub address: Option<Address>,
        /// `<latitude>`, decimal degrees north. An empty element reads as `None`.
        pub latitude: Option<f64>,
        /// `<longitude>`, decimal degrees east.
        pub longitude: Option<f64>,
        /// `<altitude>`, metres above sea level.
        pub altitude: Option<f64>,
        /// `<timezone>`, hours east of UTC.
        pub timezone: Option<f64>,
    }
}

record! {
    /// `<sitedata>`: facts about a site. Wrecks, caves, indoor pools, lakes, rivers and
    /// shores are not modelled.
    pub struct SiteData {
        /// `<arealength>`, metres.
        pub area_length: Option<f64>,
        /// `<areawidth>`, metres.
        pub area_width: Option<f64>,
        /// `<averagevisibility>`, metres.
        pub average_visibility: Option<f64>,
        /// `<bottom>`: what the seabed is.
        pub bottom: Option<String>,
        /// `<density>`, water density in kg/m³.
        pub density: Option<f64>,
        /// `<difficulty>`, 1 to 10.
        pub difficulty: Option<u8>,
        /// `<globallightintensity>`: `undetermined`, `sunny`, `half-shadow`, `shadow` or
        /// `no-light`.
        pub global_light_intensity: Option<String>,
        /// `<maximumdepth>`, metres.
        pub maximum_depth: Option<f64>,
        /// `<maximumvisibility>`, metres.
        pub maximum_visibility: Option<f64>,
        /// `<minimumdepth>`, metres.
        pub minimum_depth: Option<f64>,
        /// `<minimumvisibility>`, metres.
        pub minimum_visibility: Option<f64>,
        /// `<terrain>`.
        pub terrain: Option<String>,
    }
}

record! {
    /// `<trip>`.
    pub struct Trip {
        /// `@id`.
        pub id: Option<String>,
        /// `<name>`.
        pub name: Option<String>,
        /// `<aliasname>`.
        pub alias_names: Vec<String>,
        /// `<rating>`.
        pub ratings: Vec<Rating>,
        /// `<trippart>`: stretches of the trip, in file order.
        pub parts: Vec<TripPart>,
    }
}

vocabulary! {
    /// `<trippart @type>`.
    TripPartType {
        Boat = "boat",
        Hotel = "hotel",
        Individual = "individual",
        Organized = "organized",
    }
}

record! {
    /// `<trippart>`.
    pub struct TripPart {
        /// `@type`.
        pub kind: Option<TripPartType>,
        /// `<name>`.
        pub name: Option<String>,
        /// `<aliasname>`.
        pub alias_names: Vec<String>,
        /// `<dateoftrip @startdate>`.
        pub start_date: Option<DateTime>,
        /// `<dateoftrip @enddate>`.
        pub end_date: Option<DateTime>,
        /// `<geography>`.
        pub geography: Option<Geography>,
        /// `<accomodation>` (the schema's spelling; `<accommodation>` is read too).
        pub accommodation: Option<Accommodation>,
        /// `<operator>`: a liveaboard's operator.
        pub operator: Option<Operator>,
        /// `<vessel>`: a liveaboard's boat.
        pub vessel: Option<Vessel>,
        /// `<link ref>`: typically the dive base that ran the dives.
        pub link: Option<String>,
        /// `<relateddives>/<link ref>`: dives made during this part.
        pub related_dives: Vec<String>,
        /// `<pricedivepackage>`.
        pub price_dive_package: Option<DivePackagePrice>,
        /// `<priceperdive>`.
        pub price_per_dive: Option<Price>,
        /// `<rating>`.
        pub ratings: Vec<Rating>,
        /// `<notes>`.
        pub notes: Vec<NoteItem>,
    }
}

record! {
    /// `<accomodation>`: where the diver stayed.
    pub struct Accommodation {
        /// `@id`.
        pub id: Option<String>,
        /// `<name>`.
        pub name: Option<String>,
        /// `<aliasname>`.
        pub alias_names: Vec<String>,
        /// `<category>`.
        pub category: Option<String>,
        /// `<address>`.
        pub address: Option<Address>,
        /// `<contact>`.
        pub contact: Option<Contact>,
        /// `<rating>`.
        pub ratings: Vec<Rating>,
        /// `<notes>`.
        pub notes: Vec<NoteItem>,
    }
}

record! {
    /// `<operator>` of a liveaboard. It has no id.
    pub struct Operator {
        /// `<name>`.
        pub name: Option<String>,
        /// `<aliasname>`.
        pub alias_names: Vec<String>,
        /// `<address>`.
        pub address: Option<Address>,
        /// `<contact>`.
        pub contact: Option<Contact>,
        /// `<rating>`.
        pub ratings: Vec<Rating>,
        /// `<notes>`.
        pub notes: Vec<NoteItem>,
    }
}

record! {
    /// `<vessel>`: a liveaboard's boat. Its `<shipdimension>` is not modelled.
    pub struct Vessel {
        /// `@id`.
        pub id: Option<String>,
        /// `<name>`.
        pub name: Option<String>,
        /// `<aliasname>`.
        pub alias_names: Vec<String>,
        /// `<shiptype>`.
        pub ship_type: Option<String>,
        /// `<marina>`.
        pub marina: Option<String>,
        /// `<address>`.
        pub address: Option<Address>,
        /// `<contact>`.
        pub contact: Option<Contact>,
        /// `<rating>`.
        pub ratings: Vec<Rating>,
        /// `<notes>`.
        pub notes: Vec<NoteItem>,
    }
}
