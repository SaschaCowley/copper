enum Kind {
	Length,
	Temperature,
}

type ConversionFunction = fn(f64) -> f64;
struct Unit {
	from_base: ConversionFunction,
	to_base: ConversionFunction,
	kind: Kind,
}

impl Unit {
	fn new(from_base: ConversionFunction, to_base: ConversionFunction, kind: Kind) -> Self {
		Self { from_base, to_base, kind }
	}
	
	pub fn convert_from(&self, value: f64) -> f64 {
		(self.to_base)(value)
	}
	
	pub fn convert_to(&self, value: f64) -> f64 {
		(self.from_base)(value)
	}
}

// struct 
fn main() {
	// let a = Operation::Add(Value::Num(1), Value::Num(2));
	// let b = Operation::Mul(Value::Num(3), Value::Num(4));
	// println!("{:?}, {:?}", a, b);
	let celcius = Unit::new(|t| t, |t| t, Kind::Temperature);
	let fahrenheit = Unit::new(|t| (9. * t / 5.) + 32., |t| 5. * (t - 32.) / 9., Kind::Temperature);
	let kelvin = Unit::new(|t| t + 273.15, |t| t + 273.15, Kind::Temperature);
	let deg_f = 70.;
	let deg_c = fahrenheit.convert_from(deg_f);
	let deg_k = kelvin.convert_to(deg_c);
	println!("{}f = {}c = {}k", deg_f, deg_c, deg_k);
}
