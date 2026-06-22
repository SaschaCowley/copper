use clap::Parser;
use libcopper::{Unit, do_conversion};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
	/// Quantity to convert
	quantity: f64,
	/// Input unit
	#[arg(id = "FROM")]
	input_unit: Unit,
	/// Output unit
	#[arg(id = "TO")]
	output_unit: Unit,
}

fn main() -> Result<(), &'static str> {
	let cli = Cli::parse();
	println!("Converting {}  {} to {}", cli.quantity, cli.input_unit, cli.output_unit);
	let result = do_conversion(cli.quantity, cli.input_unit, cli.output_unit)?;
	println!("{result}");
	Ok(())
}
