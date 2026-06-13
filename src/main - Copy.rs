use std::str::FromStr;
/*
expression = operand [operator expression]
operand = integer
integer = digit-excluding-zero {digit}
digit-excluding-zero = "1"-"9"
digit = "0" | digit-excluding-zero
operator = "+" | "-" | "*" | "/"
*/

// enum Kind {
	// _Length,
	// Temperature,
// }

// type ConversionFunction = fn(f64) -> f64;
// struct Unit {
	// from_base: ConversionFunction,
	// to_base: ConversionFunction,
	// _kind: Kind,
// }

// impl Unit {
	// fn new(from_base: ConversionFunction, to_base: ConversionFunction, _kind: Kind) -> Self {
		// Self { from_base, to_base, _kind }
	// }
	
	// pub fn convert_from(&self, value: f64) -> f64 {
		// (self.to_base)(value)
	// }
	
	// pub fn convert_to(&self, value: f64) -> f64 {
		// (self.from_base)(value)
	// }
// }

// struct 
fn main() {
	// let a = Operation::Add(Value::Num(1), Value::Num(2));
	// let b = Operation::Mul(Value::Num(3), Value::Num(4));
	// println!("{:?}, {:?}", a, b);
	// let _celcius = Unit::new(|t| t, |t| t, Kind::Temperature);
	// let fahrenheit = Unit::new(|t| (9. * t / 5.) + 32., |t| 5. * (t - 32.) / 9., Kind::Temperature);
	// let kelvin = Unit::new(|t| t + 273.15, |t| t + 273.15, Kind::Temperature);
	// let deg_f = 70.;
	// let deg_c = fahrenheit.convert_from(deg_f);
	// let deg_k = kelvin.convert_to(deg_c);
	// println!("{}f = {}c = {}k", deg_f, deg_c, deg_k);
	let program = "101234";
	let parsed = parse(program);
	println!("From: {:?}; to: {:?}", program, parsed);
}

struct CodeScanner {
	cursor: usize,
	code: Vec<char>,
}

impl CodeScanner {
	fn new(code: &str) -> Self {
		Self {cursor: 0, code: code.chars().collect() }
	}
	
	fn peak(&self) -> Option<&char> {
		self.code.get(self.cursor)
	}
	
	fn pop(&mut self) -> Option<&char> {
		if let Some(character) = self.code.get(self.cursor) {
			self.cursor += 1;
			Some(character)
		} else {
			None
		}
	}
	
	fn accept(&mut self, predicate: impl FnOnce(&char) -> bool) -> Option<&char> {
		if let Some(character) = self.peak() {
			if predicate(character) { self.pop()} else { None }
		} else {
			None
		}
	}
	
	fn expect(&mut self, predicate: impl FnOnce(&char) -> bool) -> Result<&char, &'static str> {
		if let Some(character) = self.accept(predicate) {
			Ok(character)
		} else {
			Err("Unexpected character")
		}
	}
	
	fn is_exhausted(&self) -> bool { self.cursor == self.code.len() }
}

#[derive(Debug)]
struct Integer(i128);

fn parse(string: &str) -> Result<Integer, &'static str> {
	let mut scanner = CodeScanner::new(string);
	let number = integer(&mut scanner)?;
	if  scanner.is_exhausted() { Ok(number) } else { Err("Expected the string to be fully consumed") } 
}

fn integer(scanner: &mut CodeScanner) -> Result<Integer, &'static str> {
	let mut string = String::new();
	string.push(*scanner.expect(|character| ('1'..='9').contains(character))?);
	loop {
		match scanner.accept(|character| ('0'..='9').contains(character)) {
			Some(character) => string.push(*character),
			None => break,
		}
	}
	Ok(Integer(i128::from_str(&string).expect("Should be a valid integer")))
}