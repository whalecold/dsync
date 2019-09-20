use clap::{crate_version, App, Arg, SubCommand};

mod info;
mod runtime;

fn main() {
    info::info();
    let matches = App::new("dsync")
        .version(crate_version!())
        .author("whalecold")
        .about("a simple tool for transfer docker images")
        .subcommand(
            SubCommand::with_name("web")
                .alias("w")
                .about("start a web server"),
        )
        .subcommand(
            SubCommand::with_name("file")
                .alias("f")
                .about("watch a file")
                .arg(
                    Arg::with_name("config")
                        .short("c")
                        .long("config")
                        .takes_value(true)
                        .required(true)
                        .value_name("FILE")
                        .help("Specify config file"),
                ),
        )
        .get_matches();
    if let Some(matches) = matches.subcommand_matches("web") {
        runtime::web::run(matches.clone());
    }
    if let Some(matches) = matches.subcommand_matches("file") {
        runtime::file::run(matches.clone());
    }
}
