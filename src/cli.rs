use clap::{App, Arg, SubCommand, ArgMatches};

pub fn build() -> ArgMatches<'static> {
  return App::new("mtl-info")
    .version("1.0")
    .author("Patrick M. <git@metcalfe.rocks>")
    .about("Read's information from metallib files.")
    .arg(
      Arg::with_name("INPUT")
        .help("Sets the input file to use")
        .required(true)
        .index(1),
    )
    .arg(
      Arg::with_name("entries")
        .short("l")
        .long("list-entries")
        .requires("INPUT")
        .conflicts_with("count")
        .help("Returns the entries' names and offsets"),
    )
    .arg(
      Arg::with_name("count")
        .short("n")
        .long("count")
        .requires("INPUT")
        .help("Return number of functions found in INPUT"),
    )
    .arg(
      Arg::with_name("verbosity")
        .long("verbosity")
        .takes_value(true)
        .default_value("1")
        .help("Set's the logger level. Between 1 and 4"),
    )
    .subcommand(
      SubCommand::with_name("bitcode")
      .about("Print's bitcode of entry"),
    )
    .get_matches();
}
