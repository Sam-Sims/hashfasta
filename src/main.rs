use clap::Parser;
use env_logger::Builder;
use std::io::Write;

mod cli;
mod hash;
mod hashfasta;
mod parser;

pub use crate::cli::Cli;
pub use crate::hashfasta::Hashfasta;

fn init_logging() {
    let mut builder = Builder::from_env(env_logger::Env::default().default_filter_or("info"));
    builder.format(|buf, record| {
        let style = buf.default_level_style(record.level());
        write!(buf, "[{style}{}{style:#}]", record.level())?;
        writeln!(buf, " {}", record.args())
    });
    builder.init();
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    init_logging();
    let args = Cli::parse();
    let app = Hashfasta::new(args);
    app.run()
}
