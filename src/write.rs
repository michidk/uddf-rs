//! The writer: [`Uddf`] to UDDF 3.2.3 XML in schema order.
//!
//! Output is indented by two spaces, carries the UDDF 3.2 namespace and depends only on
//! the model. The writer never invents values: a required element the model lacks is
//! written empty when it is text (`<name/>`, `<firstname/>`, `<country/>`,
//! `<location/>`), and left out when it is a number or a date.

use std::io::Write;

use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};

#[allow(clippy::wildcard_imports)] // every model type is written here
use crate::model::*;
use crate::{DateTime, Error};

/// The namespace every UDDF 3.2.x document uses.
pub const NAMESPACE: &str = "http://www.streit.cc/uddf/3.2/";
/// The version the writer declares.
pub const VERSION: &str = "3.2.3";

type R = Result<(), Error>;

pub(crate) fn write<W: Write>(out: W, uddf: &Uddf) -> R {
    let mut w = Xml(Writer::new_with_indent(out, b' ', 2));
    w.event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
    w.open("uddf", &[("xmlns", NAMESPACE), ("version", VERSION)])?;
    w.generator(&uddf.generator)?;
    if !uddf.makers.is_empty() {
        w.open("maker", &[])?;
        for maker in &uddf.makers {
            w.manufacturer(maker)?;
        }
        w.close("maker")?;
    }
    if !uddf.shops.is_empty() {
        w.open("business", &[])?;
        for shop in &uddf.shops {
            w.shop(shop)?;
        }
        w.close("business")?;
    }
    if uddf.owner.is_some() || !uddf.buddies.is_empty() {
        w.open("diver", &[])?;
        if let Some(owner) = &uddf.owner {
            w.person("owner", owner, false)?;
        }
        for buddy in &uddf.buddies {
            w.person("buddy", &buddy.person, buddy.student)?;
        }
        w.close("diver")?;
    }
    if !uddf.dive_bases.is_empty() || !uddf.sites.is_empty() {
        w.open("divesite", &[])?;
        for base in &uddf.dive_bases {
            w.dive_base(base)?;
        }
        for site in &uddf.sites {
            w.site(site)?;
        }
        w.close("divesite")?;
    }
    if !uddf.trips.is_empty() {
        w.open("divetrip", &[])?;
        for trip in &uddf.trips {
            w.trip(trip)?;
        }
        w.close("divetrip")?;
    }
    if !uddf.mixes.is_empty() {
        w.open("gasdefinitions", &[])?;
        for mix in &uddf.mixes {
            w.mix(mix)?;
        }
        w.close("gasdefinitions")?;
    }
    if !uddf.deco_models.is_empty() {
        w.open("decomodel", &[])?;
        for model in &uddf.deco_models {
            w.deco_model(model)?;
        }
        w.close("decomodel")?;
    }
    if !uddf.repetition_groups.is_empty() {
        w.open("profiledata", &[])?;
        for group in &uddf.repetition_groups {
            w.open("repetitiongroup", &id_attr(group.id.as_ref()))?;
            for dive in &group.dives {
                w.dive(dive)?;
            }
            w.close("repetitiongroup")?;
        }
        w.close("profiledata")?;
    }
    w.close("uddf")?;
    w.0.get_mut().write_all(b"\n")?;
    Ok(())
}

/// `xs:float`'s spelling of a number: the shortest text that reads back to the same value.
fn number(value: f64) -> String {
    if value.is_nan() {
        "NaN".to_owned()
    } else if value.is_infinite() {
        if value > 0.0 { "INF" } else { "-INF" }.to_owned()
    } else {
        value.to_string()
    }
}

fn id_attr(id: Option<&String>) -> Vec<(&'static str, &str)> {
    id.map(|id| ("id", id.as_str())).into_iter().collect()
}

struct Xml<W: Write>(Writer<W>);

impl<W: Write> Xml<W> {
    fn event(&mut self, event: Event<'_>) -> R {
        self.0.write_event(event)?;
        Ok(())
    }

    fn open(&mut self, name: &str, attrs: &[(&str, &str)]) -> R {
        self.event(Event::Start(
            BytesStart::new(name).with_attributes(attrs.iter().copied()),
        ))
    }

    fn close(&mut self, name: &str) -> R {
        self.event(Event::End(BytesEnd::new(name)))
    }

    fn empty(&mut self, name: &str, attrs: &[(&str, &str)]) -> R {
        self.event(Event::Empty(
            BytesStart::new(name).with_attributes(attrs.iter().copied()),
        ))
    }

    /// `<name attrs>text</name>`.
    fn leaf(&mut self, name: &str, attrs: &[(&str, &str)], text: &str) -> R {
        if text.is_empty() {
            return self.empty(name, attrs);
        }
        self.open(name, attrs)?;
        self.event(Event::Text(BytesText::new(text)))?;
        self.close(name)
    }

    fn text(&mut self, name: &str, text: Option<&String>) -> R {
        match text {
            Some(text) => self.leaf(name, &[], text),
            None => Ok(()),
        }
    }

    /// A text element the schema requires: written empty when absent.
    fn required(&mut self, name: &str, text: Option<&String>) -> R {
        self.leaf(name, &[], text.map_or("", String::as_str))
    }

    fn texts(&mut self, name: &str, texts: &[String]) -> R {
        texts.iter().try_for_each(|text| self.leaf(name, &[], text))
    }

    fn num(&mut self, name: &str, value: Option<f64>) -> R {
        match value {
            Some(value) => self.leaf(name, &[], &number(value)),
            None => Ok(()),
        }
    }

    fn display(&mut self, name: &str, value: Option<impl ToString>) -> R {
        match value {
            Some(value) => self.leaf(name, &[], &value.to_string()),
            None => Ok(()),
        }
    }

    fn vocab(&mut self, name: &str, value: Option<&str>) -> R {
        match value {
            Some(value) => self.leaf(name, &[], value),
            None => Ok(()),
        }
    }

    fn link(&mut self, reference: &str) -> R {
        self.empty("link", &[("ref", reference)])
    }

    fn links(&mut self, references: &[String]) -> R {
        references
            .iter()
            .try_for_each(|reference| self.link(reference))
    }

    fn opt_link(&mut self, reference: Option<&String>) -> R {
        reference.map_or(Ok(()), |reference| self.link(reference))
    }

    /// `<name><datetime>...</datetime></name>`.
    fn wrapped_datetime(&mut self, name: &str, value: Option<&DateTime>) -> R {
        let Some(value) = value else { return Ok(()) };
        self.open(name, &[])?;
        self.display("datetime", Some(value))?;
        self.close(name)
    }

    fn notes(&mut self, notes: &[NoteItem]) -> R {
        if notes.is_empty() {
            return Ok(());
        }
        self.open("notes", &[])?;
        for item in notes {
            match item {
                NoteItem::Para(text) => self.leaf("para", &[], text)?,
                NoteItem::Link(reference) => self.link(reference)?,
            }
        }
        self.close("notes")
    }

    fn names(&mut self, name: Option<&String>, alias_names: &[String]) -> R {
        self.required("name", name)?;
        self.texts("aliasname", alias_names)
    }

    fn address(&mut self, address: Option<&Address>) -> R {
        let Some(address) = address else {
            return Ok(());
        };
        self.open("address", &[])?;
        self.text("street", address.street.as_ref())?;
        self.text("city", address.city.as_ref())?;
        self.text("postcode", address.postcode.as_ref())?;
        self.required("country", address.country.as_ref())?;
        self.text("province", address.province.as_ref())?;
        self.close("address")
    }

    fn contact(&mut self, contact: Option<&Contact>) -> R {
        let Some(contact) = contact else {
            return Ok(());
        };
        self.open("contact", &[])?;
        self.texts("language", &contact.languages)?;
        self.texts("phone", &contact.phones)?;
        self.texts("mobilephone", &contact.mobile_phones)?;
        self.texts("fax", &contact.faxes)?;
        self.texts("email", &contact.emails)?;
        self.texts("homepage", &contact.homepages)?;
        self.close("contact")
    }

    fn price(&mut self, name: &str, price: Option<&Price>) -> R {
        let Some(price) = price else { return Ok(()) };
        let attrs: Vec<(&str, &str)> = price
            .currency
            .iter()
            .map(|c| ("currency", c.as_str()))
            .collect();
        self.leaf(name, &attrs, &number(price.value))
    }

    fn package_price(&mut self, price: Option<&DivePackagePrice>) -> R {
        let Some(price) = price else { return Ok(()) };
        let dives = price.number_of_dives.map(|n| n.to_string());
        let mut attrs: Vec<(&str, &str)> = price
            .currency
            .iter()
            .map(|c| ("currency", c.as_str()))
            .collect();
        attrs.extend(dives.iter().map(|n| ("noofdives", n.as_str())));
        self.leaf("pricedivepackage", &attrs, &number(price.value))
    }

    fn rating(&mut self, rating: &Rating) -> R {
        self.open("rating", &[])?;
        self.display("datetime", rating.datetime.as_ref())?;
        self.display("ratingvalue", rating.value)?;
        self.close("rating")
    }

    fn ratings(&mut self, ratings: &[Rating]) -> R {
        ratings.iter().try_for_each(|rating| self.rating(rating))
    }

    fn generator(&mut self, generator: &Generator) -> R {
        self.open("generator", &[])?;
        self.names(generator.name.as_ref(), &generator.alias_names)?;
        self.vocab("type", generator.kind.as_ref().map(GeneratorType::as_str))?;
        self.opt_link(generator.link.as_ref())?;
        if let Some(manufacturer) = &generator.manufacturer {
            self.manufacturer(manufacturer)?;
        }
        self.text("version", generator.version.as_ref())?;
        self.display("datetime", generator.datetime.as_ref())?;
        self.close("generator")
    }

    fn manufacturer(&mut self, manufacturer: &Manufacturer) -> R {
        self.open("manufacturer", &id_attr(manufacturer.id.as_ref()))?;
        self.names(manufacturer.name.as_ref(), &manufacturer.alias_names)?;
        self.address(manufacturer.address.as_ref())?;
        self.contact(manufacturer.contact.as_ref())?;
        self.close("manufacturer")
    }

    fn shop(&mut self, shop: &Shop) -> R {
        self.open("shop", &id_attr(shop.id.as_ref()))?;
        self.names(shop.name.as_ref(), &shop.alias_names)?;
        self.address(shop.address.as_ref())?;
        self.contact(shop.contact.as_ref())?;
        self.notes(&shop.notes)?;
        self.close("shop")
    }

    fn personal(&mut self, personal: &Personal) -> R {
        self.open("personal", &[])?;
        self.required("firstname", personal.first_name.as_ref())?;
        self.text("middlename", personal.middle_name.as_ref())?;
        self.required("lastname", personal.last_name.as_ref())?;
        self.text("birthname", personal.birth_name.as_ref())?;
        self.text("honorific", personal.honorific.as_ref())?;
        self.vocab("sex", personal.sex.as_ref().map(Sex::as_str))?;
        self.wrapped_datetime("birthdate", personal.birth_date.as_ref())?;
        self.text("passport", personal.passport.as_ref())?;
        if let Some(membership) = &personal.membership {
            let mut attrs = Vec::new();
            attrs.extend(
                membership
                    .organisation
                    .iter()
                    .map(|o| ("organisation", o.as_str())),
            );
            attrs.extend(
                membership
                    .member_id
                    .iter()
                    .map(|m| ("memberid", m.as_str())),
            );
            self.empty("membership", &attrs)?;
        }
        self.num("height", personal.height)?;
        self.num("weight", personal.weight)?;
        self.text("bloodgroup", personal.blood_group.as_ref())?;
        self.text("smoking", personal.smoking.as_ref())?;
        self.close("personal")
    }

    fn person(&mut self, element: &str, person: &Person, student: bool) -> R {
        self.open(element, &id_attr(person.id.as_ref()))?;
        self.personal(&person.personal)?;
        self.address(person.address.as_ref())?;
        self.contact(person.contact.as_ref())?;
        self.equipment(&person.equipment)?;
        if !person.certifications.is_empty() {
            self.open("education", &[])?;
            for certification in &person.certifications {
                self.certification(certification)?;
            }
            self.close("education")?;
        }
        if !person.insurances.is_empty() {
            self.open("diveinsurances", &[])?;
            for insurance in &person.insurances {
                self.open("insurance", &[])?;
                self.names(insurance.name.as_ref(), &insurance.alias_names)?;
                self.wrapped_datetime("issuedate", insurance.issue_date.as_ref())?;
                self.wrapped_datetime("validdate", insurance.valid_date.as_ref())?;
                self.notes(&insurance.notes)?;
                self.close("insurance")?;
            }
            self.close("diveinsurances")?;
        }
        self.notes(&person.notes)?;
        if student {
            self.empty("student", &[])?;
        }
        self.close(element)
    }

    fn certification(&mut self, certification: &Certification) -> R {
        self.open("certification", &[])?;
        self.text("level", certification.level.as_ref())?;
        self.text("specialty", certification.specialty.as_ref())?;
        self.text(
            "certificatenumber",
            certification.certificate_number.as_ref(),
        )?;
        self.text("organization", certification.organization.as_ref())?;
        if let Some(instructor) = &certification.instructor {
            self.open("instructor", &id_attr(instructor.id.as_ref()))?;
            self.personal(&instructor.personal)?;
            self.address(instructor.address.as_ref())?;
            self.contact(instructor.contact.as_ref())?;
            self.notes(&instructor.notes)?;
            self.close("instructor")?;
        }
        self.wrapped_datetime("issuedate", certification.issue_date.as_ref())?;
        self.wrapped_datetime("validdate", certification.valid_date.as_ref())?;
        self.notes(&certification.notes)?;
        self.close("certification")
    }

    fn equipment(&mut self, equipment: &Equipment) -> R {
        if equipment.is_empty() {
            return Ok(());
        }
        self.open("equipment", &[])?;
        for (kind, pieces) in equipment.plain() {
            // The schema puts these three between the plain kinds, in alphabetical order.
            if kind == EquipmentKind::Regulator {
                for rebreather in &equipment.rebreathers {
                    self.piece_open("rebreather", &rebreather.piece)?;
                    for sensor in &rebreather.o2_sensors {
                        self.piece("o2sensor", sensor)?;
                    }
                    self.close("rebreather")?;
                }
            }
            if kind == EquipmentKind::VariousPieces {
                for suit in &equipment.suits {
                    self.piece_open("suit", &suit.piece)?;
                    self.vocab("suittype", suit.suit_type.as_ref().map(SuitType::as_str))?;
                    self.close("suit")?;
                }
                for tank in &equipment.tanks {
                    self.piece_open("tank", &tank.piece)?;
                    self.vocab(
                        "tankmaterial",
                        tank.material.as_ref().map(TankMaterial::as_str),
                    )?;
                    self.num("tankvolume", tank.volume)?;
                    self.close("tank")?;
                }
            }
            for piece in pieces {
                self.piece(kind.element(), piece)?;
            }
        }
        self.close("equipment")
    }

    fn piece(&mut self, element: &str, piece: &EquipmentPiece) -> R {
        self.piece_open(element, piece)?;
        self.close(element)
    }

    /// Opens `element` and writes the common part of a piece, leaving it open.
    fn piece_open(&mut self, element: &str, piece: &EquipmentPiece) -> R {
        self.open(element, &id_attr(piece.id.as_ref()))?;
        self.names(piece.name.as_ref(), &piece.alias_names)?;
        self.opt_link(piece.link.as_ref())?;
        if let Some(manufacturer) = &piece.manufacturer {
            self.manufacturer(manufacturer)?;
        }
        self.text("model", piece.model.as_ref())?;
        self.text("serialnumber", piece.serial_number.as_ref())?;
        if let Some(purchase) = &piece.purchase {
            self.open("purchase", &[])?;
            self.display("datetime", purchase.datetime.as_ref())?;
            self.price("price", purchase.price.as_ref())?;
            if let Some(shop) = &purchase.shop {
                self.shop(shop)?;
            }
            self.close("purchase")?;
        }
        self.display("serviceinterval", piece.service_interval)?;
        self.wrapped_datetime("nextservicedate", piece.next_service_date.as_ref())?;
        self.notes(&piece.notes)
    }

    fn dive_base(&mut self, base: &DiveBase) -> R {
        self.open("divebase", &id_attr(base.id.as_ref()))?;
        self.names(base.name.as_ref(), &base.alias_names)?;
        self.address(base.address.as_ref())?;
        self.contact(base.contact.as_ref())?;
        self.price("priceperdive", base.price_per_dive.as_ref())?;
        self.package_price(base.price_dive_package.as_ref())?;
        for guide in &base.guides {
            self.open("guide", &id_attr(guide.id.as_ref()))?;
            self.opt_link(guide.link.as_ref())?;
            self.close("guide")?;
        }
        self.ratings(&base.ratings)?;
        self.opt_link(base.link.as_ref())?;
        self.notes(&base.notes)?;
        self.close("divebase")
    }

    fn geography(&mut self, geography: Option<&Geography>) -> R {
        let Some(geography) = geography else {
            return Ok(());
        };
        self.open("geography", &[])?;
        self.required("location", geography.location.as_ref())?;
        self.address(geography.address.as_ref())?;
        self.num("latitude", geography.latitude)?;
        self.num("longitude", geography.longitude)?;
        self.num("altitude", geography.altitude)?;
        self.num("timezone", geography.timezone)?;
        self.close("geography")
    }

    fn site(&mut self, site: &Site) -> R {
        self.open("site", &id_attr(site.id.as_ref()))?;
        self.names(site.name.as_ref(), &site.alias_names)?;
        self.vocab(
            "environment",
            site.environment.as_ref().map(Environment::as_str),
        )?;
        self.geography(site.geography.as_ref())?;
        if let Some(data) = &site.site_data {
            self.open("sitedata", &[])?;
            self.num("arealength", data.area_length)?;
            self.num("areawidth", data.area_width)?;
            self.num("averagevisibility", data.average_visibility)?;
            self.text("bottom", data.bottom.as_ref())?;
            self.num("density", data.density)?;
            self.display("difficulty", data.difficulty)?;
            self.text("globallightintensity", data.global_light_intensity.as_ref())?;
            self.num("maximumdepth", data.maximum_depth)?;
            self.num("maximumvisibility", data.maximum_visibility)?;
            self.num("minimumdepth", data.minimum_depth)?;
            self.num("minimumvisibility", data.minimum_visibility)?;
            self.text("terrain", data.terrain.as_ref())?;
            self.close("sitedata")?;
        }
        self.ratings(&site.ratings)?;
        self.notes(&site.notes)?;
        self.close("site")
    }

    fn trip(&mut self, trip: &Trip) -> R {
        self.open("trip", &id_attr(trip.id.as_ref()))?;
        self.names(trip.name.as_ref(), &trip.alias_names)?;
        self.ratings(&trip.ratings)?;
        for part in &trip.parts {
            self.trip_part(part)?;
        }
        self.close("trip")
    }

    fn trip_part(&mut self, part: &TripPart) -> R {
        let attrs: Vec<(&str, &str)> = part
            .kind
            .iter()
            .map(|kind| ("type", kind.as_str()))
            .collect();
        self.open("trippart", &attrs)?;
        self.names(part.name.as_ref(), &part.alias_names)?;
        if part.start_date.is_some() || part.end_date.is_some() {
            let start = part.start_date.map(|date| date.to_string());
            let end = part.end_date.map(|date| date.to_string());
            let mut attrs = Vec::new();
            attrs.extend(start.iter().map(|date| ("startdate", date.as_str())));
            attrs.extend(end.iter().map(|date| ("enddate", date.as_str())));
            self.empty("dateoftrip", &attrs)?;
        }
        self.geography(part.geography.as_ref())?;
        if let Some(stay) = &part.accommodation {
            self.open("accomodation", &id_attr(stay.id.as_ref()))?;
            self.names(stay.name.as_ref(), &stay.alias_names)?;
            self.text("category", stay.category.as_ref())?;
            self.address(stay.address.as_ref())?;
            self.contact(stay.contact.as_ref())?;
            self.ratings(&stay.ratings)?;
            self.notes(&stay.notes)?;
            self.close("accomodation")?;
        }
        if let Some(operator) = &part.operator {
            self.open("operator", &[])?;
            self.names(operator.name.as_ref(), &operator.alias_names)?;
            self.address(operator.address.as_ref())?;
            self.contact(operator.contact.as_ref())?;
            self.ratings(&operator.ratings)?;
            self.notes(&operator.notes)?;
            self.close("operator")?;
        }
        if let Some(vessel) = &part.vessel {
            self.open("vessel", &id_attr(vessel.id.as_ref()))?;
            self.names(vessel.name.as_ref(), &vessel.alias_names)?;
            self.text("shiptype", vessel.ship_type.as_ref())?;
            self.text("marina", vessel.marina.as_ref())?;
            self.address(vessel.address.as_ref())?;
            self.contact(vessel.contact.as_ref())?;
            self.ratings(&vessel.ratings)?;
            self.notes(&vessel.notes)?;
            self.close("vessel")?;
        }
        self.opt_link(part.link.as_ref())?;
        if !part.related_dives.is_empty() {
            self.open("relateddives", &[])?;
            self.links(&part.related_dives)?;
            self.close("relateddives")?;
        }
        self.package_price(part.price_dive_package.as_ref())?;
        self.price("priceperdive", part.price_per_dive.as_ref())?;
        self.ratings(&part.ratings)?;
        self.notes(&part.notes)?;
        self.close("trippart")
    }

    fn mix(&mut self, mix: &Mix) -> R {
        self.open("mix", &id_attr(mix.id.as_ref()))?;
        self.names(mix.name.as_ref(), &mix.alias_names)?;
        self.num("o2", mix.o2)?;
        self.num("n2", mix.n2)?;
        self.num("he", mix.he)?;
        self.num("ar", mix.ar)?;
        self.num("h2", mix.h2)?;
        self.price("priceperlitre", mix.price_per_litre.as_ref())?;
        self.num("maximumpo2", mix.maximum_po2)?;
        self.num("maximumoperationdepth", mix.maximum_operation_depth)?;
        self.num("equivalentairdepth", mix.equivalent_air_depth)?;
        self.close("mix")
    }

    fn deco_model(&mut self, model: &DecoModel) -> R {
        match model {
            DecoModel::Buehlmann(m) => {
                self.open("buehlmann", &id_attr(m.id.as_ref()))?;
                self.num("gradientfactorhigh", m.gradient_factor_high)?;
                self.num("gradientfactorlow", m.gradient_factor_low)?;
                self.close("buehlmann")
            }
            DecoModel::Vpm(m) => {
                self.open("vpm", &id_attr(m.id.as_ref()))?;
                self.num("gamma", m.gamma)?;
                self.num("gc", m.gc)?;
                self.num("lambda", m.lambda)?;
                self.num("r0", m.r0)?;
                self.close("vpm")
            }
            DecoModel::Rgbm(m) => self.empty("rgbm", &id_attr(m.id.as_ref())),
        }
    }

    fn surface_interval(&mut self, name: &str, interval: Option<SurfaceInterval>) -> R {
        let Some(interval) = interval else {
            return Ok(());
        };
        self.open(name, &[])?;
        match interval {
            SurfaceInterval::Infinity => self.empty("infinity", &[])?,
            SurfaceInterval::PassedTime(seconds) => self.num("passedtime", Some(seconds))?,
        }
        self.close(name)
    }

    fn dive(&mut self, dive: &Dive) -> R {
        self.open("dive", &id_attr(dive.id.as_ref()))?;
        self.before(&dive.information_before_dive)?;
        for tank in &dive.tank_data {
            self.open("tankdata", &[])?;
            self.links(&tank.links)?;
            self.num("tankvolume", tank.tank_volume)?;
            self.num("tankpressurebegin", tank.tank_pressure_begin)?;
            self.num("tankpressureend", tank.tank_pressure_end)?;
            self.num(
                "breathingconsumptionvolume",
                tank.breathing_consumption_volume,
            )?;
            self.close("tankdata")?;
        }
        if !dive.samples.is_empty() {
            self.open("samples", &[])?;
            for waypoint in &dive.samples {
                self.waypoint(waypoint)?;
            }
            self.close("samples")?;
        }
        self.after(&dive.information_after_dive)?;
        self.close("dive")
    }

    fn before(&mut self, before: &InformationBeforeDive) -> R {
        self.open("informationbeforedive", &[])?;
        self.links(&before.links)?;
        self.display("divenumber", before.dive_number)?;
        self.display("divenumberofday", before.dive_number_of_day)?;
        self.display("internaldivenumber", before.internal_dive_number)?;
        self.display("datetime", before.datetime.as_ref())?;
        self.num("airtemperature", before.air_temperature)?;
        self.surface_interval(
            "surfaceintervalbeforedive",
            before.surface_interval_before_dive,
        )?;
        self.num("altitude", before.altitude)?;
        if let Some(used) = &before.equipment_used {
            self.open("equipmentused", &[])?;
            self.num("leadquantity", used.lead_quantity)?;
            self.links(&used.links)?;
            self.close("equipmentused")?;
        }
        self.vocab(
            "apparatus",
            before.apparatus.as_ref().map(Apparatus::as_str),
        )?;
        self.vocab("platform", before.platform.as_ref().map(Platform::as_str))?;
        self.vocab("purpose", before.purpose.as_ref().map(Purpose::as_str))?;
        self.vocab(
            "stateofrestbeforedive",
            before
                .state_of_rest_before_dive
                .as_ref()
                .map(StateOfRest::as_str),
        )?;
        if let Some(trip) = &before.trip_membership {
            self.empty("tripmembership", &[("ref", trip)])?;
        }
        if before.no_suit {
            self.empty("nosuit", &[])?;
        }
        self.price("price", before.price.as_ref())?;
        self.num("surfacepressure", before.surface_pressure)?;
        self.close("informationbeforedive")
    }

    fn after(&mut self, after: &InformationAfterDive) -> R {
        self.open("informationafterdive", &[])?;
        self.surface_interval(
            "surfaceintervalafterdive",
            after.surface_interval_after_dive,
        )?;
        self.num("lowesttemperature", after.lowest_temperature)?;
        self.num("greatestdepth", after.greatest_depth)?;
        self.num("visibility", after.visibility)?;
        self.vocab("current", after.current.as_ref().map(Current::as_str))?;
        self.vocab("diveplan", after.dive_plan.as_ref().map(DivePlan::as_str))?;
        self.vocab(
            "equipmentmalfunction",
            after
                .equipment_malfunction
                .as_ref()
                .map(EquipmentMalfunction::as_str),
        )?;
        self.num("pressuredrop", after.pressure_drop)?;
        self.vocab("problems", after.problems.as_ref().map(Problems::as_str))?;
        self.vocab("program", after.program.as_ref().map(Program::as_str))?;
        self.vocab(
            "thermalcomfort",
            after.thermal_comfort.as_ref().map(ThermalComfort::as_str),
        )?;
        self.vocab("workload", after.workload.as_ref().map(Workload::as_str))?;
        self.num("desaturationtime", after.desaturation_time)?;
        self.num("noflighttime", after.no_flight_time)?;
        self.notes(&after.notes)?;
        if let Some(rating) = &after.rating {
            self.rating(rating)?;
        }
        self.num("diveduration", after.dive_duration)?;
        self.vocab(
            "divetable",
            after.dive_table.as_ref().map(DiveTable::as_str),
        )?;
        if !after.global_alarms_given.is_empty() {
            self.open("globalalarmsgiven", &[])?;
            for alarm in &after.global_alarms_given {
                self.leaf("globalalarm", &[], alarm.as_str())?;
            }
            self.close("globalalarmsgiven")?;
        }
        self.num("highestpo2", after.highest_po2)?;
        if !after.observation_notes.is_empty() {
            self.open("observations", &[])?;
            self.notes(&after.observation_notes)?;
            self.close("observations")?;
        }
        self.num("averagedepth", after.average_depth)?;
        self.close("informationafterdive")
    }

    fn valued(&mut self, name: &str, value: f64, attrs: &[(&str, Option<&str>)]) -> R {
        let attrs: Vec<(&str, &str)> = attrs
            .iter()
            .filter_map(|(k, v)| v.map(|v| (*k, v)))
            .collect();
        self.leaf(name, &attrs, &number(value))
    }

    fn waypoint(&mut self, point: &Waypoint) -> R {
        self.open("waypoint", &[])?;
        for alarm in &point.alarms {
            let level = alarm.level.map(number);
            self.leaf(
                "alarm",
                &level
                    .iter()
                    .map(|l| ("level", l.as_str()))
                    .collect::<Vec<_>>(),
                alarm.kind.as_ref().map_or("", AlarmType::as_str),
            )?;
        }
        for battery in &point.battery_charge_conditions {
            self.valued(
                "batterychargecondition",
                battery.value,
                &[
                    ("deviceref", battery.device_ref.as_deref()),
                    ("tankref", battery.tank_ref.as_deref()),
                ],
            )?;
        }
        self.num("cns", point.cns)?;
        for stop in &point.deco_stops {
            let depth = stop.depth.map(number);
            let duration = stop.duration.map(number);
            let attrs: Vec<(&str, &str)> = [
                ("kind", stop.kind.as_ref().map(DecoStopKind::as_str)),
                ("decodepth", depth.as_deref()),
                ("duration", duration.as_deref()),
            ]
            .into_iter()
            .filter_map(|(k, v)| v.map(|v| (k, v)))
            .collect();
            self.empty("decostop", &attrs)?;
        }
        self.num("bodytemperature", point.body_temperature)?;
        self.num("calculatedpo2", point.calculated_po2)?;
        self.num("depth", point.depth)?;
        self.num("divetime", point.dive_time)?;
        self.num("heading", point.heading)?;
        self.num("otu", point.otu)?;
        self.num("pulserate", point.pulse_rate)?;
        // shortcut: 3.2.3's <heartrate> has no published XSD position; written beside <pulserate>.
        self.num("heartrate", point.heart_rate)?;
        self.num("remainingbottomtime", point.remaining_bottom_time)?;
        self.num("remainingo2time", point.remaining_o2_time)?;
        self.text("setmarker", point.set_marker.as_ref())?;
        if let Some(set) = &point.set_po2 {
            self.valued(
                "setpo2",
                set.value,
                &[("setby", set.set_by.as_ref().map(SetBy::as_str))],
            )?;
        }
        if let Some(mix) = &point.switch_mix {
            self.empty("switchmix", &[("ref", mix)])?;
        }
        for pressure in &point.tank_pressures {
            self.valued(
                "tankpressure",
                pressure.value,
                &[("ref", pressure.tank_ref.as_deref())],
            )?;
        }
        self.num("temperature", point.temperature)?;
        match &point.dive_mode {
            None => {}
            Some(DiveMode::Unspecified) => self.empty("divemode", &[])?,
            Some(mode) => self.empty("divemode", &[("type", mode.as_str())])?,
        }
        if let Some(factor) = &point.gradient_factor {
            let tissue = factor.tissue.map(|t| t.to_string());
            self.valued(
                "gradientfactor",
                factor.value,
                &[("tissue", tissue.as_deref())],
            )?;
        }
        for measured in &point.measured_po2 {
            self.valued(
                "measuredpo2",
                measured.value,
                &[("ref", measured.sensor_ref.as_deref())],
            )?;
        }
        self.num("nodecotime", point.no_deco_time)?;
        self.close("waypoint")
    }
}
