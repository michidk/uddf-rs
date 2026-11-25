//! Resolving `<link ref>` and other references to the elements whose `id` they name.

use std::collections::HashMap;

use crate::{
    Accommodation, Buddy, DecoModel, Dive, DiveBase, EquipmentKind, EquipmentPiece, Guide, Mix,
    Person, Rebreather, RepetitionGroup, Shop, Site, Suit, Tank, Trip, Uddf, Vessel,
};

/// An element a reference can name.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum Target<'a> {
    /// `<owner>`.
    Owner(&'a Person),
    /// `<buddy>`.
    Buddy(&'a Buddy),
    /// A `<guide>` of a `<divebase>`; its `link` names the buddy.
    Guide {
        /// The guide.
        guide: &'a Guide,
        /// The dive base it guides for.
        dive_base: &'a DiveBase,
    },
    /// A plain equipment piece of the owner or a buddy, or a rebreather's O₂ sensor.
    Piece {
        /// Which element it is.
        kind: EquipmentKind,
        /// The piece.
        piece: &'a EquipmentPiece,
    },
    /// `<tank>`.
    Tank(&'a Tank),
    /// `<suit>`.
    Suit(&'a Suit),
    /// `<rebreather>`.
    Rebreather(&'a Rebreather),
    /// `<business>/<shop>`.
    Shop(&'a Shop),
    /// `<divebase>`.
    DiveBase(&'a DiveBase),
    /// `<site>`.
    Site(&'a Site),
    /// `<trip>`.
    Trip(&'a Trip),
    /// `<accomodation>` of a trip part.
    Accommodation(&'a Accommodation),
    /// `<vessel>` of a trip part.
    Vessel(&'a Vessel),
    /// `<mix>`.
    Mix(&'a Mix),
    /// A `<decomodel>` child.
    DecoModel(&'a DecoModel),
    /// `<repetitiongroup>`.
    RepetitionGroup(&'a RepetitionGroup),
    /// `<dive>`.
    Dive(&'a Dive),
}

/// An index from `id` to element, built by [`Uddf::references`].
///
/// UDDF ids should be unique, but real files repeat them (Subsurface gives every dive its
/// repetition group's id). The first element in document order wins; later ones are listed
/// by [`References::duplicates`]. Manufacturer ids, which files repeat on purpose, are not
/// indexed.
#[derive(Debug, Clone, Default)]
pub struct References<'a> {
    map: HashMap<&'a str, Target<'a>>,
    duplicates: Vec<(&'a str, Target<'a>)>,
}

impl<'a> References<'a> {
    /// The element `reference` names, ignoring surrounding whitespace.
    #[must_use]
    pub fn get(&self, reference: &str) -> Option<Target<'a>> {
        self.map.get(reference.trim()).copied()
    }

    /// Elements whose id an earlier element already has, in document order.
    #[must_use]
    pub fn duplicates(&self) -> &[(&'a str, Target<'a>)] {
        &self.duplicates
    }

    fn add(&mut self, id: Option<&'a String>, target: Target<'a>) {
        let Some(id) = id else { return };
        if self.map.contains_key(id.as_str()) {
            self.duplicates.push((id, target));
        } else {
            self.map.insert(id, target);
        }
    }

    fn person(&mut self, person: &'a Person) {
        for (kind, piece) in person.equipment.pieces() {
            match kind {
                EquipmentKind::Tank | EquipmentKind::Suit | EquipmentKind::Rebreather => {}
                kind => self.add(piece.id.as_ref(), Target::Piece { kind, piece }),
            }
        }
        for tank in &person.equipment.tanks {
            self.add(tank.piece.id.as_ref(), Target::Tank(tank));
        }
        for suit in &person.equipment.suits {
            self.add(suit.piece.id.as_ref(), Target::Suit(suit));
        }
        for rebreather in &person.equipment.rebreathers {
            self.add(rebreather.piece.id.as_ref(), Target::Rebreather(rebreather));
        }
    }
}

impl Uddf {
    /// Indexes every element with an `id` that a reference can name.
    #[must_use]
    pub fn references(&self) -> References<'_> {
        let mut refs = References::default();
        for shop in &self.shops {
            refs.add(shop.id.as_ref(), Target::Shop(shop));
        }
        if let Some(owner) = &self.owner {
            refs.add(owner.id.as_ref(), Target::Owner(owner));
            refs.person(owner);
        }
        for buddy in &self.buddies {
            refs.add(buddy.person.id.as_ref(), Target::Buddy(buddy));
            refs.person(&buddy.person);
        }
        for base in &self.dive_bases {
            refs.add(base.id.as_ref(), Target::DiveBase(base));
            for guide in &base.guides {
                refs.add(
                    guide.id.as_ref(),
                    Target::Guide {
                        guide,
                        dive_base: base,
                    },
                );
            }
        }
        for site in &self.sites {
            refs.add(site.id.as_ref(), Target::Site(site));
        }
        for trip in &self.trips {
            refs.add(trip.id.as_ref(), Target::Trip(trip));
            for part in &trip.parts {
                if let Some(accommodation) = &part.accommodation {
                    refs.add(
                        accommodation.id.as_ref(),
                        Target::Accommodation(accommodation),
                    );
                }
                if let Some(vessel) = &part.vessel {
                    refs.add(vessel.id.as_ref(), Target::Vessel(vessel));
                }
            }
        }
        for mix in &self.mixes {
            refs.add(mix.id.as_ref(), Target::Mix(mix));
        }
        for model in &self.deco_models {
            let id = match model {
                DecoModel::Buehlmann(m) => m.id.as_ref(),
                DecoModel::Vpm(m) => m.id.as_ref(),
                DecoModel::Rgbm(m) => m.id.as_ref(),
            };
            refs.add(id, Target::DecoModel(model));
        }
        for group in &self.repetition_groups {
            refs.add(group.id.as_ref(), Target::RepetitionGroup(group));
            for dive in &group.dives {
                refs.add(dive.id.as_ref(), Target::Dive(dive));
            }
        }
        refs
    }
}
