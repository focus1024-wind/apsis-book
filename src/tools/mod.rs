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
pub mod labels;
pub mod out;

use std::collections::HashMap;

use crate::i18n::Text;
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
    /// Label shown to the user, in both languages.
    pub label: Text,
}

/// The description of one tool parameter.
#[derive(Debug, Clone)]
pub struct ParamSpec {
    /// Machine name, used as the JSON member name.
    pub name: &'static str,
    /// Human readable label.
    pub label: Text,
    /// One sentence explanation shown as help text.
    pub help: Text,
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
    pub name: Text,
    /// One sentence summary.
    pub summary: Text,
    /// Longer description, shown above the form.
    pub description: Text,
    /// Grouping shown in the tool list.
    pub category: Text,
    /// Parameters, in form order.
    pub params: &'static [ParamSpec],
    /// The `apsis-rs` items the tool calls, for cross linking to the
    /// interface documentation.
    pub api_items: &'static [&'static str],
}

/// The registry of every tool.
///
/// The table is a constant, so its parameter slices have a `'static`
/// lifetime and the registry holds no owned data.
pub struct Registry {
    by_id: HashMap<&'static str, usize>,
}

impl Registry {
    /// Builds the registry from the tool table.
    pub fn new() -> Self {
        let by_id = TOOL_TABLE
            .iter()
            .enumerate()
            .map(|(index, tool)| (tool.id, index))
            .collect();
        Self { by_id }
    }

    /// Returns every tool.
    pub fn entries(&self) -> &'static [ToolSpec] {
        TOOL_TABLE
    }

    /// Returns a tool by identifier.
    pub fn get(&self, id: &str) -> Option<&'static ToolSpec> {
        self.by_id.get(id).map(|index| &TOOL_TABLE[*index])
    }

    /// Returns the distinct categories in the order they first appear.
    pub fn categories(&self) -> Vec<Text> {
        let mut categories: Vec<Text> = Vec::new();
        for tool in TOOL_TABLE {
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

/// The tool table.
///
/// It is a constant rather than a function so that the parameter slices are
/// promoted to `'static` and the registry borrows it instead of owning it.
const TOOL_TABLE: &[ToolSpec] = &[
    ToolSpec {
        id: "parse-tle",
        name: Text::new("解析两行元素集", "Parse a two-line element set"),
        summary: Text::new(
            "解码两行元素集的字段并校验每行校验和。",
            "Decode the fields of a TLE and verify the line checksums.",
        ),
        description: Text::new(
            "从名称行加两行数据、或仅两行数据读取两行元素集，校验每行校验和，并按格式定义报告每个字段及其单位。",
            "Reads a two-line element set from either the name line plus two data lines or the two data lines alone, verifies the checksum of each line, and reports every field with its unit as the format defines it.",
        ),
        category: Text::new("元素集", "Elements"),
        params: &[ParamSpec {
            name: "tle",
            label: Text::new("元素集", "Element set"),
            help: Text::new(
                "两行或三行：可选的名称行，后接两行数据。",
                "Two or three lines: an optional name line followed by the two data lines.",
            ),
            kind: ParamKind::Multiline,
            default: Some(DEFAULT_TLE),
            required: true,
        }],
        api_items: &["TwoLineElement", "parse"],
    },
    ToolSpec {
        id: "sgp4-propagate",
        name: Text::new("SGP4 轨道传播", "Propagate with SGP4"),
        summary: Text::new(
            "用 SGP4 或 SDP4 分支将平均元素传播到某一时刻。",
            "Propagate mean elements to one instant with the SGP4 or SDP4 branch.",
        ),
        description: Text::new(
            "将平均元素准备用于传播，返回距元素历元指定分钟数时刻在真赤道平春分点坐标系中的位置和速度。",
            "Prepares the mean elements for propagation and returns the position and velocity in the True Equator Mean Equinox frame at a chosen number of minutes from the element epoch.",
        ),
        category: Text::new("轨道传播", "Propagation"),
        params: &[
            ParamSpec {
                name: "tle",
                label: Text::new("元素集", "Element set"),
                help: Text::new("待传播的平均元素。", "The mean elements to propagate."),
                kind: ParamKind::Multiline,
                default: Some(DEFAULT_TLE),
                required: true,
            },
            ParamSpec {
                name: "model",
                label: Text::new("引力模型", "Gravity model"),
                help: Text::new(
                    "必须与元素拟合时使用的模型一致。",
                    "Must match the model the elements were fitted against.",
                ),
                kind: ParamKind::Select(&GRAVITY_MODELS),
                default: Some("wgs72"),
                required: true,
            },
            ParamSpec {
                name: "minutes",
                label: Text::new("距历元分钟数", "Minutes from epoch"),
                help: Text::new(
                    "自元素集历元起经过的时间。",
                    "Elapsed time from the epoch written in the element set.",
                ),
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
        name: Text::new("采样星下点轨迹", "Sample a ground track"),
        summary: Text::new(
            "在时间网格上传播轨迹以便绘图。",
            "Propagate a track on a time grid for plotting.",
        ),
        description: Text::new(
            "在分钟区间内将轨道采样为星历表并返回采样点，便于绘制轨迹和高度剖面。",
            "Samples the orbit into an ephemeris table over a range of minutes and returns the points so that a track and an altitude profile can be plotted.",
        ),
        category: Text::new("轨道传播", "Propagation"),
        params: &[
            ParamSpec {
                name: "tle",
                label: Text::new("元素集", "Element set"),
                help: Text::new("待传播的平均元素。", "The mean elements to propagate."),
                kind: ParamKind::Multiline,
                default: Some(DEFAULT_TLE),
                required: true,
            },
            ParamSpec {
                name: "model",
                label: Text::new("引力模型", "Gravity model"),
                help: Text::new(
                    "必须与元素拟合时使用的模型一致。",
                    "Must match the model the elements were fitted against.",
                ),
                kind: ParamKind::Select(&GRAVITY_MODELS),
                default: Some("wgs72"),
                required: true,
            },
            ParamSpec {
                name: "start_minutes",
                label: Text::new("起始", "Start"),
                help: Text::new(
                    "第一个采样点，距元素历元的分钟数。",
                    "First sample, in minutes from the element epoch.",
                ),
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
                label: Text::new("结束", "End"),
                help: Text::new(
                    "最后一个采样点，距元素历元的分钟数。",
                    "Last sample, in minutes from the element epoch.",
                ),
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
                label: Text::new("步长", "Step"),
                help: Text::new(
                    "采样间隔。允许采样点数为 2 到 2000。",
                    "Interval between samples. Between two and two thousand samples are allowed.",
                ),
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
        name: Text::new(
            "对比平均元素与密切元素",
            "Compare mean and osculating elements",
        ),
        summary: Text::new(
            "将二体密切元素与平均元素并列显示。",
            "Show the osculating two-body elements beside the mean elements.",
        ),
        description: Text::new(
            "传播平均元素并将瞬时状态转换为开普勒元素。两者不同，因为 SGP4 包含二体模型没有的摄动，对比可显示摄动的影响。",
            "Propagates the mean elements and converts the instantaneous state to Keplerian elements. The two sets differ because SGP4 includes perturbations that the two-body model does not, so the comparison shows what the perturbations do.",
        ),
        category: Text::new("轨道传播", "Propagation"),
        params: &[
            ParamSpec {
                name: "tle",
                label: Text::new("元素集", "Element set"),
                help: Text::new("待传播的平均元素。", "The mean elements to propagate."),
                kind: ParamKind::Multiline,
                default: Some(DEFAULT_TLE),
                required: true,
            },
            ParamSpec {
                name: "model",
                label: Text::new("引力模型", "Gravity model"),
                help: Text::new(
                    "必须与元素拟合时使用的模型一致。",
                    "Must match the model the elements were fitted against.",
                ),
                kind: ParamKind::Select(&GRAVITY_MODELS),
                default: Some("wgs72"),
                required: true,
            },
            ParamSpec {
                name: "minutes",
                label: Text::new("距历元分钟数", "Minutes from epoch"),
                help: Text::new(
                    "评估密切元素的时刻。",
                    "Instant at which the osculating elements are evaluated.",
                ),
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
        name: Text::new(
            "开普勒元素转状态矢量",
            "Keplerian elements to a state vector",
        ),
        summary: Text::new(
            "将六个经典元素转换为位置和速度。",
            "Convert the six classical elements to position and velocity.",
        ),
        description: Text::new(
            "对给定元素求解二体问题，返回元素所参考惯性系中的位置和速度，以及推导出的轨道尺寸。",
            "Solves the two-body problem for the given elements and returns the position and velocity in the inertial frame the elements are referred to, together with the derived size of the orbit.",
        ),
        category: Text::new("轨道", "Orbits"),
        params: &[
            ParamSpec {
                name: "semi_major_axis",
                label: Text::new("半长轴", "Semi-major axis"),
                help: Text::new("轨道的正尺寸。", "Positive size of the orbit."),
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
                label: Text::new("偏心率", "Eccentricity"),
                help: Text::new(
                    "无量纲形状，大于等于零且小于一。",
                    "Dimensionless shape, at or above zero and below one.",
                ),
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
                label: Text::new("倾角", "Inclination"),
                help: Text::new(
                    "轨道平面与赤道面的夹角。",
                    "Angle between the orbit plane and the equator.",
                ),
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
                label: Text::new("升交点赤经", "Right ascension of the node"),
                help: Text::new(
                    "轨道向北穿越赤道的位置。",
                    "Where the orbit crosses the equator going north.",
                ),
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
                label: Text::new("近地点幅角", "Argument of perigee"),
                help: Text::new(
                    "在轨平面内从升交点到近地点的角度。",
                    "Angle from the ascending node to perigee, in the orbit plane.",
                ),
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
                label: Text::new("真近点角", "True anomaly"),
                help: Text::new(
                    "从近地点到卫星的角度。",
                    "Angle from perigee to the satellite.",
                ),
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
                label: Text::new("引力模型", "Gravity model"),
                help: Text::new(
                    "选择转换使用的引力参数。",
                    "Selects the gravitational parameter used by the conversion.",
                ),
                kind: ParamKind::Select(&GRAVITY_MODELS),
                default: Some("wgs72"),
                required: true,
            },
        ],
        api_items: &["KeplerianElements", "to_state", "GravityParameter"],
    },
    ToolSpec {
        id: "time-convert",
        name: Text::new("转换时间尺度", "Convert a time scale"),
        summary: Text::new(
            "在 UTC、TAI、TT 和 GPS 之间转换时刻。",
            "Convert an instant between UTC, TAI, TT, and GPS.",
        ),
        description: Text::new(
            "接受日历日期或约化儒略日，并报告该时刻在所有支持尺度上的值。闰秒表是 `apsis-rs` 内嵌的表，因此在覆盖范围内 UTC 转换是精确的。",
            "Accepts either a calendar date or a Modified Julian Date and reports the instant on every supported scale. The leap second table is the one embedded in `apsis-rs`, so UTC conversions are exact over the range the table covers.",
        ),
        category: Text::new("时间", "Time"),
        params: &[
            ParamSpec {
                name: "input",
                label: Text::new("输入形式", "Input form"),
                help: Text::new(
                    "时刻以日历日期还是约化儒略日给出。",
                    "Whether the instant is given as a calendar date or as a Modified Julian Date.",
                ),
                kind: ParamKind::Select(&TIME_INPUTS),
                default: Some("calendar"),
                required: true,
            },
            ParamSpec {
                name: "scale",
                label: Text::new("输入尺度", "Input scale"),
                help: Text::new("输入所表示的尺度。", "The scale the input is expressed on."),
                kind: ParamKind::Select(&TIME_SCALES),
                default: Some("utc"),
                required: true,
            },
            ParamSpec {
                name: "year",
                label: Text::new("年", "Year"),
                help: Text::new(
                    "日历年，当输入形式为日历日期时使用。",
                    "Calendar year, used when the input form is a calendar date.",
                ),
                kind: ParamKind::Integer {
                    min: Some(1950),
                    max: Some(2100),
                },
                default: Some("2024"),
                required: false,
            },
            ParamSpec {
                name: "month",
                label: Text::new("月", "Month"),
                help: Text::new(
                    "日历月，当输入形式为日历日期时使用。",
                    "Calendar month, used when the input form is a calendar date.",
                ),
                kind: ParamKind::Integer {
                    min: Some(1),
                    max: Some(12),
                },
                default: Some("1"),
                required: false,
            },
            ParamSpec {
                name: "day",
                label: Text::new("日", "Day"),
                help: Text::new(
                    "月内日期，当输入形式为日历日期时使用。",
                    "Day of the month, used when the input form is a calendar date.",
                ),
                kind: ParamKind::Integer {
                    min: Some(1),
                    max: Some(31),
                },
                default: Some("1"),
                required: false,
            },
            ParamSpec {
                name: "hour",
                label: Text::new("时", "Hour"),
                help: Text::new(
                    "一天中的小时，当输入形式为日历日期时使用。",
                    "Hour of the day, used when the input form is a calendar date.",
                ),
                kind: ParamKind::Integer {
                    min: Some(0),
                    max: Some(23),
                },
                default: Some("0"),
                required: false,
            },
            ParamSpec {
                name: "minute",
                label: Text::new("分", "Minute"),
                help: Text::new(
                    "小时中的分钟，当输入形式为日历日期时使用。",
                    "Minute of the hour, used when the input form is a calendar date.",
                ),
                kind: ParamKind::Integer {
                    min: Some(0),
                    max: Some(59),
                },
                default: Some("0"),
                required: false,
            },
            ParamSpec {
                name: "second",
                label: Text::new("秒", "Second"),
                help: Text::new(
                    "分钟中的秒，含小数，当输入形式为日历日期时使用。",
                    "Second of the minute, including the fraction, used when the input form is a calendar date.",
                ),
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
                label: Text::new("约化儒略日", "Modified Julian Date"),
                help: Text::new(
                    "输入尺度上的时刻，当输入形式为约化儒略日时使用。",
                    "Instant on the input scale, used when the input form is a Modified Julian Date.",
                ),
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
        name: Text::new(
            "大地坐标转地固坐标",
            "Geodetic position to Earth-fixed coordinates",
        ),
        summary: Text::new(
            "将纬度、经度和高度转换为地固矢量。",
            "Convert latitude, longitude, and height to an Earth-fixed vector.",
        ),
        description: Text::new(
            "将 WGS 84 椭球上的大地坐标转换为地心地固系中的直角坐标矢量，并报告推导出的曲率半径和地心纬度以便对比。",
            "Converts a geodetic position on the WGS 84 ellipsoid to a Cartesian vector in the Earth-centered Earth-fixed frame, and reports the derived radii of curvature and the geocentric latitude for comparison.",
        ),
        category: Text::new("坐标", "Coordinates"),
        params: &[
            ParamSpec {
                name: "latitude",
                label: Text::new("大地纬度", "Geodetic latitude"),
                help: Text::new(
                    "椭球法线与赤道面的夹角。",
                    "Angle between the ellipsoid normal and the equatorial plane.",
                ),
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
                label: Text::new("经度", "Longitude"),
                help: Text::new(
                    "以本初子午线向东为正。",
                    "Positive east of the prime meridian.",
                ),
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
                label: Text::new("椭球高", "Height above the ellipsoid"),
                help: Text::new(
                    "可为负值，表示点位于椭球面以下。",
                    "May be negative, which places the point below the ellipsoid.",
                ),
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
        name: Text::new(
            "惯性系与地固系之间的旋转",
            "Rotate between inertial and Earth-fixed frames",
        ),
        summary: Text::new(
            "对直角坐标矢量应用地球自转角。",
            "Apply the Earth Rotation Angle to a Cartesian vector.",
        ),
        description: Text::new(
            "在给定地球自转角下，将位置在地心惯性系与地心地固系之间旋转。旋转保持矢量长度不变，结果中会报告这一点。",
            "Rotates a position between the Earth-centered inertial frame and the Earth-centered Earth-fixed frame at a chosen Earth Rotation Angle. The rotation preserves the length of the vector, which the result reports.",
        ),
        category: Text::new("坐标", "Coordinates"),
        params: &[
            ParamSpec {
                name: "direction",
                label: Text::new("方向", "Direction"),
                help: Text::new("旋转的方向。", "Which way to rotate."),
                kind: ParamKind::Select(&FRAME_DIRECTIONS),
                default: Some("eci-to-ecef"),
                required: true,
            },
            ParamSpec {
                name: "x",
                label: Text::new("X", "X"),
                help: Text::new("位置的第一个分量。", "First component of the position."),
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
                label: Text::new("Y", "Y"),
                help: Text::new("位置的第二个分量。", "Second component of the position."),
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
                label: Text::new("Z", "Z"),
                help: Text::new("位置的第三个分量。", "Third component of the position."),
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
                label: Text::new("地球自转角", "Earth Rotation Angle"),
                help: Text::new(
                    "自 J2000.0 历元起地球转过的角度，单位圈。",
                    "Angle of the Earth's rotation since the J2000.0 epoch, in turns.",
                ),
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
        name: Text::new("规划霍曼转移", "Plan a Hohmann transfer"),
        summary: Text::new(
            "计算共面圆轨道之间转移的两次点火。",
            "Compute the two burns of a transfer between coplanar circular orbits.",
        ),
        description: Text::new(
            "返回将卫星在两个共面圆轨道之间转移的两次速度增量、两次点火之间的滑行时间以及总速度增量。正点火抬升轨道，负点火降低轨道。",
            "Returns the two velocity changes that move a satellite between two circular coplanar orbits, the coast time between them, and the total velocity change. A positive burn raises the orbit and a negative one lowers it.",
        ),
        category: Text::new("轨道", "Orbits"),
        params: &[
            ParamSpec {
                name: "from_radius",
                label: Text::new("起始轨道半径", "Starting orbit radius"),
                help: Text::new(
                    "转移起始的圆轨道半径。",
                    "Radius of the circular orbit the transfer starts from.",
                ),
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
                label: Text::new("目标轨道半径", "Target orbit radius"),
                help: Text::new(
                    "转移到达的圆轨道半径，必须与起始半径不同。",
                    "Radius of the circular orbit the transfer arrives at. It must differ from the starting radius.",
                ),
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
                label: Text::new("引力模型", "Gravity model"),
                help: Text::new(
                    "选择转移使用的引力参数。",
                    "Selects the gravitational parameter used by the transfer.",
                ),
                kind: ParamKind::Select(&GRAVITY_MODELS),
                default: Some("wgs72"),
                required: true,
            },
        ],
        api_items: &["HohmannTransfer", "hohmann_transfer"],
    },
];

/// The gravity model options shared by the propagation tools.
const GRAVITY_MODELS: [SelectOption; 2] = [
    SelectOption {
        value: "wgs72",
        label: Text::new(
            "WGS 72（SGP4 拟合所用模型）",
            "WGS 72 (the model SGP4 was fitted to)",
        ),
    },
    SelectOption {
        value: "wgs84",
        label: Text::new("WGS 84", "WGS 84"),
    },
];

/// The time scale options.
const TIME_SCALES: [SelectOption; 4] = [
    SelectOption {
        value: "utc",
        label: Text::new("UTC（协调世界时）", "UTC (Coordinated Universal Time)"),
    },
    SelectOption {
        value: "tai",
        label: Text::new("TAI（国际原子时）", "TAI (International Atomic Time)"),
    },
    SelectOption {
        value: "tt",
        label: Text::new("TT（地球时）", "TT (Terrestrial Time)"),
    },
    SelectOption {
        value: "gps",
        label: Text::new("GPS（GPS 时）", "GPS (GPS time)"),
    },
];

/// The input form options of the time tool.
const TIME_INPUTS: [SelectOption; 2] = [
    SelectOption {
        value: "calendar",
        label: Text::new("日历日期", "Calendar date"),
    },
    SelectOption {
        value: "mjd",
        label: Text::new("约化儒略日", "Modified Julian Date"),
    },
];

/// The rotation direction options of the frame tool.
const FRAME_DIRECTIONS: [SelectOption; 2] = [
    SelectOption {
        value: "eci-to-ecef",
        label: Text::new("惯性系到地固系", "Inertial to Earth-fixed"),
    },
    SelectOption {
        value: "ecef-to-eci",
        label: Text::new("地固系到惯性系", "Earth-fixed to inertial"),
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
///
/// Every label is encoded in both languages, so the frontend can switch the
/// language of the interface without another request.
pub fn registry_json(registry: &Registry) -> Value {
    let tools: Vec<Value> = registry.entries().iter().map(tool_json).collect();
    let categories: Vec<Value> = registry.categories().iter().map(Text::json).collect();
    json_object! {
        "categories" => categories,
        "tools" => tools,
    }
}

/// Encodes one tool as JSON.
fn tool_json(tool: &ToolSpec) -> Value {
    let params: Vec<Value> = tool.params.iter().map(param_json).collect();
    json_object! {
        "id" => tool.id,
        "name" => tool.name.json(),
        "summary" => tool.summary.json(),
        "description" => tool.description.json(),
        "category" => tool.category.json(),
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
                    .map(|option| json_object! { "value" => option.value, "label" => option.label.json() })
                    .collect::<Vec<Value>>(),
            },
        ),
    };
    json_object! {
        "name" => param.name,
        "label" => param.label.json(),
        "help" => param.help.json(),
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
