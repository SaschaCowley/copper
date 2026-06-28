pub(crate) struct Multiplier(&'static str, &'static [&'static str], f64);

impl Multiplier {
	pub(crate) fn name(&self) -> &'static str {
		self.0
	}
	pub(crate) fn symbols(&self) -> &[&'static str] {
		self.1
	}
	pub(crate) fn factor(&self) -> f64 {
		self.2
	}
}

pub(crate) const NONDATA_METRIC_MULTIPLIERS: [Multiplier; 14] = [
	Multiplier("quecto", &["q"], 1e-30),
	Multiplier("ronto", &["r"], 1e-27),
	Multiplier("yocto", &["y"], 1e-24),
	Multiplier("zepto", &["z"], 1e-21),
	Multiplier("atto", &["a"], 1e-18),
	Multiplier("femto", &["f"], 1e-15),
	Multiplier("pico", &["p"], 1e-12),
	Multiplier("nano", &["n"], 1e-9),
	Multiplier("micro", &["μ", "u"], 1e-6),
	Multiplier("milli", &["m"], 1e-3),
	Multiplier("centi", &["c"], 1e-2),
	Multiplier("deci", &["d"], 1e-1),
	Multiplier("deca", &["da"], 1e1),
	Multiplier("hecto", &["h"], 1e2),
];

pub(crate) const DATA_METRIC_MULTIPLIERS: [Multiplier; 10] = [
	Multiplier("kilo", &["k"], 1e3),
	Multiplier("mega", &["M"], 1e6),
	Multiplier("giga", &["G"], 1e9),
	Multiplier("tera", &["T"], 1e12),
	Multiplier("peta", &["P"], 1e15),
	Multiplier("exa", &["E"], 1e18),
	Multiplier("zetta", &["A"], 1e21),
	Multiplier("yotta", &["Y"], 1e24),
	Multiplier("ronna", &["R"], 1e27),
	Multiplier("quetta", &["Q"], 1e30),
];

pub(crate) const IEC_MULTIPLIERS: [Multiplier; 10] = [
	Multiplier("kibi", &["Ki"], (1u128 << 10) as f64),
	Multiplier("mebi", &["Mi"], (1u128 << 20) as f64),
	Multiplier("gibi", &["Gi"], (1u128 << 30) as f64),
	Multiplier("tebi", &["Ti"], (1u128 << 40) as f64),
	Multiplier("pebi", &["Pi"], (1u128 << 50) as f64),
	Multiplier("exbi", &["Ei"], (1u128 << 60) as f64),
	Multiplier("zebi", &["Zi"], (1u128 << 70) as f64),
	Multiplier("yobi", &["Yi"], (1u128 << 80) as f64),
	Multiplier("robi", &["Ri"], (1u128 << 90) as f64),
	Multiplier("quebi", &["Qi"], (1u128 << 100) as f64),
];
