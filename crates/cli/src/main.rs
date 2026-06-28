use std::io::Write;

use anyhow::{Context, Result};
use clap::Parser;
use libcopper::{Unit, do_conversion};
use log::{LevelFilter, info};

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
	/// Output verbosity
	#[arg(short, long, action = clap::ArgAction::Count)]
	verbosity: u8,
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
	info!(
		"Converting {} {} to {}",
		cli.quantity,
		if cli.quantity == 1.0 { cli.input_unit.name() } else { cli.input_unit.plural() },
		cli.output_unit.plural()
	);
	let result = do_conversion(cli.quantity, cli.input_unit, cli.output_unit)
		.with_context(|| format!("Failed to convert {}{} to {}", cli.quantity, cli.input_unit, cli.output_unit))?;
	if cli.verbosity == 0 {
		println!("{result}");
	} else {
		println!("{}{} = {}{}", cli.quantity, cli.input_unit.symbol(), result, cli.output_unit.symbol());
	}
	Ok(())
}
