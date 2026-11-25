//! Reads every vendored real-world file, checks key values, and round-trips each one.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    missing_docs
)]

use std::fs;
use std::path::{Path, PathBuf};

use uddf::{DiveMode, EquipmentKind, SurfaceInterval, Target, Uddf};

fn fixtures() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut paths: Vec<PathBuf> = [root.clone(), root.join("write")]
        .iter()
        .flat_map(|dir| fs::read_dir(dir).unwrap())
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "uddf"))
        .collect();
    paths.sort();
    assert!(paths.len() >= 18, "fixtures missing: {paths:?}");
    paths
}

fn read(name: &str) -> Uddf {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    uddf::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn every_fixture_reads_strictly() {
    for path in fixtures() {
        let text = fs::read_to_string(&path).unwrap();
        if let Err(error) = uddf::from_str(&text) {
            panic!("{}: {error}", path.display());
        }
    }
}

#[test]
fn every_fixture_round_trips() {
    // Set UDDF_WRITE_DIR to keep the written files, for example to validate them
    // against the XSD.
    let keep = std::env::var_os("UDDF_WRITE_DIR").map(PathBuf::from);
    for path in fixtures() {
        let original = uddf::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let written = uddf::to_string(&original).unwrap();
        assert_eq!(
            uddf::to_string(&original).unwrap(),
            written,
            "deterministic"
        );
        let mut again = uddf::from_str(&written).unwrap();
        assert_eq!(again.version.as_deref(), Some("3.2.3"));
        again.version.clone_from(&original.version);
        assert_eq!(again, original, "{}", path.display());
        if let Some(dir) = &keep {
            let name = path
                .strip_prefix(path.parent().unwrap().parent().unwrap())
                .unwrap();
            let name = name.to_string_lossy().replace('/', "-");
            fs::write(dir.join(name), written).unwrap();
        }
    }
}

#[test]
fn shearwater_cloud() {
    let log = read("shearwater-cloud.uddf");
    assert_eq!(log.version.as_deref(), Some("3.2.3"));
    assert_eq!(
        log.generator.name.as_deref(),
        Some("Shearwater Cloud Desktop")
    );
    assert_eq!(
        log.generator.manufacturer.as_ref().unwrap().id.as_deref(),
        Some("Shearwater_Research_Inc")
    );
    let computer = &log.owner.as_ref().unwrap().equipment.dive_computers[0];
    assert_eq!(computer.serial_number.as_deref(), Some("D9772626"));
    assert_eq!(log.mixes[0].o2, Some(0.33));
    assert_eq!(log.mixes[0].maximum_po2, Some(1.0));
    let uddf::DecoModel::Buehlmann(model) = &log.deco_models[0] else {
        panic!()
    };
    assert_eq!(
        (model.gradient_factor_low, model.gradient_factor_high),
        (Some(50.0), Some(85.0))
    );

    let dive = log.dives().next().unwrap();
    let before = &dive.information_before_dive;
    assert_eq!(before.datetime.unwrap().to_string(), "2026-09-08T15:18:10Z");
    assert_eq!(
        before.surface_interval_before_dive,
        Some(SurfaceInterval::PassedTime(86340.0))
    );
    assert_eq!(before.surface_pressure, Some(101_900.0));
    // `<link ref=""/>` carries nothing and is skipped; `" : "` is kept, trimmed.
    assert_eq!(before.links, ["profile_1588035071788880690", ":", "zhl16c"]);
    assert_eq!(dive.tank_data.len(), 2);
    assert_eq!(dive.samples.len(), 12);
    let first = &dive.samples[0];
    assert_eq!(first.depth, Some(1.300_000_07));
    assert_eq!(first.calculated_po2, Some(0.38));
    assert_eq!(first.switch_mix.as_deref(), Some("OC1:33/00"));
    assert_eq!(first.temperature, Some(306.15));
    assert_eq!(first.dive_mode, Some(DiveMode::OpenCircuit));
    assert_eq!(first.battery_charge_conditions[0].value, 1.53);
    assert_eq!(first.battery_charge_conditions[0].device_ref, None);
    assert_eq!(dive.samples[1].no_deco_time, Some(5940.0));
    assert_eq!(dive.information_after_dive.greatest_depth, Some(4.0));

    let refs = log.references();
    assert!(matches!(refs.get("zhl16c"), Some(Target::DecoModel(_))));
    assert!(matches!(
        refs.get(
            &dive
                .information_before_dive
                .equipment_used
                .as_ref()
                .unwrap()
                .links[0]
        ),
        Some(Target::Piece {
            kind: EquipmentKind::DiveComputer,
            ..
        })
    ));
    assert!(refs.get("profile_1588035071788880690").is_none());
}

#[test]
fn legacy_upper_case_writer() {
    let log = read("legacy-writer.uddf");
    assert_eq!(log.version.as_deref(), Some("2.2.0"));
    assert_eq!(log.generator.name.as_deref(), Some("Legacy Logbook"));
    assert_eq!(log.sites[0].id.as_deref(), Some("wreck1"));
    assert_eq!(log.mixes[0].o2, Some(34.0), "whole percent kept as written");
    let dive = log.dives().next().unwrap();
    let start = dive.information_before_dive.datetime.unwrap();
    assert_eq!(
        (start.to_string().as_str(), start.time),
        ("2002-06-18", None)
    );
    assert_eq!(
        dive.tank_data[0].tank_volume,
        Some(12.0),
        "litres kept as written"
    );
    assert_eq!(dive.tank_data[0].links, ["ean34"]);
    let times: Vec<_> = dive.samples.iter().map(|w| w.dive_time).collect();
    assert_eq!(times, [Some(120.0), Some(30.0), Some(30.4), None]);
    assert!(matches!(
        log.references().get("wreck1"),
        Some(Target::Site(_))
    ));
}

#[test]
fn subsurface() {
    let log = read("subsurface.uddf");
    assert_eq!(log.dives().count(), 2);
    assert_eq!(
        log.dives().map(|d| d.samples.len()).collect::<Vec<_>>(),
        [8, 3]
    );
    assert_eq!(
        log.mixes
            .iter()
            .filter_map(|m| m.id.as_deref())
            .collect::<Vec<_>>(),
        ["mix(21/0)", "mix(32/0)"]
    );
    // Every dive reuses its repetition group's id.
    assert_eq!(log.references().duplicates().len(), 2);
    let refs = log.references();
    for dive in log.dives() {
        for link in &dive.information_before_dive.links {
            assert!(refs.get(link).is_some(), "{link} resolves");
        }
    }
}

#[test]
fn divelogs_without_namespace() {
    let log = read("divelogs.uddf");
    let dive = log.dives().next().unwrap();
    assert_eq!(
        dive.information_before_dive.surface_interval_before_dive,
        Some(SurfaceInterval::Infinity)
    );
    assert_eq!(dive.information_before_dive.datetime.unwrap().offset, None);
    assert_eq!(dive.samples.len(), 5);
    assert_eq!(dive.information_after_dive.greatest_depth, Some(45.9));
}

#[test]
fn people_and_trips() {
    let log = read("write/technical-dive.uddf");
    assert_eq!(log.dives().count(), 3);
    assert_eq!(log.buddies.len(), 4);
    assert_eq!(log.mixes.len(), 5);
    let first = log.dives().next().unwrap();
    assert_eq!(first.samples.len(), 17);
    assert_eq!(
        first.information_before_dive.datetime.unwrap().to_string(),
        "2026-07-03T09:14:00+12:45"
    );
    let trip = log
        .dives()
        .find_map(|d| d.information_before_dive.trip_membership.clone())
        .unwrap();
    assert!(matches!(log.references().get(&trip), Some(Target::Trip(_))));
    assert!(log.trips[0].parts.len() >= 2);
    assert!(log.owner.as_ref().unwrap().equipment.pieces().count() > 1);
}

#[test]
fn strict_and_lenient_reading_of_bad_values() {
    let text = r#"<uddf><profiledata><repetitiongroup id="g"><dive id="d"><samples>
        <waypoint><depth>1</depth></waypoint>
        <waypoint><depth>deep</depth><divetime>10</divetime></waypoint>
    </samples></dive></repetitiongroup></profiledata></uddf>"#;
    let uddf::Error::InvalidValue(error) = uddf::from_str(text).unwrap_err() else {
        panic!()
    };
    assert_eq!(
        error.path,
        "/uddf/profiledata/repetitiongroup/dive/samples/waypoint[2]/depth"
    );
    assert_eq!(
        (error.line, error.value.as_str(), error.expected),
        (3, "deep", "a decimal number")
    );

    let (log, warnings) = uddf::from_str_lenient(text).unwrap();
    assert_eq!(warnings, [error]);
    let dive = log.dives().next().unwrap();
    assert_eq!(dive.samples[1].depth, None);
    assert_eq!(dive.samples[1].dive_time, Some(10.0));

    let bad_attr = r#"<uddf><divetrip><trip id="t"><trippart><dateoftrip startdate="soon" enddate="2026-01-02"/></trippart></trip></divetrip></uddf>"#;
    let uddf::Error::InvalidValue(error) = uddf::from_str(bad_attr).unwrap_err() else {
        panic!()
    };
    assert_eq!(
        error.path,
        "/uddf/divetrip/trip/trippart/dateoftrip/@startdate"
    );
}

#[test]
fn refuses_what_is_not_uddf() {
    assert!(
        matches!(uddf::from_str("<dives/>"), Err(uddf::Error::NotUddf(name)) if name == "dives")
    );
    assert!(matches!(
        uddf::from_str("<uddf>"),
        Err(uddf::Error::Xml { .. })
    ));
    let doctype = "<!DOCTYPE uddf [<!ENTITY x \"y\">]><uddf/>";
    assert!(matches!(
        uddf::from_str(doctype),
        Err(uddf::Error::Xml { .. })
    ));
}

#[test]
fn empty_values_are_absent() {
    let log = uddf::from_str(
        r#"<uddf><divesite><site id=" s1 "><name>  </name><geography><location>X</location><latitude/></geography></site></divesite></uddf>"#,
    )
    .unwrap();
    let site = &log.sites[0];
    assert_eq!(site.id.as_deref(), Some("s1"));
    assert_eq!(site.name, None);
    assert_eq!(site.geography.as_ref().unwrap().latitude, None);
    assert!(matches!(log.references().get(" s1"), Some(Target::Site(_))));
}

#[test]
fn written_output_snapshot() {
    insta::assert_snapshot!(uddf::to_string(&read("shearwater-cloud.uddf")).unwrap());
}

#[cfg(feature = "serde")]
#[test]
fn serde_round_trip() {
    for path in fixtures() {
        let log = uddf::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let json = serde_json::to_string(&log).unwrap();
        let back: Uddf = serde_json::from_str(&json).unwrap();
        assert_eq!(back, log, "{}", path.display());
    }
    let json = serde_json::to_value(read("shearwater-cloud.uddf")).unwrap();
    assert_eq!(
        json["repetition_groups"][0]["dives"][0]["samples"][0]["dive_mode"],
        "opencircuit"
    );
}
