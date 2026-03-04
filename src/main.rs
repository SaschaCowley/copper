use std::{
	collections::{HashMap, HashSet, VecDeque},
	env,
	sync::LazyLock,
};
use strum_macros::EnumString;

const QUETTA: f64 = 10e30; // Q
const RONNA: f64 = 10e27; // R
const YOTTA: f64 = 10e24; // Y
const ZETTA: f64 = 10e21; // A
const EXA: f64 = 10e18; // E
const PETA: f64 = 10e15; // P
const TERA: f64 = 10e12; // T
const GIGA: f64 = 10e9 ; // G
const MEGA: f64 = 10e6 ; // M
const KELO: f64 = 10e3 ; // k
const HECTO: f64 = 10e2; // h
const DECA: f64 = 10e1 ; // da
const DECI: f64 = 10e-1 ; // d
const CENTI: f64 = 10e-2 ; // c
const MILLI: f64 = 10e-3 ; // m
const MICRO: f64 = 10e-6 ; // μ/u
const NANO: f64 = 10e-9 ; // n
const PICO: f64 = 10e-12; // p
const FEMTO: f64 = 10e-15; // f
const ATTO: f64 = 10e-18; // a
const ZEPTO: f64 = 10e-21; // z
const YOCTO: f64 = 10e-24; // y
const RONTO: f64 = 10e-27; // r
const QUECTO: f64 = 10e-30; // q

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy, EnumString, strum_macros::Display)]
enum Unit {
	// Length
	#[strum(serialize="cm", to_string="centimetres")]
	Centimetre,
	#[strum(serialize="m", to_string="metres")]
	Metre,
	#[strum(serialize="km", to_string="kilometres")]
	Kilometre,
	#[strum(serialize="ft", to_string="feet")]
	Foot,
	#[strum(serialize="yd", to_string="yards")]
	Yard,
	#[strum(serialize="mi", to_string="miles")]
	Mile,
	
	// Temperature
	
	#[strum(serialize="f", to_string="degrees fahrenheit")]
	Fahrenheit,
	#[strum(serialize="c", to_string="degrees celsius")]
	Celsius,
}

type ConversionFunc = fn(f64) -> f64;

macro_rules! make_table {
	( $( $( $input_unit:path: { $( $( $output_unit:path: $func:expr ),+ $(,)? )? } ),+ $(,)? )? ) => {
		HashMap::from([
			$($((
				$input_unit,
				HashMap::from([
					$($((
						$output_unit,
						$func as ConversionFunc
					)),*)?
				])
			)),*)?
		])
	}
}


static CONVERSIONS: LazyLock<HashMap<Unit, HashMap<Unit, ConversionFunc>>> = LazyLock::new(|| {
	make_table! {
		Unit::Metre: {
			Unit::Yard: |m| m/0.9144,
			Unit::Centimetre: |m| m*100.0,
			Unit::Kilometre: |m| m/1000.0,
		},
		Unit::Yard: {
			Unit::Metre: |yd| 0.9144*yd,
			Unit::Foot: |yd| yd/3.0,
			Unit::Mile: |yd| yd/1760.0,
		},
		Unit::Foot: {Unit::Yard: |ft| 3.0*ft},
		Unit::Mile: {Unit::Yard: |mi| 1760.0*mi},
		Unit::Centimetre: {Unit::Metre: |cm| cm/100.0},
		Unit::Kilometre: {Unit::Metre: |km| km*1000.0},
		Unit::Celsius: { Unit::Fahrenheit: |c| 9.0*c/5.0 + 32.0, },
		Unit::Fahrenheit: { Unit::Celsius: |f| (f-32.0)*5.0/9.0, },
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
