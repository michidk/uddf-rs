//! The lenient reader: XML text to [`Uddf`].
//!
//! Elements and attributes are matched by their ASCII-lowercased local name, so any
//! namespace (3.2, 3.1, none) and the upper-case 2.x spelling read alike. Children are
//! taken by name in any order; unknown elements are skipped. Text is trimmed, and an
//! element holding only whitespace counts as absent. A value that cannot be read as its
//! UDDF type is an [`InvalidValue`].

use core::str::FromStr;

use roxmltree::{Document, Node};

#[allow(clippy::wildcard_imports)] // every model type is read here
use crate::model::*;
use crate::{DateTime, Error, InvalidValue};

type N<'a, 'i> = Node<'a, 'i>;
type R<T> = Result<T, Error>;

const DECIMAL: &str = "a decimal number";
const UNSIGNED: &str = "a non-negative integer";
const INTEGER: &str = "an integer";
const DATE_TIME: &str = "a date-time";

pub(crate) fn read(text: &str, lenient: bool) -> R<(Uddf, Vec<InvalidValue>)> {
    // roxmltree refuses a `<!DOCTYPE>` unless told otherwise, which is what UDDF wants.
    let document = Document::parse(text)?;
    let root = document.root_element();
    if !is(root, "uddf") {
        return Err(Error::NotUddf(root.tag_name().name().to_owned()));
    }
    let mut reader = Reader {
        lenient,
        warnings: Vec::new(),
    };
    let uddf = reader.uddf(root)?;
    Ok((uddf, reader.warnings))
}

fn is(node: N<'_, '_>, name: &str) -> bool {
    node.is_element() && node.tag_name().name().eq_ignore_ascii_case(name)
}

fn kids<'a, 'i>(node: N<'a, 'i>, name: &'static str) -> impl Iterator<Item = N<'a, 'i>> {
    node.children().filter(move |child| is(*child, name))
}

fn grandkids<'a, 'i>(
    node: N<'a, 'i>,
    name: &'static str,
    grandchild: &'static str,
) -> impl Iterator<Item = N<'a, 'i>> {
    kids(node, name).flat_map(move |child| kids(child, grandchild))
}

fn kid<'a, 'i>(node: N<'a, 'i>, name: &'static str) -> Option<N<'a, 'i>> {
    kids(node, name).next()
}

fn text(node: N<'_, '_>) -> Option<String> {
    let all: String = node.children().filter_map(|child| child.text()).collect();
    let trimmed = all.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

fn attr(node: N<'_, '_>, name: &str) -> Option<String> {
    node.attributes()
        .find(|attribute| attribute.name().eq_ignore_ascii_case(name))
        .map(|attribute| attribute.value().trim())
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn string(node: N<'_, '_>, name: &'static str) -> Option<String> {
    kid(node, name).and_then(text)
}

fn strings(node: N<'_, '_>, name: &'static str) -> Vec<String> {
    kids(node, name).filter_map(text).collect()
}

fn id(node: N<'_, '_>) -> Option<String> {
    attr(node, "id")
}

fn links(node: N<'_, '_>) -> Vec<String> {
    kids(node, "link")
        .filter_map(|link| attr(link, "ref"))
        .collect()
}

fn link(node: N<'_, '_>) -> Option<String> {
    kids(node, "link").find_map(|link| attr(link, "ref"))
}

fn vocab<T>(node: N<'_, '_>, name: &'static str, from: fn(&str) -> T) -> Option<T> {
    string(node, name).map(|value| from(&value))
}

fn address(node: N<'_, '_>) -> Address {
    Address {
        street: string(node, "street"),
        city: string(node, "city"),
        postcode: string(node, "postcode"),
        country: string(node, "country"),
        province: string(node, "province"),
    }
}

fn contact(node: N<'_, '_>) -> Contact {
    Contact {
        languages: strings(node, "language"),
        phones: strings(node, "phone"),
        mobile_phones: strings(node, "mobilephone"),
        faxes: strings(node, "fax"),
        emails: strings(node, "email"),
        homepages: strings(node, "homepage"),
    }
}

fn notes(node: N<'_, '_>) -> Vec<NoteItem> {
    kids(node, "notes")
        .flat_map(|notes| notes.children())
        .filter_map(|child| {
            if is(child, "para") {
                text(child).map(NoteItem::Para)
            } else if is(child, "link") {
                attr(child, "ref").map(NoteItem::Link)
            } else {
                None
            }
        })
        .collect()
}

/// `/uddf/profiledata/repetitiongroup/dive[2]/...`, with `/@attribute` appended if given.
fn path(node: N<'_, '_>, attribute: Option<&str>) -> String {
    let mut parts: Vec<String> = node
        .ancestors()
        .filter(Node::is_element)
        .map(|element| {
            let name = element.tag_name().name();
            let lower = name.to_ascii_lowercase();
            let Some(parent) = element.parent() else {
                return lower;
            };
            let same: Vec<N<'_, '_>> = parent
                .children()
                .filter(|sibling| {
                    sibling.is_element() && sibling.tag_name().name().eq_ignore_ascii_case(name)
                })
                .collect();
            match same.iter().position(|sibling| *sibling == element) {
                Some(index) if same.len() > 1 => format!("{lower}[{}]", index.saturating_add(1)),
                _ => lower,
            }
        })
        .collect();
    parts.reverse();
    let mut path = format!("/{}", parts.join("/"));
    if let Some(attribute) = attribute {
        path.push_str("/@");
        path.push_str(&attribute.to_ascii_lowercase());
    }
    path
}

struct Reader {
    lenient: bool,
    warnings: Vec<InvalidValue>,
}

impl Reader {
    fn invalid(
        &mut self,
        node: N<'_, '_>,
        attribute: Option<&str>,
        value: &str,
        expected: &'static str,
    ) -> R<()> {
        let start = attribute
            .and_then(|name| {
                node.attributes()
                    .find(|a| a.name().eq_ignore_ascii_case(name))
            })
            .map_or(node.range().start, |a| a.range().start);
        let pos = node.document().text_pos_at(start);
        let invalid = InvalidValue {
            path: path(node, attribute),
            line: pos.row,
            column: pos.col,
            value: value.to_owned(),
            expected,
        };
        if self.lenient {
            self.warnings.push(invalid);
            Ok(())
        } else {
            Err(invalid.into())
        }
    }

    /// The node's own text as `T`.
    fn parse<T: FromStr>(&mut self, node: N<'_, '_>, expected: &'static str) -> R<Option<T>> {
        let Some(value) = text(node) else {
            return Ok(None);
        };
        match value.parse() {
            Ok(parsed) => Ok(Some(parsed)),
            Err(_) => self.invalid(node, None, &value, expected).map(|()| None),
        }
    }

    /// The first `name` child's text as `T`.
    fn value<T: FromStr>(
        &mut self,
        node: N<'_, '_>,
        name: &'static str,
        expected: &'static str,
    ) -> R<Option<T>> {
        match kid(node, name) {
            Some(child) => self.parse(child, expected),
            None => Ok(None),
        }
    }

    fn num(&mut self, node: N<'_, '_>, name: &'static str) -> R<Option<f64>> {
        self.value(node, name, DECIMAL)
    }

    fn attr_value<T: FromStr>(
        &mut self,
        node: N<'_, '_>,
        name: &'static str,
        expected: &'static str,
    ) -> R<Option<T>> {
        let Some(value) = attr(node, name) else {
            return Ok(None);
        };
        match value.parse() {
            Ok(parsed) => Ok(Some(parsed)),
            Err(_) => self
                .invalid(node, Some(name), &value, expected)
                .map(|()| None),
        }
    }

    fn datetime(&mut self, node: N<'_, '_>, name: &'static str) -> R<Option<DateTime>> {
        self.value(node, name, DATE_TIME)
    }

    /// `<name><datetime>...</datetime></name>`, or a bare date-time in `<name>`.
    fn wrapped_datetime(&mut self, node: N<'_, '_>, name: &'static str) -> R<Option<DateTime>> {
        match kid(node, name) {
            Some(wrapper) if kid(wrapper, "datetime").is_some() => {
                self.datetime(wrapper, "datetime")
            }
            Some(wrapper) => self.parse(wrapper, DATE_TIME),
            None => Ok(None),
        }
    }

    fn list<'a, 'i: 'a, T>(
        &mut self,
        nodes: impl Iterator<Item = N<'a, 'i>>,
        mut read: impl FnMut(&mut Self, N<'a, 'i>) -> R<T>,
    ) -> R<Vec<T>> {
        nodes.map(|node| read(self, node)).collect()
    }

    fn opt<'a, 'i: 'a, T>(
        &mut self,
        node: Option<N<'a, 'i>>,
        read: impl FnOnce(&mut Self, N<'a, 'i>) -> R<T>,
    ) -> R<Option<T>> {
        node.map(|node| read(self, node)).transpose()
    }

    fn price(&mut self, node: Option<N<'_, '_>>) -> R<Option<Price>> {
        let Some(node) = node else { return Ok(None) };
        let value = self.parse(node, DECIMAL)?;
        Ok(value.map(|value| Price {
            value,
            currency: attr(node, "currency"),
        }))
    }

    fn package_price(&mut self, node: Option<N<'_, '_>>) -> R<Option<DivePackagePrice>> {
        let Some(node) = node else { return Ok(None) };
        let Some(value) = self.parse(node, DECIMAL)? else {
            return Ok(None);
        };
        Ok(Some(DivePackagePrice {
            value,
            currency: attr(node, "currency"),
            number_of_dives: self.attr_value(node, "noofdives", UNSIGNED)?,
        }))
    }

    fn uddf(&mut self, root: N<'_, '_>) -> R<Uddf> {
        let mut deco_models = Vec::new();
        for model in kids(root, "decomodel").flat_map(|node| node.children()) {
            if is(model, "buehlmann") {
                deco_models.push(DecoModel::Buehlmann(Buehlmann {
                    id: id(model),
                    gradient_factor_high: self.num(model, "gradientfactorhigh")?,
                    gradient_factor_low: self.num(model, "gradientfactorlow")?,
                }));
            } else if is(model, "vpm") {
                deco_models.push(DecoModel::Vpm(Vpm {
                    id: id(model),
                    gamma: self.num(model, "gamma")?,
                    gc: self.num(model, "gc")?,
                    lambda: self.num(model, "lambda")?,
                    r0: self.num(model, "r0")?,
                }));
            } else if is(model, "rgbm") {
                deco_models.push(DecoModel::Rgbm(Rgbm { id: id(model) }));
            }
        }
        Ok(Uddf {
            version: attr(root, "version"),
            generator: self
                .opt(kid(root, "generator"), Self::generator)?
                .unwrap_or_default(),
            makers: self.list(grandkids(root, "maker", "manufacturer"), Self::manufacturer)?,
            shops: self.list(grandkids(root, "business", "shop"), Self::shop)?,
            owner: self.opt(
                kids(root, "diver").find_map(|diver| kid(diver, "owner")),
                Self::person,
            )?,
            buddies: self.list(grandkids(root, "diver", "buddy"), Self::buddy)?,
            dive_bases: self.list(grandkids(root, "divesite", "divebase"), Self::dive_base)?,
            sites: self.list(grandkids(root, "divesite", "site"), Self::site)?,
            trips: self.list(grandkids(root, "divetrip", "trip"), Self::trip)?,
            mixes: self.list(grandkids(root, "gasdefinitions", "mix"), Self::mix)?,
            deco_models,
            repetition_groups: self.list(
                grandkids(root, "profiledata", "repetitiongroup"),
                Self::repetition_group,
            )?,
        })
    }

    fn generator(&mut self, node: N<'_, '_>) -> R<Generator> {
        Ok(Generator {
            name: string(node, "name"),
            alias_names: strings(node, "aliasname"),
            kind: vocab(node, "type", GeneratorType::from_uddf),
            link: link(node),
            manufacturer: self.opt(kid(node, "manufacturer"), Self::manufacturer)?,
            version: string(node, "version"),
            datetime: self.datetime(node, "datetime")?,
        })
    }

    #[allow(clippy::unnecessary_wraps, clippy::unused_self)] // shares `list`'s signature
    fn manufacturer(&mut self, node: N<'_, '_>) -> R<Manufacturer> {
        Ok(Manufacturer {
            id: id(node),
            name: string(node, "name"),
            alias_names: strings(node, "aliasname"),
            address: kid(node, "address").map(address),
            contact: kid(node, "contact").map(contact),
        })
    }

    #[allow(clippy::unnecessary_wraps, clippy::unused_self)] // shares `list`'s signature
    fn shop(&mut self, node: N<'_, '_>) -> R<Shop> {
        Ok(Shop {
            id: id(node),
            name: string(node, "name"),
            alias_names: strings(node, "aliasname"),
            address: kid(node, "address").map(address),
            contact: kid(node, "contact").map(contact),
            notes: notes(node),
        })
    }

    fn person(&mut self, node: N<'_, '_>) -> R<Person> {
        Ok(Person {
            id: id(node),
            personal: self
                .opt(kid(node, "personal"), Self::personal)?
                .unwrap_or_default(),
            address: kid(node, "address").map(address),
            contact: kid(node, "contact").map(contact),
            equipment: self
                .opt(kid(node, "equipment"), Self::equipment)?
                .unwrap_or_default(),
            certifications: self.list(
                grandkids(node, "education", "certification"),
                Self::certification,
            )?,
            insurances: self.list(
                grandkids(node, "diveinsurances", "insurance"),
                Self::insurance,
            )?,
            notes: notes(node),
        })
    }

    fn buddy(&mut self, node: N<'_, '_>) -> R<Buddy> {
        Ok(Buddy {
            person: self.person(node)?,
            student: kid(node, "student").is_some(),
        })
    }

    fn personal(&mut self, node: N<'_, '_>) -> R<Personal> {
        Ok(Personal {
            first_name: string(node, "firstname"),
            middle_name: string(node, "middlename"),
            last_name: string(node, "lastname"),
            birth_name: string(node, "birthname"),
            honorific: string(node, "honorific"),
            sex: vocab(node, "sex", Sex::from_uddf),
            birth_date: self.wrapped_datetime(node, "birthdate")?,
            passport: string(node, "passport"),
            membership: kid(node, "membership").map(|membership| Membership {
                organisation: attr(membership, "organisation"),
                member_id: attr(membership, "memberid"),
            }),
            height: self.num(node, "height")?,
            weight: self.num(node, "weight")?,
            blood_group: string(node, "bloodgroup"),
            smoking: string(node, "smoking"),
        })
    }

    fn certification(&mut self, node: N<'_, '_>) -> R<Certification> {
        Ok(Certification {
            level: string(node, "level"),
            specialty: string(node, "specialty"),
            certificate_number: string(node, "certificatenumber"),
            organization: string(node, "organization"),
            instructor: self.opt(kid(node, "instructor"), Self::instructor)?,
            issue_date: self.wrapped_datetime(node, "issuedate")?,
            valid_date: self.wrapped_datetime(node, "validdate")?,
            notes: notes(node),
        })
    }

    fn instructor(&mut self, node: N<'_, '_>) -> R<Instructor> {
        Ok(Instructor {
            id: id(node),
            personal: self
                .opt(kid(node, "personal"), Self::personal)?
                .unwrap_or_default(),
            address: kid(node, "address").map(address),
            contact: kid(node, "contact").map(contact),
            notes: notes(node),
        })
    }

    fn insurance(&mut self, node: N<'_, '_>) -> R<Insurance> {
        Ok(Insurance {
            name: string(node, "name"),
            alias_names: strings(node, "aliasname"),
            issue_date: self.wrapped_datetime(node, "issuedate")?,
            valid_date: self.wrapped_datetime(node, "validdate")?,
            notes: notes(node),
        })
    }

    fn equipment(&mut self, node: N<'_, '_>) -> R<Equipment> {
        let mut equipment = Equipment::default();
        for child in node.children().filter(Node::is_element) {
            let name = child.tag_name().name().to_ascii_lowercase();
            let list = match name.as_str() {
                "boots" => &mut equipment.boots,
                "buoyancycontroldevice" => &mut equipment.buoyancy_control_devices,
                "camera" => &mut equipment.cameras,
                "compass" => &mut equipment.compasses,
                "compressor" => &mut equipment.compressors,
                "divecomputer" => &mut equipment.dive_computers,
                "fins" => &mut equipment.fins,
                "gloves" => &mut equipment.gloves,
                "knife" => &mut equipment.knives,
                "lead" => &mut equipment.leads,
                "light" => &mut equipment.lights,
                "mask" => &mut equipment.masks,
                "regulator" => &mut equipment.regulators,
                "scooter" => &mut equipment.scooters,
                "variouspieces" => &mut equipment.various_pieces,
                "videocamera" => &mut equipment.video_cameras,
                "watch" => &mut equipment.watches,
                "rebreather" => {
                    let rebreather = Rebreather {
                        piece: self.piece(child)?,
                        o2_sensors: self.list(kids(child, "o2sensor"), Self::piece)?,
                    };
                    equipment.rebreathers.push(rebreather);
                    continue;
                }
                "suit" => {
                    let suit = Suit {
                        piece: self.piece(child)?,
                        suit_type: vocab(child, "suittype", SuitType::from_uddf),
                    };
                    equipment.suits.push(suit);
                    continue;
                }
                "tank" => {
                    let tank = Tank {
                        piece: self.piece(child)?,
                        material: vocab(child, "tankmaterial", TankMaterial::from_uddf),
                        volume: self.num(child, "tankvolume")?,
                    };
                    equipment.tanks.push(tank);
                    continue;
                }
                _ => continue,
            };
            list.push(self.piece(child)?);
        }
        Ok(equipment)
    }

    fn piece(&mut self, node: N<'_, '_>) -> R<EquipmentPiece> {
        Ok(EquipmentPiece {
            id: id(node),
            name: string(node, "name"),
            alias_names: strings(node, "aliasname"),
            link: link(node),
            manufacturer: self.opt(kid(node, "manufacturer"), Self::manufacturer)?,
            model: string(node, "model"),
            serial_number: string(node, "serialnumber"),
            purchase: self.opt(kid(node, "purchase"), Self::purchase)?,
            service_interval: self.value(node, "serviceinterval", UNSIGNED)?,
            next_service_date: self.wrapped_datetime(node, "nextservicedate")?,
            notes: notes(node),
        })
    }

    fn purchase(&mut self, node: N<'_, '_>) -> R<Purchase> {
        Ok(Purchase {
            datetime: self.datetime(node, "datetime")?,
            price: self.price(kid(node, "price"))?,
            shop: self.opt(kid(node, "shop"), Self::shop)?,
        })
    }

    fn rating(&mut self, node: N<'_, '_>) -> R<Rating> {
        Ok(Rating {
            datetime: self.datetime(node, "datetime")?,
            value: self.value(node, "ratingvalue", "an integer from 1 to 10")?,
        })
    }

    fn ratings(&mut self, node: N<'_, '_>) -> R<Vec<Rating>> {
        self.list(kids(node, "rating"), Self::rating)
    }

    fn dive_base(&mut self, node: N<'_, '_>) -> R<DiveBase> {
        Ok(DiveBase {
            id: id(node),
            name: string(node, "name"),
            alias_names: strings(node, "aliasname"),
            address: kid(node, "address").map(address),
            contact: kid(node, "contact").map(contact),
            price_per_dive: self.price(kid(node, "priceperdive"))?,
            price_dive_package: self.package_price(kid(node, "pricedivepackage"))?,
            guides: kids(node, "guide")
                .map(|guide| Guide {
                    id: id(guide),
                    link: link(guide),
                })
                .collect(),
            ratings: self.ratings(node)?,
            link: link(node),
            notes: notes(node),
        })
    }

    fn site(&mut self, node: N<'_, '_>) -> R<Site> {
        Ok(Site {
            id: id(node),
            name: string(node, "name"),
            alias_names: strings(node, "aliasname"),
            environment: vocab(node, "environment", Environment::from_uddf),
            geography: self.opt(kid(node, "geography"), Self::geography)?,
            site_data: self.opt(kid(node, "sitedata"), Self::site_data)?,
            ratings: self.ratings(node)?,
            notes: notes(node),
        })
    }

    fn geography(&mut self, node: N<'_, '_>) -> R<Geography> {
        Ok(Geography {
            location: string(node, "location"),
            address: kid(node, "address").map(address),
            latitude: self.num(node, "latitude")?,
            longitude: self.num(node, "longitude")?,
            altitude: self.num(node, "altitude")?,
            timezone: self.num(node, "timezone")?,
        })
    }

    fn site_data(&mut self, node: N<'_, '_>) -> R<SiteData> {
        Ok(SiteData {
            area_length: self.num(node, "arealength")?,
            area_width: self.num(node, "areawidth")?,
            average_visibility: self.num(node, "averagevisibility")?,
            bottom: string(node, "bottom"),
            density: self.num(node, "density")?,
            difficulty: self.value(node, "difficulty", "an integer from 1 to 10")?,
            global_light_intensity: string(node, "globallightintensity"),
            maximum_depth: self.num(node, "maximumdepth")?,
            maximum_visibility: self.num(node, "maximumvisibility")?,
            minimum_depth: self.num(node, "minimumdepth")?,
            minimum_visibility: self.num(node, "minimumvisibility")?,
            terrain: string(node, "terrain"),
        })
    }

    fn trip(&mut self, node: N<'_, '_>) -> R<Trip> {
        Ok(Trip {
            id: id(node),
            name: string(node, "name"),
            alias_names: strings(node, "aliasname"),
            ratings: self.ratings(node)?,
            parts: self.list(kids(node, "trippart"), Self::trip_part)?,
        })
    }

    fn trip_part(&mut self, node: N<'_, '_>) -> R<TripPart> {
        let dates = kid(node, "dateoftrip");
        let accommodation = kid(node, "accomodation").or_else(|| kid(node, "accommodation"));
        Ok(TripPart {
            kind: attr(node, "type").map(|kind| TripPartType::from_uddf(&kind)),
            name: string(node, "name"),
            alias_names: strings(node, "aliasname"),
            start_date: self
                .opt(dates, |r, d| r.attr_value(d, "startdate", DATE_TIME))?
                .flatten(),
            end_date: self
                .opt(dates, |r, d| r.attr_value(d, "enddate", DATE_TIME))?
                .flatten(),
            geography: self.opt(kid(node, "geography"), Self::geography)?,
            accommodation: self.opt(accommodation, Self::accommodation)?,
            operator: self.opt(kid(node, "operator"), Self::operator)?,
            vessel: self.opt(kid(node, "vessel"), Self::vessel)?,
            link: link(node),
            related_dives: kid(node, "relateddives").map(links).unwrap_or_default(),
            price_dive_package: self.package_price(kid(node, "pricedivepackage"))?,
            price_per_dive: self.price(kid(node, "priceperdive"))?,
            ratings: self.ratings(node)?,
            notes: notes(node),
        })
    }

    fn accommodation(&mut self, node: N<'_, '_>) -> R<Accommodation> {
        Ok(Accommodation {
            id: id(node),
            name: string(node, "name"),
            alias_names: strings(node, "aliasname"),
            category: string(node, "category"),
            address: kid(node, "address").map(address),
            contact: kid(node, "contact").map(contact),
            ratings: self.ratings(node)?,
            notes: notes(node),
        })
    }

    fn operator(&mut self, node: N<'_, '_>) -> R<Operator> {
        Ok(Operator {
            name: string(node, "name"),
            alias_names: strings(node, "aliasname"),
            address: kid(node, "address").map(address),
            contact: kid(node, "contact").map(contact),
            ratings: self.ratings(node)?,
            notes: notes(node),
        })
    }

    fn vessel(&mut self, node: N<'_, '_>) -> R<Vessel> {
        Ok(Vessel {
            id: id(node),
            name: string(node, "name"),
            alias_names: strings(node, "aliasname"),
            ship_type: string(node, "shiptype"),
            marina: string(node, "marina"),
            address: kid(node, "address").map(address),
            contact: kid(node, "contact").map(contact),
            ratings: self.ratings(node)?,
            notes: notes(node),
        })
    }

    fn mix(&mut self, node: N<'_, '_>) -> R<Mix> {
        Ok(Mix {
            id: id(node),
            name: string(node, "name"),
            alias_names: strings(node, "aliasname"),
            o2: self.num(node, "o2")?,
            n2: self.num(node, "n2")?,
            he: self.num(node, "he")?,
            ar: self.num(node, "ar")?,
            h2: self.num(node, "h2")?,
            price_per_litre: self.price(kid(node, "priceperlitre"))?,
            maximum_po2: self.num(node, "maximumpo2")?,
            maximum_operation_depth: self.num(node, "maximumoperationdepth")?,
            equivalent_air_depth: self.num(node, "equivalentairdepth")?,
        })
    }

    fn repetition_group(&mut self, node: N<'_, '_>) -> R<RepetitionGroup> {
        Ok(RepetitionGroup {
            id: id(node),
            dives: self.list(kids(node, "dive"), Self::dive)?,
        })
    }

    fn dive(&mut self, node: N<'_, '_>) -> R<Dive> {
        Ok(Dive {
            id: id(node),
            information_before_dive: self
                .opt(kid(node, "informationbeforedive"), Self::before)?
                .unwrap_or_default(),
            tank_data: self.list(kids(node, "tankdata"), Self::tank_data)?,
            samples: self.list(grandkids(node, "samples", "waypoint"), Self::waypoint)?,
            information_after_dive: self
                .opt(kid(node, "informationafterdive"), Self::after)?
                .unwrap_or_default(),
        })
    }

    fn surface_interval(&mut self, node: Option<N<'_, '_>>) -> R<Option<SurfaceInterval>> {
        let Some(node) = node else { return Ok(None) };
        if kid(node, "infinity").is_some() || kid(node, "inifinity").is_some() {
            return Ok(Some(SurfaceInterval::Infinity));
        }
        Ok(self
            .num(node, "passedtime")?
            .map(SurfaceInterval::PassedTime))
    }

    fn before(&mut self, node: N<'_, '_>) -> R<InformationBeforeDive> {
        Ok(InformationBeforeDive {
            links: links(node),
            dive_number: self.value(node, "divenumber", UNSIGNED)?,
            dive_number_of_day: self.value(node, "divenumberofday", UNSIGNED)?,
            internal_dive_number: self.value(node, "internaldivenumber", UNSIGNED)?,
            datetime: self.datetime(node, "datetime")?,
            air_temperature: self.num(node, "airtemperature")?,
            surface_interval_before_dive: self
                .surface_interval(kid(node, "surfaceintervalbeforedive"))?,
            altitude: self.num(node, "altitude")?,
            equipment_used: self.opt(kid(node, "equipmentused"), |r, used| {
                Ok(EquipmentUsed {
                    lead_quantity: r.num(used, "leadquantity")?,
                    links: links(used),
                })
            })?,
            apparatus: vocab(node, "apparatus", Apparatus::from_uddf),
            platform: vocab(node, "platform", Platform::from_uddf),
            purpose: vocab(node, "purpose", Purpose::from_uddf),
            state_of_rest_before_dive: vocab(node, "stateofrestbeforedive", StateOfRest::from_uddf),
            trip_membership: kid(node, "tripmembership").and_then(|trip| attr(trip, "ref")),
            no_suit: kid(node, "nosuit").is_some(),
            price: self.price(kid(node, "price"))?,
            surface_pressure: self.num(node, "surfacepressure")?,
        })
    }

    fn after(&mut self, node: N<'_, '_>) -> R<InformationAfterDive> {
        Ok(InformationAfterDive {
            surface_interval_after_dive: self
                .surface_interval(kid(node, "surfaceintervalafterdive"))?,
            lowest_temperature: self.num(node, "lowesttemperature")?,
            greatest_depth: self.num(node, "greatestdepth")?,
            visibility: self.num(node, "visibility")?,
            current: vocab(node, "current", Current::from_uddf),
            dive_plan: vocab(node, "diveplan", DivePlan::from_uddf),
            equipment_malfunction: vocab(
                node,
                "equipmentmalfunction",
                EquipmentMalfunction::from_uddf,
            ),
            pressure_drop: self.num(node, "pressuredrop")?,
            problems: vocab(node, "problems", Problems::from_uddf),
            program: vocab(node, "program", Program::from_uddf),
            thermal_comfort: vocab(node, "thermalcomfort", ThermalComfort::from_uddf),
            workload: vocab(node, "workload", Workload::from_uddf),
            desaturation_time: self.num(node, "desaturationtime")?,
            no_flight_time: self.num(node, "noflighttime")?,
            notes: notes(node),
            rating: self.opt(kid(node, "rating"), Self::rating)?,
            dive_duration: self.num(node, "diveduration")?,
            dive_table: vocab(node, "divetable", DiveTable::from_uddf),
            global_alarms_given: grandkids(node, "globalalarmsgiven", "globalalarm")
                .filter_map(text)
                .map(|alarm| GlobalAlarm::from_uddf(&alarm))
                .collect(),
            highest_po2: self.num(node, "highestpo2")?,
            observation_notes: kid(node, "observations").map(notes).unwrap_or_default(),
            average_depth: self.num(node, "averagedepth")?,
        })
    }

    fn tank_data(&mut self, node: N<'_, '_>) -> R<TankData> {
        Ok(TankData {
            links: links(node),
            tank_volume: self.num(node, "tankvolume")?,
            tank_pressure_begin: self.num(node, "tankpressurebegin")?,
            tank_pressure_end: self.num(node, "tankpressureend")?,
            breathing_consumption_volume: self.num(node, "breathingconsumptionvolume")?,
        })
    }

    fn waypoint(&mut self, node: N<'_, '_>) -> R<Waypoint> {
        let mut waypoint = Waypoint {
            cns: self.num(node, "cns")?,
            body_temperature: self.num(node, "bodytemperature")?,
            calculated_po2: self.num(node, "calculatedpo2")?,
            depth: self.num(node, "depth")?,
            dive_time: self.num(node, "divetime")?,
            heading: self.num(node, "heading")?,
            otu: self.num(node, "otu")?,
            pulse_rate: self.num(node, "pulserate")?,
            heart_rate: self.num(node, "heartrate")?,
            remaining_bottom_time: self.num(node, "remainingbottomtime")?,
            remaining_o2_time: self.num(node, "remainingo2time")?,
            set_marker: string(node, "setmarker"),
            switch_mix: kid(node, "switchmix").and_then(|mix| attr(mix, "ref")),
            temperature: self.num(node, "temperature")?,
            dive_mode: kid(node, "divemode").map(|mode| {
                attr(mode, "type").map_or(DiveMode::Unspecified, |kind| DiveMode::from_uddf(&kind))
            }),
            no_deco_time: self.num(node, "nodecotime")?,
            ..Waypoint::default()
        };
        for alarm in kids(node, "alarm") {
            let level = self.attr_value(alarm, "level", DECIMAL)?;
            waypoint.alarms.push(Alarm {
                kind: text(alarm).map(|kind| AlarmType::from_uddf(&kind)),
                level,
            });
        }
        for battery in kids(node, "batterychargecondition") {
            if let Some(value) = self.parse(battery, DECIMAL)? {
                waypoint
                    .battery_charge_conditions
                    .push(BatteryChargeCondition {
                        value,
                        device_ref: attr(battery, "deviceref"),
                        tank_ref: attr(battery, "tankref"),
                    });
            }
        }
        for stop in kids(node, "decostop") {
            waypoint.deco_stops.push(DecoStop {
                kind: attr(stop, "kind").map(|kind| DecoStopKind::from_uddf(&kind)),
                depth: self.attr_value(stop, "decodepth", DECIMAL)?,
                duration: self.attr_value(stop, "duration", DECIMAL)?,
            });
        }
        if let Some(set) = kid(node, "setpo2") {
            if let Some(value) = self.parse(set, DECIMAL)? {
                let set_by = attr(set, "setby").map(|by| SetBy::from_uddf(&by));
                waypoint.set_po2 = Some(SetPo2 { value, set_by });
            }
        }
        for pressure in kids(node, "tankpressure") {
            if let Some(value) = self.parse(pressure, DECIMAL)? {
                waypoint.tank_pressures.push(TankPressure {
                    value,
                    tank_ref: attr(pressure, "ref"),
                });
            }
        }
        if let Some(factor) = kid(node, "gradientfactor") {
            if let Some(value) = self.parse(factor, DECIMAL)? {
                let tissue = self.attr_value(factor, "tissue", INTEGER)?;
                waypoint.gradient_factor = Some(GradientFactor { value, tissue });
            }
        }
        for measured in kids(node, "measuredpo2") {
            if let Some(value) = self.parse(measured, DECIMAL)? {
                waypoint.measured_po2.push(MeasuredPo2 {
                    value,
                    sensor_ref: attr(measured, "ref"),
                });
            }
        }
        Ok(waypoint)
    }
}
