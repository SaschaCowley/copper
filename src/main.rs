use std::{
	collections::{HashMap, HashSet, VecDeque},
	env,
	sync::LazyLock,
};
use strum_macros::EnumString;
use libcopper::make_table;

// const QUETTA: f64 = 1e30; // Q
// const RONNA: f64 = 1e27; // R
// const YOTTA: f64 = 1e24; // Y
// const ZETTA: f64 = 1e21; // A
// const EXA: f64 = 1e18; // E
// const PETA: f64 = 1e15; // P
const TERA: f64 = 1e12; // T
const TEBI: f64 = (1i64 <<40) as f64; // T
const GIGA: f64 = 1e9 ; // G
const GIBI: f64 = (1<<30) as f64; // G
const MEGA: f64 = 1e6 ; // M
const MEBI: f64 = (1<<20) as f64; // M
const KILO: f64 = 1e3 ; // k
const KIBI: f64 = (1<<10) as f64;
// const HECTO: f64 = 1e2; // h
// const DECA: f64 = 1e1 ; // da
// const DECI: f64 = 1e-1 ; // d
const CENTI: f64 = 1e-2 ; // c
const MILLI: f64 = 1e-3 ; // m
// const MICRO: f64 = 1e-6 ; // μ/u
// const NANO: f64 = 1e-9 ; // n
// const PICO: f64 = 1e-12; // p
// const FEMTO: f64 = 1e-15; // f
// const ATTO: f64 = 1e-18; // a
// const ZEPTO: f64 = 1e-21; // z
// const YOCTO: f64 = 1e-24; // y
// const RONTO: f64 = 1e-27; // r
// const QUECTO: f64 = 1e-30; // q

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy, EnumString, strum_macros::Display)]
enum Unit {
	// Length
	// Metric
	#[strum(serialize="mm", to_string="millimetres")]
	Millimetre,
	#[strum(serialize="cm", to_string="centimetres")]
	Centimetre,
	#[strum(serialize="m", to_string="metres")]
	Metre,
	#[strum(serialize="km", to_string="kilometres")]
	Kilometre,
	// Imperial
	#[strum(serialize="in", to_string="inches")]
	Inch,
	#[strum(serialize="ft", to_string="feet")]
	Foot,
	#[strum(serialize="yd", to_string="yards")]
	Yard,
	#[strum(serialize="mi", to_string="miles")]
	Mile,
	
	// Temperature
	// Metric
	#[strum(serialize="C", to_string="degrees celsius")]
	Celsius,
	#[strum(serialize="K", to_string="kelvin")]
	Kelvin,
	// Imperial
	#[strum(serialize="F", to_string="degrees fahrenheit")]
	Fahrenheit,
	#[strum(serialize="R", to_string="degrees Rankine")]
	Rankine,
	
	// Data
	#[strum(serialize="b", to_string="bits")]
	Bit,
	#[strum(serialize="B", to_string="bytes")]
	Byte,
	// Metric
	#[strum(serialize="kb", to_string="kilobits")]
	Kilobit,
	#[strum(serialize="kB", to_string="kilobytes")]
	Kilobyte,
	#[strum(serialize="Mb", to_string="megabits")]
	Megabit,
	#[strum(serialize="MB", to_string="megabytes")]
	Megabyte,
	#[strum(serialize="Gb", to_string="gigabits")]
	Gigabit,
	#[strum(serialize="GB", to_string="gigabytes")]
	Gigabyte,
	#[strum(serialize="Tb", to_string="terabits")]
	Terabit,
	#[strum(serialize="TB", to_string="terabytes")]
	Terabyte,
	// IEC
	#[strum(serialize="Kib", to_string="kibibits")]
	Kibibit,
	#[strum(serialize="KiB", to_string="kibibytes")]
	Kibibyte,
	#[strum(serialize="Mib", to_string="mebibits")]
	Mebibit,
	#[strum(serialize="MiB", to_string="mebibytes")]
	Mebibyte,
	#[strum(serialize="Gib", to_string="gibibits")]
	Gibibit,
	#[strum(serialize="GiB", to_string="gibibytes")]
	Gibibyte,
	#[strum(serialize="Tib", to_string="tebibits")]
	Tebibit,
	#[strum(serialize="TiB", to_string="tebibytes")]
	Tebibyte,
}

type ConversionFunc = fn(f64) -> f64;

static CONVERSIONS: LazyLock<HashMap<Unit, HashMap<Unit, ConversionFunc>>> = LazyLock::new(|| {
	make_table! {
		Unit::Metre -> {
			Unit::Yard => div 0.9144,
			Unit::Millimetre => div MILLI,
			Unit::Centimetre => div CENTI,
			Unit::Kilometre => div KILO,
		},
		Unit::Yard -> {
			Unit::Inch => mul 36.0,
			Unit::Foot =>  mul 3.0,
			Unit::Mile =>  div 1760.0,
		}, 
		
		Unit::Celsius -> {
			Unit::Kelvin => add 273.15,
			Unit::Fahrenheit => fun(|c| 9.0*c/5.0 + 32.0),
		},
		Unit::Fahrenheit -> {
			Unit::Rankine => add 459.67,
			Unit::Celsius => fun(|f| (f-32.0)*5.0/9.0),
		},
		
		Unit::Bit -> {
			Unit::Kilobit => div KILO,
			Unit::Kibibit => div KIBI,
			Unit::Megabit => div MEGA,
			Unit::Mebibit => div MEBI,
			Unit::Gigabit => div GIGA,
			Unit::Gibibit => div GIBI,
			Unit::Terabit => div TERA,
			Unit::Tebibit => div TEBI,
		},
		Unit::Byte -> {
			Unit::Bit => mul 8.0,
			Unit::Kilobyte => div KILO,
			Unit::Kibibyte => div KIBI,
			Unit::Megabyte => div MEGA,
			Unit::Mebibyte => div MEBI,
			Unit::Gigabyte => div GIGA,
			Unit::Gibibyte => div GIBI,
			Unit::Terabyte => div TERA,
			Unit::Tebibyte => div TEBI,
		},
	}
});

fn do_conversion(amount: f64, input_unit: Unit, output_unit: Unit) -> Result<f64, &'static str> {
	let Some(conversion_path) = find_conversion_path(&input_unit, &output_unit) else {
		return Err("Cannot convert those units.");
	};
	println!("{conversion_path:?}");
	Ok(apply_conversion(amount, conversion_path))
}

fn apply_conversion(amount: f64, conversion_path: Vec<Unit>) -> f64 {
	let mut amount = amount;
	for i in 0..conversion_path.len() - 1 {
		let input_unit = conversion_path[i];
		let output_unit = conversion_path[i + 1];
		print!("{amount} {input_unit} = ");
		amount = CONVERSIONS[&input_unit][&output_unit](amount);
		println!("{amount} {output_unit}");
	}
	amount
}

fn find_conversion_path(input_unit: &Unit, output_unit: &Unit) -> Option<Vec<Unit>> {
	let mut queue: VecDeque<(&Unit, Vec<&Unit>)> = VecDeque::new();
	let mut seen: HashSet<&Unit> = HashSet::new();
	queue.push_back((input_unit, vec![]));
	seen.insert(input_unit);
	while !queue.is_empty() {
		let (unit, mut parents) = queue.pop_front().unwrap();
		if unit == output_unit {
			parents.push(unit);
			return Some(parents.into_iter().cloned().collect());
		}
		let Some(children) = CONVERSIONS.get(unit) else {
			continue;
		};
		for child in children.keys() {
			if !seen.contains(child) {
				seen.insert(child);
				let mut parents = parents.clone();
				parents.push(unit);
				queue.push_back((child, parents));
			}
		}
	}
	None
}

fn main() -> Result<(), &'static str> {
	let mut args = env::args().skip(1);
	let amount = args.next().ok_or("Need amount")?.parse::<f64>().or(Err("Invalid amount"))?;
	let from_unit = args.next().ok_or("Need origin unit")?.parse::<Unit>().or(Err("Invalid from unit"))?;
	let to_unit = args.next().ok_or("Need destination unit")?.parse::<Unit>().or(Err("Invalid to unit"))?;

	println!("Converting {amount} {from_unit} to {to_unit}");
	let result = do_conversion(amount, from_unit, to_unit)?;
	println!("{result}");
	Ok(())
}
