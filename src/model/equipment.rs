//! `<equipment>`: a person's kit.

use super::{Manufacturer, NoteItem, Price, Shop};
use crate::DateTime;

record! {
    /// `<equipment>`, one list per UDDF element, each in file order.
    ///
    /// The schema orders pieces by element name, so the order *between* kinds is not
    /// modelled. [`Equipment::pieces`] walks every piece.
    pub struct Equipment {
        /// `<boots>`.
        pub boots: Vec<EquipmentPiece>,
        /// `<buoyancycontroldevice>`.
        pub buoyancy_control_devices: Vec<EquipmentPiece>,
        /// `<camera>`. The 3.2.2 schema gives a camera named parts (`<body>`, `<lens>`, ...)
        /// rather than a name; those parts are not modelled, and a camera is read and
        /// written like any other piece.
        pub cameras: Vec<EquipmentPiece>,
        /// `<compass>`.
        pub compasses: Vec<EquipmentPiece>,
        /// `<compressor>`.
        pub compressors: Vec<EquipmentPiece>,
        /// `<divecomputer>`.
        pub dive_computers: Vec<EquipmentPiece>,
        /// `<fins>`.
        pub fins: Vec<EquipmentPiece>,
        /// `<gloves>`.
        pub gloves: Vec<EquipmentPiece>,
        /// `<knife>`.
        pub knives: Vec<EquipmentPiece>,
        /// `<lead>`.
        pub leads: Vec<EquipmentPiece>,
        /// `<light>`.
        pub lights: Vec<EquipmentPiece>,
        /// `<mask>`.
        pub masks: Vec<EquipmentPiece>,
        /// `<rebreather>`.
        pub rebreathers: Vec<Rebreather>,
        /// `<regulator>`.
        pub regulators: Vec<EquipmentPiece>,
        /// `<scooter>`.
        pub scooters: Vec<EquipmentPiece>,
        /// `<suit>`.
        pub suits: Vec<Suit>,
        /// `<tank>`.
        pub tanks: Vec<Tank>,
        /// `<variouspieces>`: UDDF's catch-all.
        pub various_pieces: Vec<EquipmentPiece>,
        /// `<videocamera>`, modelled like [`cameras`](Self::cameras).
        pub video_cameras: Vec<EquipmentPiece>,
        /// `<watch>`.
        pub watches: Vec<EquipmentPiece>,
    }
}

/// Which UDDF element a piece of equipment is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum EquipmentKind {
    Boots,
    BuoyancyControlDevice,
    Camera,
    Compass,
    Compressor,
    DiveComputer,
    Fins,
    Gloves,
    Knife,
    Lead,
    Light,
    Mask,
    Rebreather,
    /// An `<o2sensor>` inside a `<rebreather>`.
    O2Sensor,
    Regulator,
    Scooter,
    Suit,
    Tank,
    VariousPieces,
    VideoCamera,
    Watch,
}

impl EquipmentKind {
    /// The UDDF element name.
    #[must_use]
    pub fn element(self) -> &'static str {
        match self {
            Self::Boots => "boots",
            Self::BuoyancyControlDevice => "buoyancycontroldevice",
            Self::Camera => "camera",
            Self::Compass => "compass",
            Self::Compressor => "compressor",
            Self::DiveComputer => "divecomputer",
            Self::Fins => "fins",
            Self::Gloves => "gloves",
            Self::Knife => "knife",
            Self::Lead => "lead",
            Self::Light => "light",
            Self::Mask => "mask",
            Self::Rebreather => "rebreather",
            Self::O2Sensor => "o2sensor",
            Self::Regulator => "regulator",
            Self::Scooter => "scooter",
            Self::Suit => "suit",
            Self::Tank => "tank",
            Self::VariousPieces => "variouspieces",
            Self::VideoCamera => "videocamera",
            Self::Watch => "watch",
        }
    }
}

impl Equipment {
    /// The plain-piece lists with their kind, in schema order.
    pub(crate) fn plain(&self) -> [(EquipmentKind, &[EquipmentPiece]); 17] {
        use EquipmentKind as K;
        [
            (K::Boots, &self.boots),
            (K::BuoyancyControlDevice, &self.buoyancy_control_devices),
            (K::Camera, &self.cameras),
            (K::Compass, &self.compasses),
            (K::Compressor, &self.compressors),
            (K::DiveComputer, &self.dive_computers),
            (K::Fins, &self.fins),
            (K::Gloves, &self.gloves),
            (K::Knife, &self.knives),
            (K::Lead, &self.leads),
            (K::Light, &self.lights),
            (K::Mask, &self.masks),
            (K::Regulator, &self.regulators),
            (K::Scooter, &self.scooters),
            (K::VariousPieces, &self.various_pieces),
            (K::VideoCamera, &self.video_cameras),
            (K::Watch, &self.watches),
        ]
    }

    /// Every piece with its kind: the plain lists, then each rebreather and its O₂
    /// sensors, each suit and each tank (their common part).
    pub fn pieces(&self) -> impl Iterator<Item = (EquipmentKind, &EquipmentPiece)> {
        let plain = self
            .plain()
            .into_iter()
            .flat_map(|(kind, list)| list.iter().map(move |piece| (kind, piece)));
        let rebreathers = self.rebreathers.iter().flat_map(|rebreather| {
            core::iter::once((EquipmentKind::Rebreather, &rebreather.piece)).chain(
                rebreather
                    .o2_sensors
                    .iter()
                    .map(|sensor| (EquipmentKind::O2Sensor, sensor)),
            )
        });
        let suits = self
            .suits
            .iter()
            .map(|suit| (EquipmentKind::Suit, &suit.piece));
        let tanks = self
            .tanks
            .iter()
            .map(|tank| (EquipmentKind::Tank, &tank.piece));
        plain.chain(rebreathers).chain(suits).chain(tanks)
    }

    /// Whether no piece is recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pieces().next().is_none()
    }
}

record! {
    /// One piece of equipment (UDDF's `equipmentPieceType`).
    pub struct EquipmentPiece {
        /// `@id`.
        pub id: Option<String>,
        /// `<name>`.
        pub name: Option<String>,
        /// `<aliasname>`.
        pub alias_names: Vec<String>,
        /// `<link ref>`, typically to a `<mediadata>` image.
        pub link: Option<String>,
        /// `<manufacturer>`.
        pub manufacturer: Option<Manufacturer>,
        /// `<model>`.
        pub model: Option<String>,
        /// `<serialnumber>`.
        pub serial_number: Option<String>,
        /// `<purchase>`.
        pub purchase: Option<Purchase>,
        /// `<serviceinterval>`, days.
        pub service_interval: Option<u32>,
        /// `<nextservicedate>/<datetime>`.
        pub next_service_date: Option<DateTime>,
        /// `<notes>`.
        pub notes: Vec<NoteItem>,
    }
}

record! {
    /// `<purchase>`.
    pub struct Purchase {
        /// `<datetime>`.
        pub datetime: Option<DateTime>,
        /// `<price>`.
        pub price: Option<Price>,
        /// `<shop>`: where it was bought, inline.
        pub shop: Option<Shop>,
    }
}

vocabulary! {
    /// `<suittype>`.
    SuitType {
        DiveSkin = "dive-skin",
        WetSuit = "wet-suit",
        DrySuit = "dry-suit",
        HotWaterSuit = "hot-water-suit",
        Other = "other",
    }
}

record! {
    /// `<suit>`.
    pub struct Suit {
        /// The common part.
        #[cfg_attr(feature = "serde", serde(flatten))]
        pub piece: EquipmentPiece,
        /// `<suittype>`.
        pub suit_type: Option<SuitType>,
    }
}

vocabulary! {
    /// `<tankmaterial>`.
    TankMaterial {
        Aluminium = "aluminium",
        Carbon = "carbon",
        Steel = "steel",
    }
}

record! {
    /// `<tank>`: a cylinder the diver owns.
    pub struct Tank {
        /// The common part.
        #[cfg_attr(feature = "serde", serde(flatten))]
        pub piece: EquipmentPiece,
        /// `<tankmaterial>`.
        pub material: Option<TankMaterial>,
        /// `<tankvolume>`, cubic metres of water capacity per the specification. Some
        /// writers (older Subsurface) write litres; the value is kept as written.
        pub volume: Option<f64>,
    }
}

record! {
    /// `<rebreather>`.
    pub struct Rebreather {
        /// The common part.
        #[cfg_attr(feature = "serde", serde(flatten))]
        pub piece: EquipmentPiece,
        /// `<o2sensor>`.
        pub o2_sensors: Vec<EquipmentPiece>,
    }
}
