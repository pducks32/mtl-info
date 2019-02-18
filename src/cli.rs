use clap::{App, Arg, ArgMatches, SubCommand};

pub fn build() -> ArgMatches<'static> {
  return App::new("mtl-info")
    .version("1.0")
    .author("Patrick M. <git@metcalfe.rocks>")
    .about("Read's information from metallib files.")
    .subcommand(SubCommand::with_name("bitcode").about("Print's bitcode of entry"))
    .subcommand(SubCommand::with_name("count").about("Print's number of entries"))
    .subcommand(
      SubCommand::with_name("list")
        .about("Print's list of entries")
        .aliases(&["ls"]),
    )
    .arg(
      Arg::with_name("INPUT")
        .help("Sets the input file to use")
        .required(true)
        .index(1),
    )
    .arg(
      Arg::with_name("verbosity")
        .long("verbosity")
        .takes_value(true)
        .default_value("1")
        .help("Set's the logger level. Between 1 and 4"),
    )
    .get_matches();
}
