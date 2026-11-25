# uddf

A reader and writer for [UDDF](https://www.streit.cc/resources/UDDF/v3.2.3/en/index.html),
the Universal Dive Data Format: the XML dive-log format that Subsurface, Shearwater Cloud,
divelogs.de and many other logbooks export.

- **Reads leniently.** Real files break the schema: elements out of order, required
  elements missing, vendor extras, the 3.2, 3.1 or no namespace, upper-case 2.x tags. The
  reader accepts all of them, and an absent value stays absent.
- **Writes strictly.** The writer produces indented, deterministic UDDF 3.2.3 in schema
  order with the `http://www.streit.cc/uddf/3.2/` namespace.
- **Typed model.** Plain owned structs for the generator, owner and buddies (personal data,
  equipment, certifications, insurances, contacts), dive bases, shops, sites, trips, gas
  mixes, decompression models and dives with their samples. Values keep UDDF's SI units
  (metres, kelvin, pascal, seconds, cubic metres); every field's doc comment states its unit.
- Builds for `wasm32-unknown-unknown`. No `unsafe`.

## Installation

```sh
cargo add uddf
```

Requires Rust 1.85 or newer. Add `--features serde` for the [`serde` derives](#feature-flags).

## Example

```rust
let text = r#"<uddf xmlns="http://www.streit.cc/uddf/3.2/" version="3.2.3">
  <generator><name>Example</name></generator>
  <gasdefinitions><mix id="ean32"><name>EAN32</name><o2>0.32</o2></mix></gasdefinitions>
  <profiledata><repetitiongroup id="rg"><dive id="d1">
    <informationbeforedive><datetime>2026-04-19T08:12:00+02:00</datetime></informationbeforedive>
    <tankdata><link ref="ean32"/><tankpressurebegin>20000000</tankpressurebegin></tankdata>
    <samples>
      <waypoint><depth>0</depth><divetime>0</divetime><switchmix ref="ean32"/></waypoint>
      <waypoint><depth>18.4</depth><divetime>60</divetime><temperature>295.15</temperature></waypoint>
    </samples>
    <informationafterdive><greatestdepth>18.4</greatestdepth><diveduration>2400</diveduration></informationafterdive>
  </dive></repetitiongroup></profiledata>
</uddf>"#;

let log = uddf::from_str(text)?;
let dive = log.dives().next().expect("one dive");
assert_eq!(dive.information_after_dive.greatest_depth, Some(18.4)); // metres
assert_eq!(dive.samples[1].temperature, Some(295.15)); // kelvin

// Cross-references resolve through an index of every element with an id.
let refs = log.references();
let gas = dive.tank_data[0].links.iter().find_map(|r| match refs.get(r) {
    Some(uddf::Target::Mix(mix)) => mix.o2,
    _ => None,
});
assert_eq!(gas, Some(0.32)); // fraction

// Writing gives schema-ordered UDDF 3.2.3 that reads back to the same model.
let written = uddf::to_string(&log)?;
let mut again = uddf::from_str(&written)?;
again.version = log.version.clone();
assert_eq!(again, log);
# Ok::<(), uddf::Error>(())
```

## Reading policy

- Elements and attributes are matched by their lowercased local name, in any order, in any
  namespace. Unknown elements are skipped and not kept.
- Text is trimmed; an element holding only whitespace, such as Subsurface's `<latitude/>`,
  is absent. Ids and references are trimmed too, so `" ff47210"` and its links match.
- Nothing is invented or converted. A writer's known slips — `<o2>34</o2>` in percent,
  `<tankvolume>12</tankvolume>` in litres, Shearwater's bar-valued `<calculatedpo2>` and
  percent gradient factors, a `Z` on a local time — are kept as written for the caller to
  interpret.
- Date-times keep what the file states: date, optional time of day, optional offset
  ([`DateTime`](https://docs.rs/uddf/latest/uddf/struct.DateTime.html)). `2002-06-18` and
  the truncated `2002-06-18T` read as a date without a time.
- A value that cannot be read as its UDDF type (a `<depth>` of `deep`, a `<datetime>` of
  `yesterday`) is an `InvalidValue` naming its element path, line and column.
  `from_str` fails on it; `from_str_lenient` reads it as absent and returns it as a
  warning.
- Malformed XML, a `<!DOCTYPE>` and a root other than `<uddf>` are errors in both modes.

## Writing policy

The writer emits what the model holds, in the 3.2.2 XSD's order (3.2.3 has no newer
schema). It never invents values: a required text element the model lacks (`<name>`,
`<firstname>`, `<country>`, `<location>`) is written empty, and a required number or date
is left out. Ids and references are written as read, so a file whose ids break
`xs:ID` (Subsurface's `mix(21/0)`, Shearwater's missing owner id) stays that way. Each
vendored fixture's output was checked against the 3.2.2 XSD: 15 of 18 validate, and the
other three fail only on what their source states (invalid or repeated ids, a missing
`@deviceref`, a `<buehlmann>` without the tissues the schema requires). A date
without a time of day is written as a bare date, which `xs:dateTime` does not allow but
UDDF's documentation does. `@version` is always `3.2.3`.

## Not modelled

`<mediadata>`, `<tablegeneration>`, `<divecomputercontrol>`, `<applicationdata>`, a
person's `<medical>` and `<divepermissions>`, `<equipmentconfiguration>`, camera parts,
site ecology and wreck/cave/indoor/lake/river/shore data, a vessel's `<shipdimension>`,
species observations, symptoms, drinks and medicine before a dive, planned and input
profiles, surface-interval altitude exposure, and decompression-model tissues. These are
skipped when reading and therefore not written.

## Feature flags

- `serde`: derives `Serialize` and `Deserialize` on the model. Vocabulary enums and
  date-times serialize as their UDDF spelling.

## Licence

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option. Unless you explicitly state otherwise, any
contribution intentionally submitted for inclusion in this crate, as defined in the
Apache-2.0 license, shall be dual licensed as above, without any additional terms or
conditions.
