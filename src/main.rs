use clap::{arg, command, Command};
use tracing_subscriber;

use hapctl;

fn main() {
    tracing_subscriber::fmt::init();

    let matches = command!()
        .arg(arg!(-e --endpoint <ENDPOINT> "Override the endpoint URL the client tries to connect to. Default is to auto-detect").required(false))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("get-weight")
                .about("Check current server weights")
                .arg(arg!(<SERVERNAME>))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("set-weight")
                .about("Set a weight for a server")
                .arg(arg!(<SERVERNAME>))
                .arg(
                    arg!(<WEIGHT>)
                        .validator(|s| s.parse::<u32>())
                )
                .arg(arg!(<REASON>))
                .arg_required_else_help(true),
        )
        .get_matches();

    let endpoint = matches.value_of("endpoint");

    match matches.subcommand() {
        Some(("get-weight", sub_matches)) => {
            let name = sub_matches.value_of("SERVERNAME").unwrap();
            let client = hapctl::Client::new(name, endpoint);

            println!(
                "{}",
                client.get_weight(&name).unwrap_or_else(|c| c.to_string())
            );
        }
        Some(("set-weight", sub_matches)) => {
            let name = sub_matches.value_of("SERVERNAME").unwrap();
            let weight = sub_matches
                .value_of("WEIGHT")
                .unwrap()
                .parse::<u32>()
                .unwrap();
            let reason = sub_matches.value_of("REASON").unwrap();
            let client = hapctl::Client::new(name, endpoint);

            println!(
                "{}",
                client
                    .set_weight(&name, weight, &reason)
                    .unwrap_or_else(|c| c.to_string())
            );
        }
        _ => unreachable!("No subcommand found"),
    }
}
