use std::io::Write;

use anyhow::{Context, Result};
use clap::{Args, Parser};
use libcopper::{Unit, do_conversion};
use log::{LevelFilter, info};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
	#[command(flatten)]
	convert: Option<Convert>,
	/// Output verbosity
	#[arg(short, long, action = clap::ArgAction::Count)]
	verbosity: u8,
	/// List supported units
	#[arg(short, long, conflicts_with = "Convert")]
	list: bool,
}

#[derive(Args)]
struct Convert {
	/// Quantity to convert
	quantity: f64,
	/// Input unit
	#[arg(id = "FROM")]
	input_unit: Unit,
	/// Output unit
	#[arg(id = "TO")]
	output_unit: Unit,
}

fn main() -> Result<()> {
	let cli = Cli::parse();
	env_logger::Builder::new()
		.filter(
			None,
			match cli.verbosity {
				0..=1 => LevelFilter::Error,
				2 => LevelFilter::Info,
				3.. => LevelFilter::Debug,
			},
		)
		.format(|buf, record| writeln!(buf, "{}", record.args()))
		.init();
	if cli.list {
		list_units();
	} else {
		let Convert { quantity, input_unit, output_unit } = cli.convert.with_context(|| "No conversion specified")?;
		info!(
			"Converting {} {} to {}",
			quantity,
			if quantity == 1.0 { input_unit.name() } else { input_unit.plural() },
			output_unit.plural()
		);
		let result = do_conversion(quantity, input_unit, output_unit)
			.with_context(|| format!("Failed to convert {}{} to {}", quantity, input_unit, output_unit))?;
		if cli.verbosity == 0 {
			println!("{result}");
		} else {
			println!("{}{} = {}{}", quantity, input_unit.symbol(), result, output_unit.symbol());
		}
	}
	Ok(())
}

fn list_units() {
	println!("Supported units:");
	for unit in Unit::iter() {
		println!("{} ({})", unit.plural(), unit.symbols().join(", "));
	}
}
