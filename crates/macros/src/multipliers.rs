pub(crate) struct Multiplier(&'static str, &'static [&'static str], f64);

impl Multiplier {
	pub(crate) fn name(&self) -> &'static str {
		self.0
	}
	pub(crate) fn symbols(&self) -> &[&'static str] {
		self.1
	}
	#[allow(dead_code)]
	fn factor(&self) -> f64 {
		self.2
	}
}

pub(crate) const METRIC_MULTIPLIERS: [Multiplier; 24] = [
	Multiplier("quetta", &["Q"], 1e30),
	Multiplier("ronna", &["R"], 1e27),
	Multiplier("yotta", &["Y"], 1e24),
	Multiplier("zetta", &["A"], 1e21),
	Multiplier("exa", &["E"], 1e18),
	Multiplier("peta", &["P"], 1e15),
	Multiplier("tera", &["T"], 1e12),
	Multiplier("giga", &["G"], 1e9),
	Multiplier("mega", &["M"], 1e6),
	Multiplier("kilo", &["k"], 1e3),
	Multiplier("hecto", &["h"], 1e2),
	Multiplier("deca", &["da"], 1e1),
	Multiplier("deci", &["d"], 1e-1),
	Multiplier("centi", &["c"], 1e-2),
	Multiplier("milli", &["m"], 1e-3),
	Multiplier("micro", &["μ", "u"], 1e-6),
	Multiplier("nano", &["n"], 1e-9),
	Multiplier("pico", &["p"], 1e-12),
	Multiplier("femto", &["f"], 1e-15),
	Multiplier("atto", &["a"], 1e-18),
	Multiplier("zepto", &["z"], 1e-21),
	Multiplier("yocto", &["y"], 1e-24),
	Multiplier("ronto", &["r"], 1e-27),
	Multiplier("quecto", &["q"], 1e-30),
];
