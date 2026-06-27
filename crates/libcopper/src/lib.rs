use std::collections::{HashSet, VecDeque};

use copper_macros::{declare_units, make_table2};
use log::debug;
use thiserror::Error;

#[derive(Error, Debug, Clone, Eq, PartialEq)]
pub enum ConversionError {
	#[error("Cannot convert from {from} to {to}: no conversion path found.")]
	IncompatibleUnits { from: Unit, to: Unit },
}

#[derive(Error, Debug)]
pub enum ParseUnitError {
	#[error("Unknown unit: no unit with symbol \"{0}\" found.")]
	VariantNotFound(String),
}

declare_units! {
	pub Unit (
		// Length
		metric("metre", "metres", "m"),
		("inch", "inches", "″", "in"),
		("foot", "feet", "′", "ft"),
		("yard", "yards", "yd"),
		("mile", "miles", "mi"),

		// Data
		data("byte", "bytes", "B"),
		data("bit", "bits", "b"),

		// Temperature
		(Celsius, "degree celsius", "degrees celsius", "°C", "C"),
		("kelvin", "kelvins", "K"),
		(Fahrenheit, "degree fahrenheit", "degrees fahrenheit", "°F", "F"),
		(Rankine, "degree Rankine", "degrees Rankine", "°R", "R"),
	)
}

type ConversionFunc = fn(f64) -> f64;

make_table2! {
	pub CONVERSIONS<ConversionFunc> Unit {
		data(Bit, Byte),
		metric(Metre),
		Metre -> Yard => div 0.9144,
		Yard -> {
			Inch => mul 36.0,
			Foot =>  mul 3.0,
			Mile =>  div 1760.0,
		},
		Celsius -> {
			Kelvin => add 273.15,
			Fahrenheit => fun(|c| 9.0*c/5.0 + 32.0),
		},
		Fahrenheit -> {
			Rankine => add 459.67,
			Celsius => fun(|f| (f-32.0)*5.0/9.0),
		},
	}
}

pub fn do_conversion(amount: f64, input_unit: Unit, output_unit: Unit) -> Result<f64, ConversionError> {
	let Some(conversion_path) = find_conversion_path(&input_unit, &output_unit) else {
		return Err(ConversionError::IncompatibleUnits { from: input_unit, to: output_unit });
	};
	debug!("{conversion_path:?}");
	Ok(apply_conversion(amount, conversion_path))
}

fn apply_conversion(amount: f64, conversion_path: Vec<Unit>) -> f64 {
	let mut amount = amount;
	for i in 0..conversion_path.len() - 1 {
		let input_unit = conversion_path[i];
		let output_unit = conversion_path[i + 1];
		let new_amount = CONVERSIONS[&input_unit][&output_unit](amount);
		debug!("{amount}{input_unit} = {new_amount}{output_unit}");
		amount = new_amount;
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
