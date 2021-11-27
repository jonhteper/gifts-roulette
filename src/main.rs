use crate::roulette::Roulette;
use crate::store::read_excel;
use clap::{App, Arg};
use delay_timer::anyhow::Context;
use eyre::{eyre, ContextCompat, Result, WrapErr};
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time::timeout;

mod daemon;
mod roulette;
mod send;
mod store;

#[macro_use]
extern crate serde_derive;
extern crate clap;

const START_COMMAND: &str = "start";
const DAEMON_COMMAND: &str = "daemon";

#[derive(Debug, Clone)]
struct Args {
    action: String,
    excel_path: String,
    store_path: String,
}

fn cli_args() -> Result<Args> {
    let matches = App::new("Gifts Roulette")
        .version("0.1.0")
        .author("jonhteper <jonhteper@triamseletea.com>")
        .about("Gift exchange program, based in excel file store and email")
        .arg(
            Arg::with_name("COMMAND")
                .help("Action for tool: \"start\" or \"daemon\"")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("input")
                .help("Sets the excel input file. Default: input.xlsx")
                .short("i")
                .value_name("INPUT-FILE")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .help("Sets the output json file to save the couples. Default: db.json")
                .value_name("OUTPUT-FILE")
                .takes_value(true),
        )
        .get_matches();

    let args = Args {
        action: matches
            .value_of("COMMAND")
            .wrap_err("Error parsing cli value")?
            .to_string(),
        excel_path: matches
            .value_of("input")
            .unwrap_or("input.xlsx")
            .clone()
            .to_string(),
        store_path: matches
            .value_of("output")
            .unwrap_or("db.json")
            .clone()
            .to_string(),
    };

    Ok(args)
}

fn shuffle_and_send(args: &Args) -> Result<()> {
    let participants =
        read_excel(PathBuf::from(&args.excel_path)).wrap_err("Error parsing excel file")?;
    let mut roulette = Roulette::new(participants, &args.store_path)
        .wrap_err("Error creating roulette to shuffle participants")?;
    let _ = roulette
        .run()
        .wrap_err("Error running shuffle function and saving data")?;

    let _ = roulette.send_emails().wrap_err("Error sending emails")?;

    Ok(())
}

fn daemon(args: &Args) -> Result<()> {
    println!("Daemon starts");
    Ok(())
}

fn cli() -> Result<()> {
    let args = cli_args().wrap_err("Error parsing args")?;
    let action = args.action.as_str();
    match &action {
        &START_COMMAND => shuffle_and_send(&args)?,
        &DAEMON_COMMAND => daemon(&args)?,
        _ => println!("Use only {} or {}", START_COMMAND, DAEMON_COMMAND),
    };

    Ok(())
}

fn main() {
    println!("Starting Gift Exchange...");
    match cli() {
        Err(e) => panic!("Fatal error:\n{:?}", e),
        Ok(()) => println!("No errors"),
    }
}
