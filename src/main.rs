use std::{
	collections::{HashMap, HashSet, VecDeque},
	env, fmt,
	str::FromStr,
	sync::LazyLock,
};

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
enum Unit {
	Centimeter,
	Meter,
	Kilometer,
	Foot,
	Yard,
	Mile,
}

struct ParseUnitError;
impl FromStr for Unit {
	type Err = ParseUnitError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"m" => Ok(Unit::Meter),
			"cm" => Ok(Unit::Centimeter),
			"km" => Ok(Unit::Kilometer),
			"yd" => Ok(Unit::Yard),
			"ft" => Ok(Unit::Foot),
			"mi" => Ok(Unit::Mile),
			_ => Err(ParseUnitError),
		}
	}
}

impl fmt::Display for Unit {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Meter => "meter",
				Self::Centimeter => "centimeter",
				Self::Kilometer => "kilometer",
				Self::Yard => "yard",
				Self::Foot => "foot",
				Self::Mile => "mile",
			}
		)
	}
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
		Unit::Meter: {
			Unit::Yard: |m| m/0.9144,
			Unit::Centimeter: |m| m*100.,
			Unit::Kilometer: |m| m/1000.,
		},
		Unit::Yard: {
			Unit::Meter: |yd| 0.9144*yd,
			Unit::Foot: |yd| yd/3.,
			Unit::Mile: |yd| yd/1760.,
		},
		Unit::Foot: {Unit::Yard: |ft| 3.*ft},
		Unit::Mile: {Unit::Yard: |mi| 1760.*mi},
		Unit::Centimeter: {Unit::Meter: |cm| cm/100.},
		Unit::Kilometer: {Unit::Meter: |km| km*1000.},
	}
});

fn do_conversion(amount: f64, input_unit: Unit, output_unit: Unit) -> Result<f64, &'static str> {
	if input_unit == output_unit {
		return Ok(amount);
	}
	let Some(conversion_path) = find_conversion_path(&input_unit, &output_unit) else {
		return Err("Cannot convert those units.");
	};
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

	println!("Converting {amount}{from_unit} to {to_unit}");
	let result = do_conversion(amount, from_unit, to_unit)?;
	println!("{result}");
	Ok(())
}
