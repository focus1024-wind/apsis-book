//! The tool handlers.
//!
//! Each handler receives validated parameters and calls the `apsis-rs`
//! public interface. The result is shaped by [`super::out`] so that the
//! frontend renders it with the same components for every tool.
//!
//! Errors are reported through [`ToolError`] with a message written for the
//! user of the form. The underlying error types of `apsis-rs` describe the
//! condition, which is what a caller with the input in front of them needs;
//! the handler adds the parameter name so that the form can highlight the
//! field.

use apsis::apsis_coord::{Ecef, Eci, Geodetic};
use apsis::apsis_frames::EarthRotationAngle;
use apsis::apsis_kepler::{GravityParameter, KeplerianElements};
use apsis::apsis_orbit::Orbit;
use apsis::apsis_sgp4::GravityModel;
use apsis::apsis_time::{Calendar, Epoch, TimeScale};
use apsis::apsis_tle::TwoLineElement;

use crate::json::Value;
use crate::json_object;
use crate::tools::out::{self, Column, Scalar, Series, Table};
use crate::tools::{Params, ToolError};

/// Runs a tool by identifier.
///
/// The identifier is one of the identifiers in the registry; an unknown one
/// is reported as a general failure.
pub fn run(id: &str, params: &Params<'_>) -> Result<Value, ToolError> {
    match id {
        "parse-tle" => parse_tle(params),
        "sgp4-propagate" => sgp4_propagate(params),
        "sgp4-track" => sgp4_track(params),
        "orbit-osculating" => orbit_osculating(params),
        "kepler-state" => kepler_state(params),
        "time-convert" => time_convert(params),
        "geodetic-ecef" => geodetic_ecef(params),
        "frame-transform" => frame_transform(params),
        "hohmann-transfer" => hohmann_transfer(params),
        other => Err(ToolError::general(format!("unknown tool {other:?}"))),
    }
}

/// Parses a two-line element set and reports its fields.
fn parse_tle(params: &Params<'_>) -> Result<Value, ToolError> {
    let text = params.text("tle")?;
    let element = TwoLineElement::parse(&text).map_err(|error| {
        ToolError::parameter("tle", format!("the element set is not valid: {error}"))
    })?;

    let mut scalars = vec![
        Scalar::plain(
            "Name",
            element
                .name
                .clone()
                .unwrap_or_else(|| "(not given)".to_string()),
        ),
        Scalar::plain("Catalog number", element.satellite_number).monospace(true),
        Scalar::plain("Classification", element.classification.as_char()).monospace(true),
        Scalar::plain(
            "International designator",
            element.international_designator.clone(),
        )
        .monospace(true),
        Scalar::plain("Element set number", element.element_set_number).monospace(true),
        Scalar::plain("Revolution number", element.revolution_number).monospace(true),
        Scalar::with_unit("Epoch", format_calendar(element.epoch), "UTC"),
    ];

    let mut table = Table::new(vec![
        Column::text("field", "Field"),
        Column::number("value", "Value", ""),
        Column::text("unit", "Unit"),
        Column::text("meaning", "Meaning"),
    ]);
    let mean_motion = element.mean_motion;
    let period_minutes = if mean_motion > 0.0 {
        1_440.0 / mean_motion
    } else {
        f64::NAN
    };
    for (field, value, unit, meaning) in [
        ("mean_motion", element.mean_motion, "rev/day", "Mean motion"),
        (
            "mean_motion_dot",
            element.mean_motion_dot,
            "rev/day^2",
            "First derivative of mean motion, divided by two",
        ),
        (
            "mean_motion_double_dot",
            element.mean_motion_double_dot,
            "rev/day^3",
            "Second derivative of mean motion, divided by six",
        ),
        ("bstar", element.bstar, "1/earth radius", "SGP4 drag term"),
        ("inclination", element.inclination, "deg", "Inclination"),
        (
            "right_ascension",
            element.right_ascension,
            "deg",
            "Right ascension of the ascending node",
        ),
        ("eccentricity", element.eccentricity, "", "Eccentricity"),
        (
            "argument_of_perigee",
            element.argument_of_perigee,
            "deg",
            "Argument of perigee",
        ),
        ("mean_anomaly", element.mean_anomaly, "deg", "Mean anomaly"),
        (
            "period",
            period_minutes,
            "min",
            "Orbital period implied by the mean motion",
        ),
    ] {
        let value = if value.is_finite() { value } else { f64::NAN };
        table.push(vec![
            Value::from(field),
            Value::from(value),
            Value::from(unit),
            Value::from(meaning),
        ]);
    }

    scalars.push(Scalar::with_unit(
        "Orbital period",
        out::fmt(period_minutes, 3),
        "min",
    ));
    scalars.push(Scalar::with_unit(
        "Semi-major axis estimate",
        out::fmt(semi_major_axis_estimate(element.mean_motion), 3),
        "km",
    ));

    Ok(out::table_response(
        "element-set",
        "Two-line element set",
        scalars,
        &table,
        Vec::new(),
        vec![
            "Angles are reported in the units the element set uses, which are degrees for the orientation angles.".to_string(),
            "The semi-major axis estimate follows from the mean motion and Kepler's third law with the WGS 72 gravitational parameter; it ignores the perturbations the propagator models.".to_string(),
        ],
    ))
}

/// Propagates one instant with SGP4 and reports the state.
fn sgp4_propagate(params: &Params<'_>) -> Result<Value, ToolError> {
    let text = params.text("tle")?;
    let model = gravity_model(params.choice("model")?);
    let minutes = params.number("minutes")?;

    let element = TwoLineElement::parse(&text).map_err(|error| {
        ToolError::parameter("tle", format!("the element set is not valid: {error}"))
    })?;
    let satellite =
        apsis::apsis_sgp4::Satellite::try_from_element(&element, model).map_err(|error| {
            ToolError::parameter("tle", format!("the elements cannot be propagated: {error}"))
        })?;
    let state = satellite.propagate_minutes(minutes).map_err(|error| {
        ToolError::parameter("minutes", format!("the propagation failed: {error}"))
    })?;

    let branch = if satellite.is_deep_space() {
        "SDP4"
    } else {
        "SGP4"
    };
    let radius = state.altitude_radius();
    let scalars = vec![
        Scalar::plain("Branch", branch).monospace(true),
        Scalar::with_unit("Minutes from epoch", out::fmt(minutes, 3), "min"),
        Scalar::with_unit("Position X", out::fmt(state.position.x, 6), "km"),
        Scalar::with_unit("Position Y", out::fmt(state.position.y, 6), "km"),
        Scalar::with_unit("Position Z", out::fmt(state.position.z, 6), "km"),
        Scalar::with_unit("Velocity X", out::fmt(state.velocity.x, 9), "km/s"),
        Scalar::with_unit("Velocity Y", out::fmt(state.velocity.y, 9), "km/s"),
        Scalar::with_unit("Velocity Z", out::fmt(state.velocity.z, 9), "km/s"),
        Scalar::with_unit("Radius", out::fmt(radius, 6), "km"),
        Scalar::with_unit(
            "Altitude above the equatorial radius",
            out::fmt(radius - model.radius, 3),
            "km",
        ),
        Scalar::with_unit("Speed", out::fmt(state.speed(), 9), "km/s"),
        Scalar::with_unit("Semi-major axis", out::fmt(state.semi_major_axis, 6), "km"),
    ];

    let mut table = Table::new(vec![
        Column::text("component", "Component"),
        Column::number("value", "Value", ""),
        Column::text("unit", "Unit"),
    ]);
    for (component, value, unit) in [
        ("position.x", state.position.x, "km"),
        ("position.y", state.position.y, "km"),
        ("position.z", state.position.z, "km"),
        ("velocity.x", state.velocity.x, "km/s"),
        ("velocity.y", state.velocity.y, "km/s"),
        ("velocity.z", state.velocity.z, "km/s"),
        ("radius", radius, "km"),
        ("speed", state.speed(), "km/s"),
        ("semi_major_axis", state.semi_major_axis, "km"),
    ] {
        table.push(vec![
            Value::from(component),
            Value::from(value),
            Value::from(unit),
        ]);
    }

    Ok(out::table_response(
        "state-vector",
        "Propagated state",
        scalars,
        &table,
        Vec::new(),
        vec![
            "The frame is the True Equator Mean Equinox frame, which is the frame the model defines.".to_string(),
            format!("The gravity model used is {}.", gravity_model_name(params.choice("model")?)),
        ],
    ))
}

/// Samples a ground track and returns the points.
fn sgp4_track(params: &Params<'_>) -> Result<Value, ToolError> {
    let text = params.text("tle")?;
    let model = gravity_model(params.choice("model")?);
    let start = params.number("start_minutes")?;
    let end = params.number("end_minutes")?;
    let step = params.number("step_minutes")?;

    if end <= start {
        return Err(ToolError::parameter(
            "end_minutes",
            "must be later than the start",
        ));
    }
    let count = ((end - start) / step).floor() as i64 + 1;
    if count > 2_000 {
        return Err(ToolError::parameter(
            "step_minutes",
            "would produce more than two thousand samples; increase the step or shorten the range",
        ));
    }

    let element = TwoLineElement::parse(&text).map_err(|error| {
        ToolError::parameter("tle", format!("the element set is not valid: {error}"))
    })?;
    let orbit = Orbit::try_from_element(&element, model).map_err(|error| {
        ToolError::parameter("tle", format!("the elements cannot be propagated: {error}"))
    })?;
    let ephemeris = orbit.sample_ephemeris(start, end, step).map_err(|error| {
        ToolError::parameter(
            "step_minutes",
            format!("the track could not be sampled: {error}"),
        )
    })?;

    let mut table = Table::new(vec![
        Column::number("minutes", "Minutes", "min"),
        Column::number("x", "X", "km"),
        Column::number("y", "Y", "km"),
        Column::number("z", "Z", "km"),
        Column::number("radius", "Radius", "km"),
        Column::number("speed", "Speed", "km/s"),
    ]);
    let mut radii = Vec::new();
    let mut speeds = Vec::new();
    let mut times = Vec::new();
    for sample in ephemeris.samples() {
        let minutes = sample.seconds / 60.0;
        times.push(minutes);
        radii.push(sample.radius());
        speeds.push(sample.speed());
        table.push(vec![
            Value::from(minutes),
            Value::from(sample.position.x),
            Value::from(sample.position.y),
            Value::from(sample.position.z),
            Value::from(sample.radius()),
            Value::from(sample.speed()),
        ]);
    }

    let (minimum, maximum) = min_max(&radii);
    let scalars = vec![
        Scalar::plain("Samples", table.rows.len()).monospace(true),
        Scalar::with_unit("Duration", out::fmt(end - start, 3), "min"),
        Scalar::with_unit("Lowest radius", out::fmt(minimum, 3), "km"),
        Scalar::with_unit("Highest radius", out::fmt(maximum, 3), "km"),
        Scalar::with_unit(
            "Altitude above the equatorial radius at the lowest point",
            out::fmt(minimum - model.radius, 3),
            "km",
        ),
    ];

    let series = vec![
        Series::new("Radius", "km", radii),
        Series::new("Speed", "km/s", speeds),
    ];

    Ok(out::table_response(
        "track",
        "Sampled ground track",
        scalars,
        &table,
        series,
        vec![
            "Each sample is one propagation of the mean elements; the table is not interpolated.".to_string(),
            "The track is given in the True Equator Mean Equinox frame, so a ground projection needs the Earth rotation in addition.".to_string(),
            format!("Sample times are minutes from the element epoch, starting at {}.", out::fmt(start, 3)),
        ],
    ))
    .map(|value| {
        // The time axis is carried separately so that the chart can label
        // the abscissa without reading the table.
        let mut members = value.members().to_vec();
        members.push((
            "axis".to_string(),
            json_object! {
                "name" => "Minutes from epoch",
                "unit" => "min",
                "values" => times.into_iter().map(Value::from).collect::<Vec<Value>>(),
            },
        ));
        Value::Object(members)
    })
}

/// Compares the mean elements with the osculating elements.
fn orbit_osculating(params: &Params<'_>) -> Result<Value, ToolError> {
    let text = params.text("tle")?;
    let model = gravity_model(params.choice("model")?);
    let minutes = params.number("minutes")?;

    let element = TwoLineElement::parse(&text).map_err(|error| {
        ToolError::parameter("tle", format!("the element set is not valid: {error}"))
    })?;
    let orbit = Orbit::try_from_element(&element, model).map_err(|error| {
        ToolError::parameter("tle", format!("the elements cannot be propagated: {error}"))
    })?;
    let osculating = orbit.osculating_elements(minutes).map_err(|error| {
        ToolError::parameter("minutes", format!("the conversion failed: {error}"))
    })?;

    let mean_motion_radians = if element.mean_motion > 0.0 {
        element.mean_motion * 2.0 * std::f64::consts::PI / 86_400.0
    } else {
        0.0
    };
    let mean_semi_major_axis = if mean_motion_radians > 0.0 {
        (orbit.gravity_parameter().value() / (mean_motion_radians * mean_motion_radians)).cbrt()
    } else {
        f64::NAN
    };

    let mut table = Table::new(vec![
        Column::text("element", "Element"),
        Column::number("mean", "Mean", ""),
        Column::number("osculating", "Osculating", ""),
        Column::text("unit", "Unit"),
    ]);
    let rows = [
        (
            "semi_major_axis",
            mean_semi_major_axis,
            osculating.semi_major_axis,
            "km",
        ),
        (
            "eccentricity",
            element.eccentricity,
            osculating.eccentricity,
            "",
        ),
        (
            "inclination",
            element.inclination,
            out::degrees(osculating.inclination),
            "deg",
        ),
        (
            "right_ascension",
            element.right_ascension,
            out::degrees(osculating.right_ascension),
            "deg",
        ),
        (
            "argument_of_perigee",
            element.argument_of_perigee,
            out::degrees(osculating.argument_of_perigee),
            "deg",
        ),
        (
            "true_anomaly",
            element.mean_anomaly,
            out::degrees(osculating.true_anomaly),
            "deg",
        ),
    ];
    for (name, mean, osculating, unit) in rows {
        table.push(vec![
            Value::from(name),
            Value::from(mean),
            Value::from(osculating),
            Value::from(unit),
        ]);
    }

    let scalars = vec![
        Scalar::with_unit("Minutes from epoch", out::fmt(minutes, 3), "min"),
        Scalar::with_unit(
            "Semi-major axis change",
            out::fmt(osculating.semi_major_axis - mean_semi_major_axis, 3),
            "km",
        ),
        Scalar::with_unit(
            "Eccentricity change",
            out::fmt(osculating.eccentricity - element.eccentricity, 6),
            "",
        ),
        Scalar::with_unit(
            "Inclination change",
            out::fmt(
                out::degrees(osculating.inclination) - element.inclination,
                6,
            ),
            "deg",
        ),
    ];

    Ok(out::table_response(
        "comparison",
        "Mean and osculating elements",
        scalars,
        &table,
        Vec::new(),
        vec![
            "The mean elements are the ones written in the element set and are the inputs of the propagator.".to_string(),
            "The osculating elements describe the two-body orbit that matches the instantaneous state; they are an output of the propagation, not an input.".to_string(),
            "The mean semi-major axis follows from the mean motion with Kepler's third law, which is an approximation the propagator refines.".to_string(),
            "The row labelled true_anomaly compares the mean anomaly of the element set with the true anomaly of the osculating elements; the two are different angles and the comparison shows the difference between them.".to_string(),
        ],
    ))
}

/// Converts Keplerian elements to a state vector.
fn kepler_state(params: &Params<'_>) -> Result<Value, ToolError> {
    let parameter = gravity_parameter(params.choice("model")?);
    let elements = KeplerianElements {
        semi_major_axis: params.number("semi_major_axis")?,
        eccentricity: params.number("eccentricity")?,
        inclination: out::radians(params.number("inclination")?),
        right_ascension: out::radians(params.number("right_ascension")?),
        argument_of_perigee: out::radians(params.number("argument_of_perigee")?),
        true_anomaly: out::radians(params.number("true_anomaly")?),
    };
    if !elements.is_valid() {
        return Err(ToolError::general(
            "the elements do not describe an elliptical orbit; check that the semi-major axis is positive and the eccentricity is below one",
        ));
    }
    let state = elements
        .to_state(parameter)
        .ok_or_else(|| ToolError::general("the elements could not be converted"))?;

    let period = elements.period(parameter);
    let scalars = vec![
        Scalar::with_unit("Position X", out::fmt(state.position.x, 6), "km"),
        Scalar::with_unit("Position Y", out::fmt(state.position.y, 6), "km"),
        Scalar::with_unit("Position Z", out::fmt(state.position.z, 6), "km"),
        Scalar::with_unit("Velocity X", out::fmt(state.velocity.x, 9), "km/s"),
        Scalar::with_unit("Velocity Y", out::fmt(state.velocity.y, 9), "km/s"),
        Scalar::with_unit("Velocity Z", out::fmt(state.velocity.z, 9), "km/s"),
        Scalar::with_unit("Radius", out::fmt(state.position.norm(), 6), "km"),
        Scalar::with_unit("Speed", out::fmt(state.velocity.norm(), 9), "km/s"),
        Scalar::with_unit(
            "Perigee radius",
            out::fmt(elements.perigee_radius(), 6),
            "km",
        ),
        Scalar::with_unit("Apogee radius", out::fmt(elements.apogee_radius(), 6), "km"),
        Scalar::with_unit("Period", out::fmt(period, 3), "s"),
        Scalar::with_unit("Period", out::fmt(period / 60.0, 3), "min"),
        Scalar::with_unit(
            "Semi-latus rectum",
            out::fmt(elements.semi_latus_rectum(), 6),
            "km",
        ),
        Scalar::with_unit(
            "Semi-minor axis",
            out::fmt(elements.semi_minor_axis(), 6),
            "km",
        ),
    ];

    let mut table = Table::new(vec![
        Column::text("quantity", "Quantity"),
        Column::number("value", "Value", ""),
        Column::text("unit", "Unit"),
    ]);
    for (name, value, unit) in [
        ("position.x", state.position.x, "km"),
        ("position.y", state.position.y, "km"),
        ("position.z", state.position.z, "km"),
        ("velocity.x", state.velocity.x, "km/s"),
        ("velocity.y", state.velocity.y, "km/s"),
        ("velocity.z", state.velocity.z, "km/s"),
        ("semi_major_axis", elements.semi_major_axis, "km"),
        ("eccentricity", elements.eccentricity, ""),
        ("inclination", out::degrees(elements.inclination), "deg"),
        (
            "right_ascension",
            out::degrees(elements.right_ascension),
            "deg",
        ),
        (
            "argument_of_perigee",
            out::degrees(elements.argument_of_perigee),
            "deg",
        ),
        ("true_anomaly", out::degrees(elements.true_anomaly), "deg"),
        (
            "eccentric_anomaly",
            out::degrees(elements.eccentric_anomaly()),
            "deg",
        ),
        ("mean_anomaly", out::degrees(elements.mean_anomaly()), "deg"),
    ] {
        table.push(vec![
            Value::from(name),
            Value::from(value),
            Value::from(unit),
        ]);
    }

    Ok(out::table_response(
        "state-vector",
        "Keplerian elements to a state vector",
        scalars,
        &table,
        Vec::new(),
        vec![
            "The conversion is the two-body solution; it does not include the perturbations that SGP4 models.".to_string(),
            format!("The gravitational parameter is {}.", gravity_model_name(params.choice("model")?)),
        ],
    ))
}

/// Converts an instant between time scales.
fn time_convert(params: &Params<'_>) -> Result<Value, ToolError> {
    let scale = time_scale(params.choice("scale")?);
    let form = params.choice("input")?;

    let epoch = if form == "mjd" {
        let mjd = params.number("mjd")?;
        Epoch::from_mjd(mjd, scale).ok_or_else(|| {
            ToolError::parameter(
                "mjd",
                "the instant cannot be expressed on the requested scale; the leap second table starts in 1972",
            )
        })?
    } else {
        let calendar = Calendar::new(
            params.integer("year")? as i32,
            params.integer("month")? as u32,
            params.integer("day")? as u32,
            params.integer("hour")? as u32,
            params.integer("minute")? as u32,
            params.number("second")?,
        );
        if !calendar.is_valid() {
            return Err(ToolError::general(
                "the calendar fields do not form a valid date and time",
            ));
        }
        Epoch::from_calendar(calendar, scale).ok_or_else(|| {
            ToolError::general(
                "the instant cannot be expressed on the requested scale; the leap second table starts in 1972",
            )
        })?
    };

    let mut table = Table::new(vec![
        Column::text("scale", "Scale"),
        Column::number("mjd", "Modified Julian Date", "day"),
        Column::text("calendar", "Calendar date"),
        Column::number("offset", "Offset from TAI", "s"),
    ]);
    let mut scalars = Vec::new();
    for target in [
        TimeScale::Utc,
        TimeScale::Tai,
        TimeScale::Tt,
        TimeScale::Gps,
    ] {
        let mjd = epoch.mjd(target);
        let calendar = epoch.calendar(target);
        let offset = epoch.tai_mjd() - mjd.unwrap_or(f64::NAN);
        if target == scale {
            if let Some(mjd) = mjd {
                scalars.push(Scalar::with_unit(
                    format!("Input on {}", target.name()),
                    out::fmt(mjd, 9),
                    "MJD",
                ));
            }
        }
        table.push(vec![
            Value::from(target.name()),
            mjd.map(Value::from).unwrap_or(Value::Null),
            match calendar {
                Some(calendar) => Value::from(format_calendar_text(calendar)),
                None => Value::Null,
            },
            Value::from(offset * 86_400.0),
        ]);
    }

    let tai = epoch.tai_mjd();
    let utc = epoch.mjd(TimeScale::Utc);
    if let Some(utc) = utc {
        scalars.push(Scalar::with_unit(
            "TAI minus UTC at this instant",
            out::fmt((tai - utc) * 86_400.0, 3),
            "s",
        ));
    }

    let mut notes = vec![
        "The instant is stored as a Modified Julian Date on the TAI scale, so the difference between two instants is exact elapsed time.".to_string(),
        "UTC is not a uniform scale; the offset column is the difference between TAI and the scale at that instant.".to_string(),
    ];
    if utc.is_none() {
        notes.push(
            "This instant precedes the start of the leap second table, so it has no UTC representation.".to_string(),
        );
    }

    Ok(out::table_response(
        "time-scales",
        "Time scale conversion",
        scalars,
        &table,
        Vec::new(),
        notes,
    ))
}

/// Converts a geodetic position to Earth-fixed coordinates.
fn geodetic_ecef(params: &Params<'_>) -> Result<Value, ToolError> {
    let latitude = out::radians(params.number("latitude")?);
    let longitude = out::radians(params.number("longitude")?);
    let height = params.number("height")?;

    let geodetic = Geodetic::new(latitude, longitude, height);
    let ecef = geodetic.to_ecef();
    let vector = ecef.vector();

    let ellipsoid = apsis::apsis_geodesy::WGS84;
    let prime_vertical = ellipsoid.prime_vertical_radius(latitude);
    let meridian = ellipsoid.meridian_radius(latitude);
    let geocentric = vector.y.atan2(vector.x).abs();
    let geocentric_latitude = (vector.z / vector.norm()).asin();

    let scalars = vec![
        Scalar::with_unit("X", out::fmt(vector.x, 6), "m"),
        Scalar::with_unit("Y", out::fmt(vector.y, 6), "m"),
        Scalar::with_unit("Z", out::fmt(vector.z, 6), "m"),
        Scalar::with_unit("Geocentric radius", out::fmt(vector.norm(), 6), "m"),
        Scalar::with_unit(
            "Prime vertical radius of curvature",
            out::fmt(prime_vertical, 6),
            "m",
        ),
        Scalar::with_unit("Meridian radius of curvature", out::fmt(meridian, 6), "m"),
        Scalar::with_unit(
            "Geocentric latitude",
            out::fmt(out::degrees(geocentric_latitude), 9),
            "deg",
        ),
        Scalar::with_unit(
            "Geodetic minus geocentric latitude",
            out::fmt((latitude - geocentric_latitude).to_degrees(), 9),
            "deg",
        ),
    ];

    let mut table = Table::new(vec![
        Column::text("quantity", "Quantity"),
        Column::number("value", "Value", ""),
        Column::text("unit", "Unit"),
    ]);
    for (name, value, unit) in [
        ("x", vector.x, "m"),
        ("y", vector.y, "m"),
        ("z", vector.z, "m"),
        ("longitude", longitude, "rad"),
        ("latitude", latitude, "rad"),
        ("height", height, "m"),
        ("prime_vertical_radius", prime_vertical, "m"),
        ("meridian_radius", meridian, "m"),
        ("geocentric_latitude", geocentric_latitude, "rad"),
    ] {
        table.push(vec![
            Value::from(name),
            Value::from(value),
            Value::from(unit),
        ]);
    }

    let _ = geocentric;
    Ok(out::table_response(
        "geodetic",
        "Geodetic position to Earth-fixed coordinates",
        scalars,
        &table,
        Vec::new(),
        vec![
            "The ellipsoid is WGS 84; its semi-major axis and inverse flattening are the defining parameters.".to_string(),
            "Latitude is geodetic, measured from the ellipsoid normal, not geocentric; the difference is largest at mid latitudes.".to_string(),
        ],
    ))
}

/// Rotates a vector between the inertial and Earth-fixed frames.
fn frame_transform(params: &Params<'_>) -> Result<Value, ToolError> {
    let direction = params.choice("direction")?;
    let vector = apsis::apsis_math::Vec3::new(
        params.number("x")?,
        params.number("y")?,
        params.number("z")?,
    );
    let angle = EarthRotationAngle::from_turns(params.number("earth_rotation")?);

    let (output, from, to) = if direction == "eci-to-ecef" {
        (Eci::new(vector).to_ecef(angle).vector(), "ECI", "ECEF")
    } else {
        (Ecef::new(vector).to_eci(angle).vector(), "ECEF", "ECI")
    };

    let input_radius = vector.norm();
    let output_radius = output.norm();
    let scalars = vec![
        Scalar::plain("Direction", format!("{from} to {to}")).monospace(true),
        Scalar::with_unit("Input X", out::fmt(vector.x, 6), "m"),
        Scalar::with_unit("Input Y", out::fmt(vector.y, 6), "m"),
        Scalar::with_unit("Input Z", out::fmt(vector.z, 6), "m"),
        Scalar::with_unit("Output X", out::fmt(output.x, 6), "m"),
        Scalar::with_unit("Output Y", out::fmt(output.y, 6), "m"),
        Scalar::with_unit("Output Z", out::fmt(output.z, 6), "m"),
        Scalar::with_unit("Input radius", out::fmt(input_radius, 6), "m"),
        Scalar::with_unit("Output radius", out::fmt(output_radius, 6), "m"),
        Scalar::with_unit(
            "Radius change",
            out::fmt(output_radius - input_radius, 12),
            "m",
        ),
        Scalar::with_unit(
            "Earth Rotation Angle",
            out::fmt(angle.radians_positive().to_degrees(), 9),
            "deg",
        ),
        Scalar::with_unit(
            "Earth Rotation Angle",
            out::fmt(angle.turns_positive(), 12),
            "turn",
        ),
    ];

    let mut table = Table::new(vec![
        Column::text("component", "Component"),
        Column::number("input", "Input", "m"),
        Column::number("output", "Output", "m"),
    ]);
    for (name, before, after) in [
        ("x", vector.x, output.x),
        ("y", vector.y, output.y),
        ("z", vector.z, output.z),
    ] {
        table.push(vec![
            Value::from(name),
            Value::from(before),
            Value::from(after),
        ]);
    }

    Ok(out::table_response(
        "rotation",
        "Frame rotation",
        scalars,
        &table,
        Vec::new(),
        vec![
            "The rotation is about the third axis by the Earth Rotation Angle and preserves the length of the vector.".to_string(),
            "Passing an Earth-fixed position through the inertial frame and back returns the Earth-fixed position; the angle is not reduced to one turn.".to_string(),
        ],
    ))
}

/// Plans a Hohmann transfer between two circular orbits.
///
/// The transfer is the classic two impulse maneuver: a burn onto an ellipse
/// whose perigee touches the lower orbit and whose apogee touches the upper
/// one, a coast for half the ellipse's period, and a second burn onto the
/// destination circle.
fn hohmann_transfer(params: &Params<'_>) -> Result<Value, ToolError> {
    let from_radius = params.number("from_radius")?;
    let to_radius = params.number("to_radius")?;
    let parameter = gravity_parameter(params.choice("model")?);

    if from_radius == to_radius {
        return Err(ToolError::parameter(
            "to_radius",
            "must differ from the starting radius, because a transfer to the same orbit is not a maneuver",
        ));
    }

    let transfer = apsis::apsis_maneuver::hohmann_transfer(from_radius, to_radius, parameter)
        .map_err(|error| {
            ToolError::general(format!("the transfer could not be planned: {error}"))
        })?;

    let direction = if transfer.is_raising() {
        "raising"
    } else {
        "lowering"
    };
    let scalars = vec![
        Scalar::plain("Direction", direction),
        Scalar::with_unit(
            "First burn",
            out::fmt(transfer.first_burn_magnitude(), 6),
            "km/s",
        ),
        Scalar::with_unit(
            "Second burn",
            out::fmt(transfer.second_burn_magnitude(), 6),
            "km/s",
        ),
        Scalar::with_unit(
            "Total velocity change",
            out::fmt(transfer.total_delta_v(), 6),
            "km/s",
        ),
        Scalar::with_unit(
            "Transfer semi-major axis",
            out::fmt(transfer.transfer_semi_major_axis, 6),
            "km",
        ),
        Scalar::with_unit("Transfer time", out::fmt(transfer.transfer_time, 3), "s"),
        Scalar::with_unit(
            "Transfer time",
            out::fmt(transfer.transfer_time / 60.0, 3),
            "min",
        ),
        Scalar::with_unit(
            "Transfer time",
            out::fmt(transfer.transfer_time / 86_400.0, 6),
            "day",
        ),
    ];

    let mut table = Table::new(vec![
        Column::text("burn", "Burn"),
        Column::number("signed", "Signed", "km/s"),
        Column::number("magnitude", "Magnitude", "km/s"),
    ]);
    table.push(vec![
        Value::from("first"),
        Value::from(transfer.first_burn),
        Value::from(transfer.first_burn_magnitude()),
    ]);
    table.push(vec![
        Value::from("second"),
        Value::from(transfer.second_burn),
        Value::from(transfer.second_burn_magnitude()),
    ]);

    Ok(out::table_response(
        "maneuver",
        "Hohmann transfer",
        scalars,
        &table,
        Vec::new(),
        vec![
            "The burns are along the direction of motion; a positive value raises the orbit and a negative value lowers it.".to_string(),
            "The transfer is the cheapest two impulse transfer between two coplanar circular orbits in the impulsive model.".to_string(),
            "The coast between the burns is half the period of the transfer ellipse.".to_string(),
            format!(
                "The gravitational parameter is {}.",
                gravity_model_name(params.choice("model")?)
            ),
        ],
    ))
}

/// Returns the gravity model named by a select value.
fn gravity_model(value: &str) -> GravityModel {
    match value {
        "wgs84" => GravityModel::WGS84,
        _ => GravityModel::WGS72,
    }
}

/// Returns the display name of a gravity model select value.
fn gravity_model_name(value: &str) -> &'static str {
    match value {
        "wgs84" => "WGS 84",
        _ => "WGS 72",
    }
}

/// Returns the gravitational parameter of a gravity model select value.
fn gravity_parameter(value: &str) -> GravityParameter {
    match value {
        "wgs84" => GravityParameter::WGS84,
        _ => GravityParameter::WGS72,
    }
}

/// Returns the time scale named by a select value.
fn time_scale(value: &str) -> TimeScale {
    match value {
        "tai" => TimeScale::Tai,
        "tt" => TimeScale::Tt,
        "gps" => TimeScale::Gps,
        _ => TimeScale::Utc,
    }
}

/// Formats an epoch as a UTC calendar string.
fn format_calendar(epoch: Epoch) -> String {
    match epoch.calendar(TimeScale::Utc) {
        Some(calendar) => format_calendar_text(calendar),
        None => "(no UTC representation)".to_string(),
    }
}

/// Formats a calendar value.
fn format_calendar_text(calendar: Calendar) -> String {
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:09.6}",
        calendar.year,
        calendar.month,
        calendar.day,
        calendar.hour,
        calendar.minute,
        calendar.second,
    )
}

/// Returns the smallest and largest value of a slice.
///
/// An empty slice returns not-a-number for both, which encodes as `null` and
/// is rendered by the frontend as a missing value.
fn min_max(values: &[f64]) -> (f64, f64) {
    let mut minimum = f64::NAN;
    let mut maximum = f64::NAN;
    for value in values.iter().filter(|value| value.is_finite()) {
        minimum = if minimum.is_nan() {
            *value
        } else {
            minimum.min(*value)
        };
        maximum = if maximum.is_nan() {
            *value
        } else {
            maximum.max(*value)
        };
    }
    (minimum, maximum)
}

/// Estimates the semi-major axis implied by a mean motion.
///
/// The estimate uses Kepler's third law with the WGS 72 gravitational
/// parameter, so it is the size of the circular orbit with that mean motion.
fn semi_major_axis_estimate(mean_motion: f64) -> f64 {
    if mean_motion <= 0.0 {
        return f64::NAN;
    }
    let radians_per_second = mean_motion * 2.0 * std::f64::consts::PI / 86_400.0;
    (GravityModel::WGS72.mu / (radians_per_second * radians_per_second)).cbrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::Value;
    use crate::tools::{Params, Registry};

    /// Runs a tool with the given parameters.
    fn call(id: &str, values: Value) -> Result<Value, ToolError> {
        let registry = Registry::new();
        let tool = registry.get(id).unwrap();
        let params = Params::new(tool, &values).unwrap();
        run(id, &params)
    }

    const TLE: &str = "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927\n2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537";

    #[test]
    fn parse_tle_reports_the_fields() {
        let value = call("parse-tle", json_object! { "tle" => TLE }).unwrap();
        assert_eq!(value.get("kind").unwrap().as_str(), Some("element-set"));
        let rows = value.get("table").unwrap().get("rows").unwrap().as_array();
        assert_eq!(rows.len(), 10);
    }

    #[test]
    fn parse_tle_rejects_a_bad_checksum() {
        let broken = TLE.replace("2927", "2928");
        let error = call("parse-tle", json_object! { "tle" => broken }).unwrap_err();
        assert_eq!(error.parameter, Some("tle"));
    }

    #[test]
    fn sgp4_propagates_the_sample() {
        let value = call(
            "sgp4-propagate",
            json_object! { "tle" => TLE, "minutes" => 0.0 },
        )
        .unwrap();
        let scalars = value.get("scalars").unwrap().as_array();
        let radius = scalars
            .iter()
            .find(|scalar| scalar.get("label").unwrap().as_str() == Some("Radius"))
            .unwrap()
            .get("value")
            .unwrap()
            .as_str()
            .unwrap()
            .parse::<f64>()
            .unwrap();
        assert!((6_600.0..7_200.0).contains(&radius));
    }

    #[test]
    fn sgp4_track_returns_samples_and_series() {
        let value = call(
            "sgp4-track",
            json_object! { "tle" => TLE, "start_minutes" => 0.0, "end_minutes" => 10.0, "step_minutes" => 1.0 },
        )
        .unwrap();
        let rows = value.get("table").unwrap().get("rows").unwrap().as_array();
        assert_eq!(rows.len(), 11);
        assert_eq!(value.get("series").unwrap().as_array().len(), 2);
        assert!(value.get("axis").is_some());
    }

    #[test]
    fn sgp4_track_rejects_a_range_that_is_too_large() {
        let error = call(
            "sgp4-track",
            json_object! { "tle" => TLE, "start_minutes" => 0.0, "end_minutes" => 100000.0, "step_minutes" => 0.001 },
        )
        .unwrap_err();
        assert_eq!(error.parameter, Some("step_minutes"));
    }

    #[test]
    fn osculating_comparison_lists_six_rows() {
        let value = call("orbit-osculating", json_object! { "tle" => TLE }).unwrap();
        let rows = value.get("table").unwrap().get("rows").unwrap().as_array();
        assert_eq!(rows.len(), 6);
    }

    #[test]
    fn kepler_conversion_returns_a_state() {
        let value = call("kepler-state", json_object! {}).unwrap();
        assert_eq!(value.get("kind").unwrap().as_str(), Some("state-vector"));
    }

    #[test]
    fn kepler_conversion_rejects_a_non_elliptical_orbit() {
        // An eccentricity of one is a parabola; the conversion requires an
        // ellipse, so the tool reports the condition without naming a
        // parameter.
        let error = call(
            "kepler-state",
            json_object! { "eccentricity" => 1.0, "semi_major_axis" => 7000.0 },
        )
        .unwrap_err();
        assert!(error.parameter.is_none());
    }

    #[test]
    fn time_conversion_lists_every_scale() {
        let value = call(
            "time-convert",
            json_object! { "input" => "calendar", "year" => 2024.0, "month" => 6.0, "day" => 15.0 },
        )
        .unwrap();
        let rows = value.get("table").unwrap().get("rows").unwrap().as_array();
        assert_eq!(rows.len(), 4);
    }

    #[test]
    fn time_conversion_rejects_a_date_outside_the_leap_table() {
        let error = call(
            "time-convert",
            json_object! { "input" => "calendar", "year" => 1960.0, "month" => 1.0, "day" => 1.0 },
        )
        .unwrap_err();
        assert!(error.message.contains("1972"));
    }

    #[test]
    fn geodetic_conversion_matches_the_equatorial_radius() {
        let value = call(
            "geodetic-ecef",
            json_object! { "latitude" => 0.0, "longitude" => 0.0, "height" => 0.0 },
        )
        .unwrap();
        let scalars = value.get("scalars").unwrap().as_array();
        let x = scalars
            .iter()
            .find(|scalar| scalar.get("label").unwrap().as_str() == Some("X"))
            .unwrap()
            .get("value")
            .unwrap()
            .as_str()
            .unwrap()
            .parse::<f64>()
            .unwrap();
        assert!((x - 6_378_137.0).abs() < 1e-3);
    }

    #[test]
    fn frame_rotation_preserves_the_radius() {
        let value = call(
            "frame-transform",
            json_object! { "x" => 6871000.0, "y" => 1234000.0, "z" => 500000.0, "earth_rotation" => 0.3 },
        )
        .unwrap();
        let scalars = value.get("scalars").unwrap().as_array();
        let change = scalars
            .iter()
            .find(|scalar| scalar.get("label").unwrap().as_str() == Some("Radius change"))
            .unwrap()
            .get("value")
            .unwrap()
            .as_str()
            .unwrap()
            .parse::<f64>()
            .unwrap();
        assert!(change.abs() < 1e-6);
    }

    #[test]
    fn hohmann_transfer_reports_two_burns() {
        let value = call("hohmann-transfer", json_object! {}).unwrap();
        assert_eq!(value.get("kind").unwrap().as_str(), Some("maneuver"));
        let rows = value.get("table").unwrap().get("rows").unwrap().as_array();
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn hohmann_transfer_rejects_equal_radii() {
        let error = call(
            "hohmann-transfer",
            json_object! { "from_radius" => 7000.0, "to_radius" => 7000.0 },
        )
        .unwrap_err();
        assert_eq!(error.parameter, Some("to_radius"));
    }

    #[test]
    fn unknown_tools_are_rejected() {
        assert!(
            run(
                "nope",
                &Params::new(Registry::new().get("parse-tle").unwrap(), &json_object! {}).unwrap()
            )
            .is_err()
        );
    }

    #[test]
    fn encoded_results_are_parsable_json() {
        let value = call("sgp4-propagate", json_object! { "tle" => TLE }).unwrap();
        assert!(crate::json::parse(&crate::json::to_string(&value)).is_ok());
    }
}
