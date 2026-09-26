//! The online tool registry.
//!
//! Task three of the service is an online test bench for the `apsis-rs`
//! kernels. The registry is the single description of what the service can
//! run: every tool declares its parameters, their types, their units, and
//! their defaults, and the frontend renders a form from that description.
//! Adding a tool therefore means adding one entry here and one handler in
//! [`handlers`], with no change to the frontend.
//!
//! Parameter values arrive from the client as a JSON object. [`Params`]
//! wraps that object and performs the typed reads and the validation, so a
//! handler never has to parse a value itself and never sees a value of the
//! wrong type.

pub mod handlers;
pub mod out;

use std::collections::HashMap;

use crate::json::Value;
use crate::json_object;

/// The type of a parameter, selecting the form control the frontend draws.
#[derive(Debug, Clone)]
pub enum ParamKind {
    /// A floating point number, optionally bounded and with a unit.
    Number {
        /// Unit shown next to the input, or an empty string.
        unit: &'static str,
        /// Smallest accepted value, when bounded.
        min: Option<f64>,
        /// Largest accepted value, when bounded.
        max: Option<f64>,
    },
    /// An integer, optionally bounded.
    Integer {
        /// Smallest accepted value, when bounded.
        min: Option<i64>,
        /// Largest accepted value, when bounded.
        max: Option<i64>,
    },
    /// A free text field of several lines, drawn as a text area.
    Multiline,
    /// A choice among a fixed set of values.
    Select(&'static [SelectOption]),
}

/// One option of a [`ParamKind::Select`] parameter.
#[derive(Debug, Clone, Copy)]
pub struct SelectOption {
    /// Value sent by the frontend.
    pub value: &'static str,
    /// Label shown to the user.
    pub label: &'static str,
}

/// The description of one tool parameter.
#[derive(Debug, Clone)]
pub struct ParamSpec {
    /// Machine name, used as the JSON member name.
    pub name: &'static str,
    /// Human readable label.
    pub label: &'static str,
    /// One sentence explanation shown as help text.
    pub help: &'static str,
    /// Type and constraints.
    pub kind: ParamKind,
    /// Default value, as it would be typed into the form.
    pub default: Option<&'static str>,
    /// Whether the form must supply a value.
    pub required: bool,
}

/// The description of one tool.
#[derive(Debug, Clone)]
pub struct ToolSpec {
    /// Identifier used in the route.
    pub id: &'static str,
    /// Display name.
    pub name: &'static str,
    /// One sentence summary.
    pub summary: &'static str,
    /// Longer description, shown above the form.
    pub description: &'static str,
    /// Grouping shown in the tool list.
    pub category: &'static str,
    /// Parameters, in form order.
    pub params: &'static [ParamSpec],
    /// The `apsis-rs` items the tool calls, for cross linking to the
    /// interface documentation.
    pub api_items: &'static [&'static str],
}

/// The registry of every tool.
pub struct Registry {
    tools: Vec<ToolSpec>,
    by_id: HashMap<&'static str, usize>,
}

impl Registry {
    /// Builds the registry from the tool table.
    pub fn new() -> Self {
        let tools = tool_table();
        let by_id = tools
            .iter()
            .enumerate()
            .map(|(index, tool)| (tool.id, index))
            .collect();
        Self { tools, by_id }
    }

    /// Returns every tool.
    pub fn entries(&self) -> &[ToolSpec] {
        &self.tools
    }

    /// Returns a tool by identifier.
    pub fn get(&self, id: &str) -> Option<&ToolSpec> {
        self.by_id.get(id).map(|index| &self.tools[*index])
    }

    /// Returns the distinct categories in the order they first appear.
    pub fn categories(&self) -> Vec<&'static str> {
        let mut categories = Vec::new();
        for tool in &self.tools {
            if !categories.contains(&tool.category) {
                categories.push(tool.category);
            }
        }
        categories
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns the tool table.
fn tool_table() -> Vec<ToolSpec> {
    vec![
        ToolSpec {
            id: "parse-tle",
            name: "Parse a two-line element set",
            summary: "Decode the fields of a TLE and verify the line checksums.",
            description: "Reads a two-line element set from either the name line plus two data \
                 lines or the two data lines alone, verifies the checksum of each line, and \
                 reports every field with its unit as the format defines it.",
            category: "Elements",
            params: &[ParamSpec {
                name: "tle",
                label: "Element set",
                help: "Two or three lines: an optional name line followed by the two data lines.",
                kind: ParamKind::Multiline,
                default: Some(DEFAULT_TLE),
                required: true,
            }],
            api_items: &["TwoLineElement", "parse"],
        },
        ToolSpec {
            id: "sgp4-propagate",
            name: "Propagate with SGP4",
            summary: "Propagate mean elements to one instant with the SGP4 or SDP4 branch.",
            description: "Prepares the mean elements for propagation and returns the position and \
                 velocity in the True Equator Mean Equinox frame at a chosen number of minutes \
                 from the element epoch.",
            category: "Propagation",
            params: &[
                ParamSpec {
                    name: "tle",
                    label: "Element set",
                    help: "The mean elements to propagate.",
                    kind: ParamKind::Multiline,
                    default: Some(DEFAULT_TLE),
                    required: true,
                },
                ParamSpec {
                    name: "model",
                    label: "Gravity model",
                    help: "Must match the model the elements were fitted against.",
                    kind: ParamKind::Select(&GRAVITY_MODELS),
                    default: Some("wgs72"),
                    required: true,
                },
                ParamSpec {
                    name: "minutes",
                    label: "Minutes from epoch",
                    help: "Elapsed time from the epoch written in the element set.",
                    kind: ParamKind::Number {
                        unit: "min",
                        min: Some(-100_000.0),
                        max: Some(1_000_000.0),
                    },
                    default: Some("0"),
                    required: true,
                },
            ],
            api_items: &["Satellite", "propagate_minutes", "State"],
        },
        ToolSpec {
            id: "sgp4-track",
            name: "Sample a ground track",
            summary: "Propagate a track on a time grid for plotting.",
            description: "Samples the orbit into an ephemeris table over a range of minutes and \
                 returns the points so that a track and an altitude profile can be plotted.",
            category: "Propagation",
            params: &[
                ParamSpec {
                    name: "tle",
                    label: "Element set",
                    help: "The mean elements to propagate.",
                    kind: ParamKind::Multiline,
                    default: Some(DEFAULT_TLE),
                    required: true,
                },
                ParamSpec {
                    name: "model",
                    label: "Gravity model",
                    help: "Must match the model the elements were fitted against.",
                    kind: ParamKind::Select(&GRAVITY_MODELS),
                    default: Some("wgs72"),
                    required: true,
                },
                ParamSpec {
                    name: "start_minutes",
                    label: "Start",
                    help: "First sample, in minutes from the element epoch.",
                    kind: ParamKind::Number {
                        unit: "min",
                        min: Some(-100_000.0),
                        max: Some(1_000_000.0),
                    },
                    default: Some("0"),
                    required: true,
                },
                ParamSpec {
                    name: "end_minutes",
                    label: "End",
                    help: "Last sample, in minutes from the element epoch.",
                    kind: ParamKind::Number {
                        unit: "min",
                        min: Some(-100_000.0),
                        max: Some(1_000_000.0),
                    },
                    default: Some("100"),
                    required: true,
                },
                ParamSpec {
                    name: "step_minutes",
                    label: "Step",
                    help: "Interval between samples. Between two and two thousand samples are allowed.",
                    kind: ParamKind::Number {
                        unit: "min",
                        min: Some(0.001),
                        max: Some(100_000.0),
                    },
                    default: Some("2"),
                    required: true,
                },
            ],
            api_items: &["Orbit", "sample_ephemeris", "Ephemeris"],
        },
        ToolSpec {
            id: "orbit-osculating",
            name: "Compare mean and osculating elements",
            summary: "Show the osculating two-body elements beside the mean elements.",
            description: "Propagates the mean elements and converts the instantaneous state to \
                 Keplerian elements. The two sets differ because SGP4 includes perturbations \
                 that the two-body model does not, so the comparison shows what the \
                 perturbations do.",
            category: "Propagation",
            params: &[
                ParamSpec {
                    name: "tle",
                    label: "Element set",
                    help: "The mean elements to propagate.",
                    kind: ParamKind::Multiline,
                    default: Some(DEFAULT_TLE),
                    required: true,
                },
                ParamSpec {
                    name: "model",
                    label: "Gravity model",
                    help: "Must match the model the elements were fitted against.",
                    kind: ParamKind::Select(&GRAVITY_MODELS),
                    default: Some("wgs72"),
                    required: true,
                },
                ParamSpec {
                    name: "minutes",
                    label: "Minutes from epoch",
                    help: "Instant at which the osculating elements are evaluated.",
                    kind: ParamKind::Number {
                        unit: "min",
                        min: Some(-100_000.0),
                        max: Some(1_000_000.0),
                    },
                    default: Some("0"),
                    required: true,
                },
            ],
            api_items: &["Orbit", "osculating_elements", "KeplerianElements"],
        },
        ToolSpec {
            id: "kepler-state",
            name: "Keplerian elements to a state vector",
            summary: "Convert the six classical elements to position and velocity.",
            description: "Solves the two-body problem for the given elements and returns the \
                 position and velocity in the inertial frame the elements are referred to, \
                 together with the derived size of the orbit.",
            category: "Orbits",
            params: &[
                ParamSpec {
                    name: "semi_major_axis",
                    label: "Semi-major axis",
                    help: "Positive size of the orbit.",
                    kind: ParamKind::Number {
                        unit: "km",
                        min: Some(1.0),
                        max: Some(1_000_000.0),
                    },
                    default: Some("7000"),
                    required: true,
                },
                ParamSpec {
                    name: "eccentricity",
                    label: "Eccentricity",
                    help: "Dimensionless shape, at or above zero and below one.",
                    kind: ParamKind::Number {
                        unit: "",
                        min: Some(0.0),
                        max: Some(1.0),
                    },
                    default: Some("0.001"),
                    required: true,
                },
                ParamSpec {
                    name: "inclination",
                    label: "Inclination",
                    help: "Angle between the orbit plane and the equator.",
                    kind: ParamKind::Number {
                        unit: "deg",
                        min: Some(0.0),
                        max: Some(180.0),
                    },
                    default: Some("51.6"),
                    required: true,
                },
                ParamSpec {
                    name: "right_ascension",
                    label: "Right ascension of the node",
                    help: "Where the orbit crosses the equator going north.",
                    kind: ParamKind::Number {
                        unit: "deg",
                        min: None,
                        max: None,
                    },
                    default: Some("120"),
                    required: true,
                },
                ParamSpec {
                    name: "argument_of_perigee",
                    label: "Argument of perigee",
                    help: "Angle from the ascending node to perigee, in the orbit plane.",
                    kind: ParamKind::Number {
                        unit: "deg",
                        min: None,
                        max: None,
                    },
                    default: Some("90"),
                    required: true,
                },
                ParamSpec {
                    name: "true_anomaly",
                    label: "True anomaly",
                    help: "Angle from perigee to the satellite.",
                    kind: ParamKind::Number {
                        unit: "deg",
                        min: None,
                        max: None,
                    },
                    default: Some("0"),
                    required: true,
                },
                ParamSpec {
                    name: "model",
                    label: "Gravity model",
                    help: "Selects the gravitational parameter used by the conversion.",
                    kind: ParamKind::Select(&GRAVITY_MODELS),
                    default: Some("wgs72"),
                    required: true,
                },
            ],
            api_items: &["KeplerianElements", "to_state", "GravityParameter"],
        },
        ToolSpec {
            id: "time-convert",
            name: "Convert a time scale",
            summary: "Convert an instant between UTC, TAI, TT, and GPS.",
            description: "Accepts either a calendar date or a Modified Julian Date and reports \
                 the instant on every supported scale. The leap second table is the one \
                 embedded in `apsis-rs`, so UTC conversions are exact over the range the table \
                 covers.",
            category: "Time",
            params: &[
                ParamSpec {
                    name: "input",
                    label: "Input form",
                    help: "Whether the instant is given as a calendar date or as a Modified Julian Date.",
                    kind: ParamKind::Select(&TIME_INPUTS),
                    default: Some("calendar"),
                    required: true,
                },
                ParamSpec {
                    name: "scale",
                    label: "Input scale",
                    help: "The scale the input is expressed on.",
                    kind: ParamKind::Select(&TIME_SCALES),
                    default: Some("utc"),
                    required: true,
                },
                ParamSpec {
                    name: "year",
                    label: "Year",
                    help: "Calendar year, used when the input form is a calendar date.",
                    kind: ParamKind::Integer {
                        min: Some(1950),
                        max: Some(2100),
                    },
                    default: Some("2024"),
                    required: false,
                },
                ParamSpec {
                    name: "month",
                    label: "Month",
                    help: "Calendar month, used when the input form is a calendar date.",
                    kind: ParamKind::Integer {
                        min: Some(1),
                        max: Some(12),
                    },
                    default: Some("1"),
                    required: false,
                },
                ParamSpec {
                    name: "day",
                    label: "Day",
                    help: "Day of the month, used when the input form is a calendar date.",
                    kind: ParamKind::Integer {
                        min: Some(1),
                        max: Some(31),
                    },
                    default: Some("1"),
                    required: false,
                },
                ParamSpec {
                    name: "hour",
                    label: "Hour",
                    help: "Hour of the day, used when the input form is a calendar date.",
                    kind: ParamKind::Integer {
                        min: Some(0),
                        max: Some(23),
                    },
                    default: Some("0"),
                    required: false,
                },
                ParamSpec {
                    name: "minute",
                    label: "Minute",
                    help: "Minute of the hour, used when the input form is a calendar date.",
                    kind: ParamKind::Integer {
                        min: Some(0),
                        max: Some(59),
                    },
                    default: Some("0"),
                    required: false,
                },
                ParamSpec {
                    name: "second",
                    label: "Second",
                    help: "Second of the minute, including the fraction, used when the input form is a calendar date.",
                    kind: ParamKind::Number {
                        unit: "s",
                        min: Some(0.0),
                        max: Some(61.0),
                    },
                    default: Some("0"),
                    required: false,
                },
                ParamSpec {
                    name: "mjd",
                    label: "Modified Julian Date",
                    help: "Instant on the input scale, used when the input form is a Modified Julian Date.",
                    kind: ParamKind::Number {
                        unit: "day",
                        min: Some(0.0),
                        max: Some(100_000.0),
                    },
                    default: Some("60000"),
                    required: false,
                },
            ],
            api_items: &["Epoch", "TimeScale", "Calendar"],
        },
        ToolSpec {
            id: "geodetic-ecef",
            name: "Geodetic position to Earth-fixed coordinates",
            summary: "Convert latitude, longitude, and height to an Earth-fixed vector.",
            description: "Converts a geodetic position on the WGS 84 ellipsoid to a Cartesian \
                 vector in the Earth-centered Earth-fixed frame, and reports the derived radii \
                 of curvature and the geocentric latitude for comparison.",
            category: "Coordinates",
            params: &[
                ParamSpec {
                    name: "latitude",
                    label: "Geodetic latitude",
                    help: "Angle between the ellipsoid normal and the equatorial plane.",
                    kind: ParamKind::Number {
                        unit: "deg",
                        min: Some(-90.0),
                        max: Some(90.0),
                    },
                    default: Some("45"),
                    required: true,
                },
                ParamSpec {
                    name: "longitude",
                    label: "Longitude",
                    help: "Positive east of the prime meridian.",
                    kind: ParamKind::Number {
                        unit: "deg",
                        min: Some(-180.0),
                        max: Some(180.0),
                    },
                    default: Some("0"),
                    required: true,
                },
                ParamSpec {
                    name: "height",
                    label: "Height above the ellipsoid",
                    help: "May be negative, which places the point below the ellipsoid.",
                    kind: ParamKind::Number {
                        unit: "m",
                        min: Some(-11_000.0),
                        max: Some(40_000_000.0),
                    },
                    default: Some("0"),
                    required: true,
                },
            ],
            api_items: &["Geodetic", "to_ecef", "WGS84"],
        },
        ToolSpec {
            id: "frame-transform",
            name: "Rotate between inertial and Earth-fixed frames",
            summary: "Apply the Earth Rotation Angle to a Cartesian vector.",
            description: "Rotates a position between the Earth-centered inertial frame and the \
                 Earth-centered Earth-fixed frame at a chosen Earth Rotation Angle. The \
                 rotation preserves the length of the vector, which the result reports.",
            category: "Coordinates",
            params: &[
                ParamSpec {
                    name: "direction",
                    label: "Direction",
                    help: "Which way to rotate.",
                    kind: ParamKind::Select(&FRAME_DIRECTIONS),
                    default: Some("eci-to-ecef"),
                    required: true,
                },
                ParamSpec {
                    name: "x",
                    label: "X",
                    help: "First component of the position.",
                    kind: ParamKind::Number {
                        unit: "m",
                        min: None,
                        max: None,
                    },
                    default: Some("6871000"),
                    required: true,
                },
                ParamSpec {
                    name: "y",
                    label: "Y",
                    help: "Second component of the position.",
                    kind: ParamKind::Number {
                        unit: "m",
                        min: None,
                        max: None,
                    },
                    default: Some("0"),
                    required: true,
                },
                ParamSpec {
                    name: "z",
                    label: "Z",
                    help: "Third component of the position.",
                    kind: ParamKind::Number {
                        unit: "m",
                        min: None,
                        max: None,
                    },
                    default: Some("0"),
                    required: true,
                },
                ParamSpec {
                    name: "earth_rotation",
                    label: "Earth Rotation Angle",
                    help: "Angle of the Earth's rotation since the J2000.0 epoch, in turns.",
                    kind: ParamKind::Number {
                        unit: "turn",
                        min: None,
                        max: None,
                    },
                    default: Some("0.25"),
                    required: true,
                },
            ],
            api_items: &["Eci", "Ecef", "EarthRotationAngle"],
        },
        ToolSpec {
            id: "hohmann-transfer",
            name: "Plan a Hohmann transfer",
            summary: "Compute the two burns of a transfer between coplanar circular orbits.",
            description: "Returns the two velocity changes that move a satellite between two \
                 circular coplanar orbits, the coast time between them, and the total velocity \
                 change. A positive burn raises the orbit and a negative one lowers it.",
            category: "Orbits",
            params: &[
                ParamSpec {
                    name: "from_radius",
                    label: "Starting orbit radius",
                    help: "Radius of the circular orbit the transfer starts from.",
                    kind: ParamKind::Number {
                        unit: "km",
                        min: Some(1.0),
                        max: Some(1_000_000.0),
                    },
                    default: Some("6778"),
                    required: true,
                },
                ParamSpec {
                    name: "to_radius",
                    label: "Target orbit radius",
                    help: "Radius of the circular orbit the transfer arrives at. It must differ from the starting radius.",
                    kind: ParamKind::Number {
                        unit: "km",
                        min: Some(1.0),
                        max: Some(1_000_000.0),
                    },
                    default: Some("42164"),
                    required: true,
                },
                ParamSpec {
                    name: "model",
                    label: "Gravity model",
                    help: "Selects the gravitational parameter used by the transfer.",
                    kind: ParamKind::Select(&GRAVITY_MODELS),
                    default: Some("wgs72"),
                    required: true,
                },
            ],
            api_items: &["HohmannTransfer", "hohmann_transfer"],
        },
    ]
}

/// The gravity model options shared by the propagation tools.
const GRAVITY_MODELS: [SelectOption; 2] = [
    SelectOption {
        value: "wgs72",
        label: "WGS 72 (the model SGP4 was fitted to)",
    },
    SelectOption {
        value: "wgs84",
        label: "WGS 84",
    },
];

/// The time scale options.
const TIME_SCALES: [SelectOption; 4] = [
    SelectOption {
        value: "utc",
        label: "UTC (Coordinated Universal Time)",
    },
    SelectOption {
        value: "tai",
        label: "TAI (International Atomic Time)",
    },
    SelectOption {
        value: "tt",
        label: "TT (Terrestrial Time)",
    },
    SelectOption {
        value: "gps",
        label: "GPS (GPS time)",
    },
];

/// The input form options of the time tool.
const TIME_INPUTS: [SelectOption; 2] = [
    SelectOption {
        value: "calendar",
        label: "Calendar date",
    },
    SelectOption {
        value: "mjd",
        label: "Modified Julian Date",
    },
];

/// The rotation direction options of the frame tool.
const FRAME_DIRECTIONS: [SelectOption; 2] = [
    SelectOption {
        value: "eci-to-ecef",
        label: "Inertial to Earth-fixed",
    },
    SelectOption {
        value: "ecef-to-eci",
        label: "Earth-fixed to inertial",
    },
];

/// A sample element set, used as the default of the propagation tools.
///
/// It is the International Space Station element set that the `apsis-rs`
/// crate documentation uses in its examples, so the values in the interface
/// documentation and in the tool agree.
const DEFAULT_TLE: &str = "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927\n2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537";

/// A tool execution failure.
///
/// The message is written for the user of the form. It names the parameter
/// and the accepted range and never quotes internal state, so a failure
/// cannot leak the shape of the implementation.
#[derive(Debug, Clone)]
pub struct ToolError {
    /// Name of the parameter at fault, when the failure is a bad input.
    pub parameter: Option<&'static str>,
    /// Message shown to the user.
    pub message: String,
}

impl ToolError {
    /// Creates a failure for a named parameter.
    pub fn parameter(name: &'static str, message: impl Into<String>) -> Self {
        Self {
            parameter: Some(name),
            message: message.into(),
        }
    }

    /// Creates a failure that is not tied to one parameter.
    pub fn general(message: impl Into<String>) -> Self {
        Self {
            parameter: None,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ToolError {}

/// The parameter values of one tool call.
///
/// The wrapper performs the typed reads and validates against the tool's
/// [`ParamSpec`] list, so handlers deal in `f64`, `i64`, `String`, and
/// enumerated choices rather than in JSON.
pub struct Params<'a> {
    tool: &'a ToolSpec,
    values: &'a Value,
}

impl<'a> Params<'a> {
    /// Wraps the values of a call to a tool.
    pub fn new(tool: &'a ToolSpec, values: &'a Value) -> Result<Self, ToolError> {
        if matches!(values, Value::Object(_)) {
            Ok(Self { tool, values })
        } else {
            Err(ToolError::general(
                "the request body must be a JSON object of parameters",
            ))
        }
    }

    /// Returns a required number.
    ///
    /// A value may arrive as a JSON number or as a string, because a form
    /// control sends text. An absent member falls back to the default
    /// declared by the parameter, and only a parameter with neither a value
    /// nor a default is reported as required.
    pub fn number(&self, name: &str) -> Result<f64, ToolError> {
        let spec = self.spec(name)?;
        let text = match self.raw(name) {
            Some(Value::Number(number)) => {
                if number.is_finite() {
                    return self.check_number(spec, *number);
                }
                return Err(ToolError::parameter(spec.name, "must be a finite number"));
            }
            Some(Value::String(text)) => text.trim().to_string(),
            Some(_) => {
                return Err(ToolError::parameter(spec.name, "must be a number"));
            }
            None => match spec.default {
                Some(default) => default.to_string(),
                None => return Err(ToolError::parameter(spec.name, "is required")),
            },
        };
        if text.is_empty() {
            return match spec.default {
                Some(default) => {
                    let value: f64 = default
                        .parse()
                        .map_err(|_| ToolError::parameter(spec.name, "is not a number"))?;
                    self.check_number(spec, value)
                }
                None => Err(ToolError::parameter(spec.name, "is required")),
            };
        }
        let value: f64 = text
            .parse()
            .map_err(|_| ToolError::parameter(spec.name, "must be a number"))?;
        self.check_number(spec, value)
    }

    /// Returns a required integer.
    ///
    /// A value that is not a whole number is rejected rather than rounded,
    /// because rounding would silently change a field such as a year.
    pub fn integer(&self, name: &str) -> Result<i64, ToolError> {
        let value = self.number(name)?;
        if value.fract() != 0.0 {
            let spec = self.spec(name)?;
            return Err(ToolError::parameter(spec.name, "must be a whole number"));
        }
        Ok(value as i64)
    }

    /// Returns a required string.
    pub fn text(&self, name: &str) -> Result<String, ToolError> {
        let spec = self.spec(name)?;
        match self.raw(name) {
            Some(Value::String(text)) if !text.trim().is_empty() => Ok(text.clone()),
            Some(Value::String(_)) | None => match spec.default {
                Some(default) => Ok(default.to_string()),
                None if spec.required => Err(ToolError::parameter(spec.name, "is required")),
                None => Ok(String::new()),
            },
            Some(_) => Err(ToolError::parameter(spec.name, "must be text")),
        }
    }

    /// Returns a value that must be one of the options of a select
    /// parameter.
    pub fn choice(&self, name: &str) -> Result<&'static str, ToolError> {
        let spec = self.spec(name)?;
        let accepted = match spec.kind {
            ParamKind::Select(options) => options,
            _ => {
                return Err(ToolError::parameter(
                    spec.name,
                    "is not an enumerated parameter",
                ));
            }
        };
        let text = self.text(name)?;
        let lowered = text.trim().to_ascii_lowercase();
        accepted
            .iter()
            .find(|option| option.value == lowered)
            .map(|option| option.value)
            .ok_or_else(|| {
                let names: Vec<&str> = accepted.iter().map(|option| option.value).collect();
                ToolError::parameter(spec.name, format!("must be one of {}", names.join(", ")))
            })
    }

    /// Returns the raw value of a parameter.
    fn raw(&self, name: &str) -> Option<&Value> {
        self.values.get(name)
    }

    /// Returns the specification of a parameter.
    fn spec(&self, name: &str) -> Result<&'a ParamSpec, ToolError> {
        self.tool
            .params
            .iter()
            .find(|param| param.name == name)
            .ok_or_else(|| ToolError::general(format!("unknown parameter {name:?}")))
    }

    /// Validates a number against the bounds of its specification.
    fn check_number(&self, spec: &ParamSpec, value: f64) -> Result<f64, ToolError> {
        if !value.is_finite() {
            return Err(ToolError::parameter(spec.name, "must be a finite number"));
        }
        if let ParamKind::Number { min, max, .. } = spec.kind {
            if let Some(min) = min {
                if value < min {
                    return Err(ToolError::parameter(
                        spec.name,
                        format!("must be at least {min}"),
                    ));
                }
            }
            if let Some(max) = max {
                if value > max {
                    return Err(ToolError::parameter(
                        spec.name,
                        format!("must be at most {max}"),
                    ));
                }
            }
        }
        Ok(value)
    }
}

/// Executes a tool by identifier.
///
/// The registry resolves the identifier, the parameters are validated, and
/// the handler runs. An unknown identifier is reported as a general failure
/// so that the HTTP layer can answer with a not found status.
pub fn execute(registry: &Registry, id: &str, values: &Value) -> Result<Value, ToolError> {
    let tool = registry
        .get(id)
        .ok_or_else(|| ToolError::general(format!("unknown tool {id:?}")))?;
    let params = Params::new(tool, values)?;
    handlers::run(tool.id, &params)
}

/// Encodes the registry as JSON for the frontend.
pub fn registry_json(registry: &Registry) -> Value {
    let tools: Vec<Value> = registry.entries().iter().map(tool_json).collect();
    json_object! {
        "categories" => registry.categories().iter().map(|name| Value::from(*name)).collect::<Vec<Value>>(),
        "tools" => tools,
    }
}

/// Encodes one tool as JSON.
fn tool_json(tool: &ToolSpec) -> Value {
    let params: Vec<Value> = tool.params.iter().map(param_json).collect();
    json_object! {
        "id" => tool.id,
        "name" => tool.name,
        "summary" => tool.summary,
        "description" => tool.description,
        "category" => tool.category,
        "params" => params,
        "api_items" => tool.api_items.iter().map(|name| Value::from(*name)).collect::<Vec<Value>>(),
    }
}

/// Encodes one parameter as JSON.
fn param_json(param: &ParamSpec) -> Value {
    let (kind, options) = match &param.kind {
        ParamKind::Number { unit, min, max } => (
            "number",
            json_object! {
                "unit" => *unit,
                "min" => min.map(Value::from).unwrap_or(Value::Null),
                "max" => max.map(Value::from).unwrap_or(Value::Null),
            },
        ),
        ParamKind::Integer { min, max } => (
            "integer",
            json_object! {
                "min" => min.map(|value| Value::from(value as f64)).unwrap_or(Value::Null),
                "max" => max.map(|value| Value::from(value as f64)).unwrap_or(Value::Null),
            },
        ),
        ParamKind::Multiline => ("multiline", Value::Object(Vec::new())),
        ParamKind::Select(choices) => (
            "select",
            json_object! {
                "options" => choices
                    .iter()
                    .map(|option| json_object! { "value" => option.value, "label" => option.label })
                    .collect::<Vec<Value>>(),
            },
        ),
    };
    json_object! {
        "name" => param.name,
        "label" => param.label,
        "help" => param.help,
        "kind" => kind,
        "required" => param.required,
        "default" => param.default.map(Value::from).unwrap_or(Value::Null),
        "constraints" => options,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_is_populated() {
        let registry = Registry::new();
        assert!(registry.entries().len() >= 7);
        assert!(registry.get("parse-tle").is_some());
        assert!(registry.get("sgp4-propagate").is_some());
        assert!(registry.get("missing").is_none());
    }

    #[test]
    fn registry_json_lists_tools_and_categories() {
        let value = registry_json(&Registry::new());
        assert!(!value.get("tools").unwrap().as_array().is_empty());
        assert!(!value.get("categories").unwrap().as_array().is_empty());
    }

    #[test]
    fn number_parameters_honour_defaults_and_bounds() {
        let registry = Registry::new();
        let tool = registry.get("sgp4-propagate").unwrap();
        let values = json_object! {};
        let params = Params::new(tool, &values).unwrap();
        assert_eq!(params.number("minutes").unwrap(), 0.0);
    }

    #[test]
    fn rejects_out_of_range_values() {
        let registry = Registry::new();
        let tool = registry.get("sgp4-propagate").unwrap();
        let values = json_object! { "minutes" => 5_000_000.0 };
        let params = Params::new(tool, &values).unwrap();
        let error = params.number("minutes").unwrap_err();
        assert_eq!(error.parameter, Some("minutes"));
    }

    #[test]
    fn select_parameters_reject_unknown_choices() {
        let registry = Registry::new();
        let tool = registry.get("sgp4-propagate").unwrap();
        let values = json_object! { "model" => "wgs99" };
        let params = Params::new(tool, &values).unwrap();
        assert!(params.choice("model").is_err());
        let values = json_object! { "model" => "WGS84" };
        let params = Params::new(tool, &values).unwrap();
        assert_eq!(params.choice("model").unwrap(), "wgs84");
    }

    #[test]
    fn string_numbers_are_accepted() {
        let registry = Registry::new();
        let tool = registry.get("sgp4-propagate").unwrap();
        let values = json_object! { "minutes" => "12.5" };
        let params = Params::new(tool, &values).unwrap();
        assert_eq!(params.number("minutes").unwrap(), 12.5);
    }
}
