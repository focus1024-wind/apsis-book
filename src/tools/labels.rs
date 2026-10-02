//! Chinese labels for the tool results.
//!
//! A tool handler writes its labels in English, because English is the
//! language the interface identifiers and the `apsis-rs` documentation use.
//! This table pairs each label with its Chinese form so that a result can be
//! shown in either language without the handler carrying two copies of every
//! string.
//!
//! A label that is absent from the table is shown in English in both
//! languages. That is the correct behaviour for a value that is a symbol
//! rather than prose, such as `position.x`, `BSTAR`, or a unit name.
//!
//! 工具结果的中文标签。
//!
//! 工具处理函数以英文书写其标签，因为英文是接口标识与 `apsis-rs` 文档所用的语言。
//! 本表把每个标签与其中文形式配对，使结果可以以任一语言显示，而处理函数无须为
//! 每条字符串携带两份副本。
//!
//! 表中没有的标签在两种语言中都显示英文。对一个符号而非散文的值而言，这是正确的
//! 行为，例如 `position.x`、`BSTAR` 或单位名。

use crate::i18n::Text;

/// Returns the bilingual form of a result label.
///
/// 返回结果标签的双语形式。
pub fn label(english: &str) -> Text {
    // The lookup is a linear scan over a static table. The table is small
    // and a result carries at most a few dozen labels, so a scan is simpler
    // than a map that would need to be built at startup.

    // 查找是对静态表的线性扫描。表很小，而一个结果至多携带几十个标签，因此扫描
    // 比在启动时构建映射更简单。
    for (key, zh) in PAIRS {
        if *key == english {
            // Both members are borrowed from the static table, so the
            // result is `'static`.

            // 两个成员都借自静态表，因此结果是 `'static` 的。
            return Text::new(zh, key);
        }
    }
    // A label that is not in the table is a symbol. It is shown as it is in
    // both languages, which is what a reader of either language expects for
    // a name.

    // 不在表中的标签是一个符号。它在两种语言中都按原样显示，对一个名字而言，
    // 这正是两种语言的读者所期望的。
    Text::new(own(english), own(english))
}

/// Leaks a runtime string so that it can be carried by a [`Text`].
///
/// The number of distinct labels is bounded by the tool table and every
/// label is a literal written in a handler, so the leaks are bounded and
/// happen once per distinct label. This keeps [`Text`] a `Copy` type with
/// static fields, which is what the registry and the result shape rely on.
///
/// 泄漏一个运行期字符串，使它能被 [`Text`] 承载。
///
/// 不同标签的数量受工具表限制，且每个标签都是处理函数中书写的字面量，因此泄漏是
/// 有界的，且每个不同标签只发生一次。这让 [`Text`] 保持为带静态字段的 `Copy`
/// 类型，注册表与结果形状正依赖于此。
fn own(text: &str) -> &'static str {
    use std::collections::HashSet;
    use std::sync::Mutex;
    use std::sync::OnceLock;

    static SEEN: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();
    let seen = SEEN.get_or_init(|| Mutex::new(HashSet::new()));
    let mut guard = match seen.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    if let Some(existing) = guard.get(text) {
        return existing;
    }
    let leaked: &'static str = Box::leak(text.to_string().into_boxed_str());
    guard.insert(leaked);
    leaked
}

/// The label pairs, ordered as the labels appear in the results.
///
/// 标签对，按标签在结果中出现的顺序排列。
const PAIRS: &[(&str, &str)] = &[
    // Element set parsing
    ("Name", "名称"),
    ("Catalog number", "编目号"),
    ("Classification", "密级"),
    ("International designator", "国际编号"),
    ("Element set number", "元素集编号"),
    ("Revolution number", "圈数"),
    ("Epoch", "历元"),
    ("Field", "字段"),
    ("Value", "数值"),
    ("Meaning", "含义"),
    ("Mean motion", "平均运动"),
    (
        "First derivative of mean motion, divided by two",
        "平均运动一阶导数的二分之一",
    ),
    (
        "Second derivative of mean motion, divided by six",
        "平均运动二阶导数的六分之一",
    ),
    ("SGP4 drag term", "SGP4 阻力项"),
    ("Inclination", "倾角"),
    ("Right ascension of the ascending node", "升交点赤经"),
    ("Eccentricity", "偏心率"),
    ("Argument of perigee", "近地点幅角"),
    ("Mean anomaly", "平近点角"),
    ("Orbital period", "轨道周期"),
    (
        "Orbital period implied by the mean motion",
        "由平均运动推出的轨道周期",
    ),
    ("Semi-major axis estimate", "半长轴估计"),
    ("Unit", "单位"),
    ("first", "第一次"),
    ("Two-line element set", "两行元素集"),
    ("second", "第二次"),
    // Propagation
    ("Branch", "分支"),
    ("Minutes from epoch", "距历元分钟数"),
    ("Minutes", "分钟"),
    ("Position X", "位置 X"),
    ("Position Y", "位置 Y"),
    ("Position Z", "位置 Z"),
    ("Velocity X", "速度 X"),
    ("Velocity Y", "速度 Y"),
    ("Velocity Z", "速度 Z"),
    ("Radius", "半径"),
    ("Altitude above the equatorial radius", "赤道半径以上的高度"),
    (
        "Altitude above the equatorial radius at the lowest point",
        "最低点处赤道半径以上的高度",
    ),
    ("Speed", "速率"),
    ("Semi-major axis", "半长轴"),
    ("Component", "分量"),
    ("Propagated state", "传播得到的状态"),
    ("Samples", "采样点数"),
    ("Duration", "时长"),
    ("Lowest radius", "最低半径"),
    ("Highest radius", "最高半径"),
    ("Sampled ground track", "采样的星下点轨迹"),
    ("Mean", "平均"),
    ("Osculating", "密切"),
    ("Element", "元素"),
    ("Semi-major axis change", "半长轴变化"),
    ("Eccentricity change", "偏心率变化"),
    ("Inclination change", "倾角变化"),
    ("Mean and osculating elements", "平均元素与密切元素"),
    // Keplerian conversion
    ("Semi-major axis", "半长轴"),
    ("Perigee radius", "近地点半径"),
    ("Apogee radius", "远地点半径"),
    ("Period", "周期"),
    ("Semi-latus rectum", "半通径"),
    ("Semi-minor axis", "短半轴"),
    ("Quantity", "量"),
    ("Quantity", "量"),
    (
        "Keplerian elements to a state vector",
        "开普勒元素转状态矢量",
    ),
    // Time
    ("Scale", "尺度"),
    ("Modified Julian Date", "约化儒略日"),
    ("Calendar date", "日历日期"),
    ("Offset from TAI", "相对 TAI 的偏移"),
    ("TAI minus UTC at this instant", "该时刻的 TAI 减 UTC"),
    ("Time scale conversion", "时间尺度转换"),
    // Geodesy
    ("X", "X"),
    ("Y", "Y"),
    ("Z", "Z"),
    ("Geocentric radius", "地心半径"),
    ("Prime vertical radius of curvature", "卯酉圈曲率半径"),
    ("Meridian radius of curvature", "子午圈曲率半径"),
    ("Geocentric latitude", "地心纬度"),
    ("Geodetic minus geocentric latitude", "大地纬度减地心纬度"),
    (
        "Geodetic position to Earth-fixed coordinates",
        "大地坐标转地固坐标",
    ),
    // Frames
    ("Direction", "方向"),
    ("Input X", "输入 X"),
    ("Input Y", "输入 Y"),
    ("Input Z", "输入 Z"),
    ("Output X", "输出 X"),
    ("Output Y", "输出 Y"),
    ("Output Z", "输出 Z"),
    ("Input radius", "输入半径"),
    ("Output radius", "输出半径"),
    ("Radius change", "半径变化"),
    ("Earth Rotation Angle", "地球自转角"),
    ("Input", "输入"),
    ("Output", "输出"),
    ("Frame rotation", "参考系旋转"),
    // Maneuver
    ("First burn", "第一次点火"),
    ("Second burn", "第二次点火"),
    ("Total velocity change", "总速度增量"),
    ("Transfer semi-major axis", "转移轨道半长轴"),
    ("Transfer time", "转移时间"),
    ("Burn", "点火"),
    ("Signed", "带符号"),
    ("Magnitude", "大小"),
    ("Hohmann transfer", "霍曼转移"),
    // Notes shown with a result
    (
        "Angles are reported in the units the element set uses, which are degrees for the orientation angles.",
        "角度按元素集所用的单位报告，姿态角为度。",
    ),
    (
        "The semi-major axis estimate follows from the mean motion and Kepler's third law with the WGS 72 gravitational parameter; it ignores the perturbations the propagator models.",
        "半长轴估计由平均运动与开普勒第三定律、配合 WGS 72 引力参数得出；它忽略了传播器所建模的摄动。",
    ),
    (
        "The frame is the True Equator Mean Equinox frame, which is the frame the model defines.",
        "坐标系为真赤道平春分点坐标系，即模型所定义的坐标系。",
    ),
    (
        "Each sample is one propagation of the mean elements; the table is not interpolated.",
        "每个采样点是对平均元素的一次传播；表中数据未经插值。",
    ),
    (
        "The track is given in the True Equator Mean Equinox frame, so a ground projection needs the Earth rotation in addition.",
        "轨迹位于真赤道平春分点坐标系，因此地面投影还需额外施加地球自转。",
    ),
    (
        "The mean elements are the ones written in the element set and are the inputs of the propagator.",
        "平均元素即元素集所写的元素，是传播器的输入。",
    ),
    (
        "The osculating elements describe the two-body orbit that matches the instantaneous state; they are an output of the propagation, not an input.",
        "密切元素描述与瞬时状态相符的二体轨道；它是传播的输出，不是输入。",
    ),
    (
        "The mean semi-major axis follows from the mean motion with Kepler's third law, which is an approximation the propagator refines.",
        "平均半长轴由平均运动与开普勒第三定律得出，该近似会被传播器进一步修正。",
    ),
    (
        "The row labelled true_anomaly compares the mean anomaly of the element set with the true anomaly of the osculating elements; the two are different angles and the comparison shows the difference between them.",
        "标为 true_anomaly 的行比较元素集的平近点角与密切元素的真近点角；两者是不同的角，该行显示其差异。",
    ),
    (
        "The conversion is the two-body solution; it does not include the perturbations that SGP4 models.",
        "该转换为二体解；不包含 SGP4 所建模的摄动。",
    ),
    (
        "The instant is stored as a Modified Julian Date on the TAI scale, so the difference between two instants is exact elapsed time.",
        "时刻以 TAI 尺度上的约化儒略日存储，因此两个时刻之差恰为经过时间。",
    ),
    (
        "UTC is not a uniform scale; the offset column is the difference between TAI and the scale at that instant.",
        "UTC 不是均匀尺度；偏移列为该时刻 TAI 与该尺度之差。",
    ),
    (
        "This instant precedes the start of the leap second table, so it has no UTC representation.",
        "该时刻早于闰秒表的起点，因此没有 UTC 表示。",
    ),
    (
        "The ellipsoid is WGS 84; its semi-major axis and inverse flattening are the defining parameters.",
        "椭球为 WGS 84；其长半轴与扁率倒数是定义参数。",
    ),
    (
        "Latitude is geodetic, measured from the ellipsoid normal, not geocentric; the difference is largest at mid latitudes.",
        "纬度为大地纬度，自椭球法线量起，不是地心纬度；两者在中纬度差异最大。",
    ),
    (
        "The rotation is about the third axis by the Earth Rotation Angle and preserves the length of the vector.",
        "该旋转为绕第三轴转动地球自转角，保持矢量长度不变。",
    ),
    (
        "Passing an Earth-fixed position through the inertial frame and back returns the Earth-fixed position; the angle is not reduced to one turn.",
        "把地固位置经惯性系再转回可得到原地固位置；该角不归约到一圈之内。",
    ),
    (
        "The burns are along the direction of motion; a positive value raises the orbit and a negative value lowers it.",
        "两次点火沿运动方向；正值抬升轨道，负值降低轨道。",
    ),
    (
        "The transfer is the cheapest two impulse transfer between two coplanar circular orbits in the impulsive model.",
        "在脉冲模型中，这是两个共面圆轨道之间最省燃料的两次脉冲转移。",
    ),
    (
        "The coast between the burns is half the period of the transfer ellipse.",
        "两次点火之间的滑行为转移椭圆周期的一半。",
    ),
    // Chart axes and series
    ("Minutes from epoch", "距历元分钟数"),
];
