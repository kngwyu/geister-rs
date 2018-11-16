#[macro_use]
extern crate derive_more;
use clap;
mod rnghandle;
mod yowagoshi;
mod metadata;
mod random;

pub use self::random::RandomAi;
pub use self::yowagoshi::Player as YowagoshiAi;

pub fn args<'a, 'b>(app: clap::App<'a, 'b>) -> clap::ArgMatches<'a> {
    app.arg(
        clap::Arg::with_name("id")
            .short("i")
            .long("id")
            .value_name("ID")
            .help("Player ID")
            .required(true)
            .takes_value(true),
    ).arg(
        clap::Arg::with_name("addr")
            .short("a")
            .long("addr")
            .value_name("ADDR")
            .help("ip address")
            .required(true)
            .takes_value(true),
    )
    .subcommand(
        clap::SubCommand::with_name("random")
            .about("random player")
            .version("0.1"),
    )
    .subcommand(
        clap::SubCommand::with_name("yowagoshi")
            .about("yowagoshi player")
            .version("0.1"),
    )
    .get_matches()
}
