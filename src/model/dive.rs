//! `<gasdefinitions>`, `<decomodel>` and `<profiledata>`: gases and dives.

use super::{NoteItem, Price, Rating};
use crate::DateTime;

record! {
    /// `<mix>`: a breathing gas.
    pub struct Mix {
        /// `@id`, which `<tankdata>`, `<switchmix>` and `<tankpressure>` refer to.
        pub id: Option<String>,
        /// `<name>`.
        pub name: Option<String>,
        /// `<aliasname>`.
        pub alias_names: Vec<String>,
        /// `<o2>`, fraction (0.21 for air). Some writers wrote whole percent (`21`); the
        /// value is kept as written.
        pub o2: Option<f64>,
        /// `<n2>`, fraction.
        pub n2: Option<f64>,
        /// `<he>`, fraction.
        pub he: Option<f64>,
        /// `<ar>`, fraction.
        pub ar: Option<f64>,
        /// `<h2>`, fraction.
        pub h2: Option<f64>,
        /// `<priceperlitre>`.
        pub price_per_litre: Option<Price>,
        /// `<maximumpo2>`, bar: the one partial pressure UDDF does not give in pascal.
        pub maximum_po2: Option<f64>,
        /// `<maximumoperationdepth>`, metres.
        pub maximum_operation_depth: Option<f64>,
        /// `<equivalentairdepth>`, metres.
        pub equivalent_air_depth: Option<f64>,
    }
}

/// A child of `<decomodel>`. Tissue definitions are not modelled.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[non_exhaustive]
pub enum DecoModel {
    /// `<buehlmann>`.
    Buehlmann(Buehlmann),
    /// `<vpm>`.
    Vpm(Vpm),
    /// `<rgbm>`.
    Rgbm(Rgbm),
}

impl DecoModel {
    /// The model's `@id`, which a dive's `<informationbeforedive>/<link>` refers to.
    #[must_use]
    pub fn id(&self) -> Option<&str> {
        match self {
            Self::Buehlmann(model) => model.id.as_deref(),
            Self::Vpm(model) => model.id.as_deref(),
            Self::Rgbm(model) => model.id.as_deref(),
        }
    }
}

record! {
    /// `<buehlmann>`.
    pub struct Buehlmann {
        /// `@id`.
        pub id: Option<String>,
        /// `<gradientfactorhigh>`, fraction (0.85) per the specification. Shearwater Cloud
        /// Desktop writes whole percent (`85`); the value is kept as written.
        pub gradient_factor_high: Option<f64>,
        /// `<gradientfactorlow>`, fraction; see `gradient_factor_high`.
        pub gradient_factor_low: Option<f64>,
    }
}

record! {
    /// `<vpm>`.
    pub struct Vpm {
        /// `@id`.
        pub id: Option<String>,
        /// `<gamma>`.
        pub gamma: Option<f64>,
        /// `<gc>`.
        pub gc: Option<f64>,
        /// `<lambda>`.
        pub lambda: Option<f64>,
        /// `<r0>`.
        pub r0: Option<f64>,
    }
}

record! {
    /// `<rgbm>`.
    pub struct Rgbm {
        /// `@id`.
        pub id: Option<String>,
    }
}

record! {
    /// `<repetitiongroup>`: dives separated by surface intervals short enough to count
    /// as repetitive.
    pub struct RepetitionGroup {
        /// `@id`.
        pub id: Option<String>,
        /// `<dive>`.
        pub dives: Vec<Dive>,
    }
}

record! {
    /// `<dive>`.
    pub struct Dive {
        /// `@id`.
        pub id: Option<String>,
        /// `<informationbeforedive>`.
        pub information_before_dive: InformationBeforeDive,
        /// `<tankdata>`: the cylinders breathed from.
        pub tank_data: Vec<TankData>,
        /// `<samples>/<waypoint>`: the profile, in file order.
        pub samples: Vec<Waypoint>,
        /// `<informationafterdive>`.
        pub information_after_dive: InformationAfterDive,
    }
}

vocabulary! {
    /// `<apparatus>`.
    Apparatus {
        OpenScuba = "open-scuba",
        Rebreather = "rebreather",
        SurfaceSupplied = "surface-supplied",
        Chamber = "chamber",
        Experimental = "experimental",
    }
}

vocabulary! {
    /// `<platform>`: where the dive started from.
    Platform {
        BeachShore = "beach-shore",
        Pier = "pier",
        SmallBoat = "small-boat",
        CharterBoat = "charter-boat",
        LiveAboard = "live-aboard",
        Barge = "barge",
        Landside = "landside",
        HyperbaricFacility = "hyperbaric-facility",
        Other = "other",
    }
}

vocabulary! {
    /// `<purpose>`.
    Purpose {
        Sightseeing = "sightseeing",
        Learning = "learning",
        Teaching = "teaching",
        Research = "research",
        PhotographyVideography = "photography-videography",
        Spearfishing = "spearfishing",
        Proficiency = "proficiency",
        Work = "work",
        Other = "other",
    }
}

vocabulary! {
    /// `<stateofrestbeforedive>`.
    StateOfRest {
        NotSpecified = "not-specified",
        Rested = "rested",
        Tired = "tired",
        Exhausted = "exhausted",
    }
}

/// `<surfaceintervalbeforedive>` / `<surfaceintervalafterdive>`. Altitude exposure
/// details are not modelled.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[non_exhaustive]
pub enum SurfaceInterval {
    /// `<infinity/>`: no previous dive counts (also read from the misspelling
    /// `<inifinity/>`).
    Infinity,
    /// `<passedtime>`, seconds.
    PassedTime(f64),
}

record! {
    /// `<equipmentused>`.
    pub struct EquipmentUsed {
        /// `<leadquantity>`, kilograms.
        pub lead_quantity: Option<f64>,
        /// `<link ref>`: the equipment pieces used.
        pub links: Vec<String>,
    }
}

record! {
    /// `<informationbeforedive>`. Drinks, medicine and planned or input profiles are not
    /// modelled.
    pub struct InformationBeforeDive {
        /// `<link ref>`: sites, buddies, guides, dive bases, shops and deco models, in file
        /// order.
        pub links: Vec<String>,
        /// `<divenumber>`: the diver's own count.
        pub dive_number: Option<u32>,
        /// `<divenumberofday>`.
        pub dive_number_of_day: Option<u32>,
        /// `<internaldivenumber>`: the dive computer's count.
        pub internal_dive_number: Option<u32>,
        /// `<datetime>`: when the dive started (required by the schema).
        pub datetime: Option<DateTime>,
        /// `<airtemperature>`, kelvin.
        pub air_temperature: Option<f64>,
        /// `<surfaceintervalbeforedive>`.
        pub surface_interval_before_dive: Option<SurfaceInterval>,
        /// `<altitude>`, metres above sea level.
        pub altitude: Option<f64>,
        /// `<equipmentused>`.
        pub equipment_used: Option<EquipmentUsed>,
        /// `<apparatus>`.
        pub apparatus: Option<Apparatus>,
        /// `<platform>`.
        pub platform: Option<Platform>,
        /// `<purpose>`.
        pub purpose: Option<Purpose>,
        /// `<stateofrestbeforedive>`.
        pub state_of_rest_before_dive: Option<StateOfRest>,
        /// `<tripmembership ref>`: the trip this dive belongs to.
        pub trip_membership: Option<String>,
        /// `<nosuit/>`: dived without a suit.
        pub no_suit: bool,
        /// `<price>`.
        pub price: Option<Price>,
        /// `<surfacepressure>`, pascal.
        pub surface_pressure: Option<f64>,
    }
}

vocabulary! {
    /// `<current>`.
    Current {
        NoCurrent = "no-current",
        VeryMildCurrent = "very-mild-current",
        MildCurrent = "mild-current",
        ModerateCurrent = "moderate-current",
        HardCurrent = "hard-current",
        VeryHardCurrent = "very-hard-current",
    }
}

vocabulary! {
    /// `<diveplan>`.
    DivePlan {
        NoPlan = "none",
        Table = "table",
        DiveComputer = "dive-computer",
        AnotherDiver = "another-diver",
    }
}

vocabulary! {
    /// `<equipmentmalfunction>`.
    EquipmentMalfunction {
        NoMalfunction = "none",
        FaceMask = "face-mask",
        Fins = "fins",
        WeightBelt = "weight-belt",
        BuoyancyControlDevice = "buoyancy-control-device",
        ThermalProtection = "thermal-protection",
        DiveComputer = "dive-computer",
        DepthGauge = "depth-gauge",
        PressureGauge = "pressure-gauge",
        BreathingApparatus = "breathing-apparatus",
        DecoReel = "deco-reel",
        Other = "other",
    }
}

vocabulary! {
    /// `<problems>`.
    Problems {
        NoProblems = "none",
        Equalisation = "equalisation",
        Vertigo = "vertigo",
        OutOfAir = "out-of-air",
        Buoyancy = "buoyancy",
        SharedAir = "shared-air",
        RapidAscent = "rapid-ascent",
        SeaSickness = "sea-sickness",
        Other = "other",
    }
}

vocabulary! {
    /// `<program>`.
    Program {
        Recreation = "recreation",
        Training = "training",
        Scientific = "scientific",
        Medical = "medical",
        Commercial = "commercial",
        Military = "military",
        Competitive = "competitive",
        Other = "other",
    }
}

vocabulary! {
    /// `<thermalcomfort>`.
    ThermalComfort {
        NotIndicated = "not-indicated",
        Comfortable = "comfortable",
        Cold = "cold",
        VeryCold = "very-cold",
        Hot = "hot",
    }
}

vocabulary! {
    /// `<workload>`.
    Workload {
        NotSpecified = "not-specified",
        Resting = "resting",
        Light = "light",
        Moderate = "moderate",
        Severe = "severe",
        Exhausting = "exhausting",
    }
}

vocabulary! {
    /// `<divetable>`.
    DiveTable {
        Padi = "PADI",
        Naui = "NAUI",
        Bsac = "BSAC",
        Buehlmann = "Buehlmann",
        Dciem = "DCIEM",
        UsNavy = "US-Navy",
        Csmd = "CSMD",
        Comex = "COMEX",
        Other = "other",
    }
}

vocabulary! {
    /// `<globalalarmsgiven>/<globalalarm>`.
    GlobalAlarm {
        AscentWarningTooLong = "ascent-warning-too-long",
        SosMode = "sos-mode",
        WorkTooHard = "work-too-hard",
    }
}

record! {
    /// `<informationafterdive>`. Symptoms and species observations are not modelled.
    pub struct InformationAfterDive {
        /// `<surfaceintervalafterdive>`.
        pub surface_interval_after_dive: Option<SurfaceInterval>,
        /// `<lowesttemperature>`, kelvin.
        pub lowest_temperature: Option<f64>,
        /// `<greatestdepth>`, metres (required by the schema; writers with nothing to say
        /// often write `0`).
        pub greatest_depth: Option<f64>,
        /// `<visibility>`, metres.
        pub visibility: Option<f64>,
        /// `<current>`.
        pub current: Option<Current>,
        /// `<diveplan>`.
        pub dive_plan: Option<DivePlan>,
        /// `<equipmentmalfunction>`.
        pub equipment_malfunction: Option<EquipmentMalfunction>,
        /// `<pressuredrop>`, pascal.
        pub pressure_drop: Option<f64>,
        /// `<problems>`.
        pub problems: Option<Problems>,
        /// `<program>`.
        pub program: Option<Program>,
        /// `<thermalcomfort>`.
        pub thermal_comfort: Option<ThermalComfort>,
        /// `<workload>`.
        pub workload: Option<Workload>,
        /// `<desaturationtime>`, seconds.
        pub desaturation_time: Option<f64>,
        /// `<noflighttime>`, seconds.
        pub no_flight_time: Option<f64>,
        /// `<notes>`.
        pub notes: Vec<NoteItem>,
        /// `<rating>`.
        pub rating: Option<Rating>,
        /// `<diveduration>`, seconds (required by the schema).
        pub dive_duration: Option<f64>,
        /// `<divetable>`.
        pub dive_table: Option<DiveTable>,
        /// `<globalalarmsgiven>/<globalalarm>`.
        pub global_alarms_given: Vec<GlobalAlarm>,
        /// `<highestpo2>`, pascal per the specification.
        pub highest_po2: Option<f64>,
        /// `<observations>/<notes>`.
        pub observation_notes: Vec<NoteItem>,
        /// `<averagedepth>`, metres.
        pub average_depth: Option<f64>,
    }
}

record! {
    /// `<tankdata>`: one cylinder breathed from on a dive.
    pub struct TankData {
        /// `<link ref>`: the `<mix>` it held and/or the `<tank>` it was, in file order.
        pub links: Vec<String>,
        /// `<tankvolume>`, cubic metres per the specification (older Subsurface wrote
        /// litres; kept as written). May be omitted when a linked `<tank>` gives it.
        pub tank_volume: Option<f64>,
        /// `<tankpressurebegin>`, pascal (required by the schema; `0` often means
        /// unknown).
        pub tank_pressure_begin: Option<f64>,
        /// `<tankpressureend>`, pascal.
        pub tank_pressure_end: Option<f64>,
        /// `<breathingconsumptionvolume>`, cubic metres per second.
        pub breathing_consumption_volume: Option<f64>,
    }
}

vocabulary! {
    /// `<alarm>` text.
    AlarmType {
        Ascent = "ascent",
        Breath = "breath",
        Deco = "deco",
        Error = "error",
        Link = "link",
        Microbubbles = "microbubbles",
        Rbt = "rbt",
        SkinCooling = "skincooling",
        Surface = "surface",
    }
}

record! {
    /// `<alarm>`.
    pub struct Alarm {
        /// The alarm, from the element's text.
        pub kind: Option<AlarmType>,
        /// `@level`.
        pub level: Option<f64>,
    }
}

record! {
    /// `<batterychargecondition>`.
    pub struct BatteryChargeCondition {
        /// The reading, kept as written (Shearwater writes volts).
        pub value: f64,
        /// `@deviceref`: the device whose battery this is (required by the schema).
        pub device_ref: Option<String>,
        /// `@tankref`: the tank whose transmitter's battery this is.
        pub tank_ref: Option<String>,
    }
}

vocabulary! {
    /// `<decostop @kind>`.
    DecoStopKind {
        Safety = "safety",
        Mandatory = "mandatory",
    }
}

record! {
    /// `<decostop>`: a stop the computer calls for.
    pub struct DecoStop {
        /// `@kind`.
        pub kind: Option<DecoStopKind>,
        /// `@decodepth`, metres.
        pub depth: Option<f64>,
        /// `@duration`, seconds.
        pub duration: Option<f64>,
    }
}

vocabulary! {
    /// `<setpo2 @setby>`.
    SetBy {
        User = "user",
        Computer = "computer",
    }
}

record! {
    /// `<setpo2>`: the maximum tolerated ppO₂ set on the computer.
    pub struct SetPo2 {
        /// The value, pascal per the specification.
        pub value: f64,
        /// `@setby`.
        pub set_by: Option<SetBy>,
    }
}

record! {
    /// `<tankpressure>`.
    pub struct TankPressure {
        /// The pressure, pascal.
        pub value: f64,
        /// `@ref`: the `<mix>` (or tank) the reading belongs to; absent when there is one.
        pub tank_ref: Option<String>,
    }
}

vocabulary! {
    /// `<divemode @type>`.
    DiveMode {
        /// `<divemode/>` without a `@type` (the attribute is optional in the 3.2.2
        /// schema); written back without one.
        Unspecified = "",
        Apnea = "apnea",
        /// The original German spelling of `apnea`; both are valid.
        Apnoe = "apnoe",
        ClosedCircuit = "closedcircuit",
        OpenCircuit = "opencircuit",
        SemiClosedCircuit = "semiclosedcircuit",
    }
}

record! {
    /// `<gradientfactor>`.
    pub struct GradientFactor {
        /// The value, a fraction per the specification's example; Shearwater Cloud Desktop
        /// writes whole percent. Kept as written.
        pub value: f64,
        /// `@tissue`: the leading tissue's number.
        pub tissue: Option<i32>,
    }
}

record! {
    /// `<measuredpo2>`: one oxygen sensor's reading.
    pub struct MeasuredPo2 {
        /// The value, pascal per the specification.
        pub value: f64,
        /// `@ref`: the `<o2sensor>` that measured it.
        pub sensor_ref: Option<String>,
    }
}

record! {
    /// `<waypoint>`: everything recorded at one instant of the profile.
    ///
    /// Values a writer states only on change (temperature, for example) are absent on the
    /// other waypoints; nothing is carried forward.
    pub struct Waypoint {
        /// `<alarm>`.
        pub alarms: Vec<Alarm>,
        /// `<batterychargecondition>`.
        pub battery_charge_conditions: Vec<BatteryChargeCondition>,
        /// `<cns>`, percent of the CNS oxygen toxicity limit.
        pub cns: Option<f64>,
        /// `<decostop>`.
        pub deco_stops: Vec<DecoStop>,
        /// `<bodytemperature>`, kelvin.
        pub body_temperature: Option<f64>,
        /// `<calculatedpo2>`, pascal per the specification. Shearwater Cloud Desktop writes
        /// bar; kept as written.
        pub calculated_po2: Option<f64>,
        /// `<depth>`, metres.
        pub depth: Option<f64>,
        /// `<divetime>`, seconds since the start of the dive.
        pub dive_time: Option<f64>,
        /// `<heading>`, degrees.
        pub heading: Option<f64>,
        /// `<otu>`, oxygen toxicity units.
        pub otu: Option<f64>,
        /// `<pulserate>`, beats per second per the specification.
        pub pulse_rate: Option<f64>,
        /// `<heartrate>` (new in 3.2.3), beats per second per the specification; at least
        /// one writer uses beats per minute. Not in the 3.2.2 schema.
        pub heart_rate: Option<f64>,
        /// `<remainingbottomtime>`, seconds.
        pub remaining_bottom_time: Option<f64>,
        /// `<remainingo2time>`, seconds.
        pub remaining_o2_time: Option<f64>,
        /// `<setmarker>`: a free-text marker.
        pub set_marker: Option<String>,
        /// `<setpo2>`.
        pub set_po2: Option<SetPo2>,
        /// `<switchmix ref>`: the `<mix>` switched to at this instant.
        pub switch_mix: Option<String>,
        /// `<tankpressure>`: one per cylinder read.
        pub tank_pressures: Vec<TankPressure>,
        /// `<temperature>`, kelvin.
        pub temperature: Option<f64>,
        /// `<divemode @type>`.
        pub dive_mode: Option<DiveMode>,
        /// `<gradientfactor>`.
        pub gradient_factor: Option<GradientFactor>,
        /// `<measuredpo2>`: one per oxygen sensor.
        pub measured_po2: Vec<MeasuredPo2>,
        /// `<nodecotime>`, seconds of no-decompression time left.
        pub no_deco_time: Option<f64>,
    }
}
