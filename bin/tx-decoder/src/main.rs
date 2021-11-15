// Copyright 2021 Parity Technologies (UK) Ltd.
// This file is part of substrate-desub.
//
// substrate-desub is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// substrate-desub is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with substrate-desub.  If not, see <http://www.gnu.org/licenses/>.

mod app;
mod queries;

use anyhow::Error;
use colored::Colorize;
use fern::colors::{Color, ColoredLevelConfig};

#[async_std::main]
async fn main() -> Result<(), Error> {
	let app: self::app::App = argh::from_env();
	let level = if app.verbose { log::LevelFilter::Trace } else { log::LevelFilter::Warn };
	let colors =
		ColoredLevelConfig::new().trace(Color::Magenta).error(Color::Red).debug(Color::Blue).info(Color::Green);

	// Configure logger at runtime
	fern::Dispatch::new()
		.level(log::LevelFilter::Error)
		.level_for("desub_legacy", level)
		.level_for("desub_current", level)
		.level_for("desub_json_resolver", level)
		.level_for("tx_decoder", level)
		.format(move |out, message, record| {
			out.finish(format_args!(
				" {} {}::{}		>{} ",
				colors.color(record.level()),
				record.target().bold(),
				record.line().map(|l| l.to_string()).unwrap_or_default(),
				message,
			))
		})
		// Output to stdout, files, and other Dispatch configurations
		.chain(std::io::stdout())
		// Apply globally
		.apply()?;

	app::app(app).await?;
	Ok(())
}
