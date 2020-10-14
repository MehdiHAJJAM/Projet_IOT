extern crate serde_json;

use nkeys::{self, KeyPair, KeyPairType};
use serde_json::json;
use std::error::Error;
use std::fmt;
use std::str::FromStr;
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(Debug, StructOpt, Clone)]
#[structopt(
    global_settings(&[AppSettings::ColoredHelp, AppSettings::VersionlessSubcommands]),
    name = "nk",
    about = "A tool for manipulating nkeys"
)]
struct Cli {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug, Clone)]
enum Command {
    #[structopt(name = "gen", about = "Generates a key pair")]
    Gen {
        /// The type of key pair to generate. May be Account, User, Module, Server, Operator, Cluster
        #[structopt(case_insensitive = true)]
        keytype: KeyPairType,
        #[structopt(
            short = "o",
            long = "output",
            default_value = "text",
            help = "Specify output format (text or json)"
        )]
        output: Output,
    },
    Didcomm{

    }
}

#[derive(StructOpt, Debug, Clone)]
enum Output {
    Text,
    JSON,
}

impl FromStr for Output {
    type Err = OutputParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(Output::JSON),
            "text" => Ok(Output::Text),
            _ => Err(OutputParseErr),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OutputParseErr;

impl Error for OutputParseErr {}

impl fmt::Display for OutputParseErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            "error parsing output type, see help for the list of accepted outputs"
        )
    }
}

fn main() {
    let args = Cli::from_args();
    let cmd = &args.cmd;
    env_logger::init();

    match cmd {
        Command::Gen { keytype, output } => {
            generate(keytype, output);
        }

        Command::Didcomm {} => {
            didcomm();
        }
    }
}

fn generate(kt: &KeyPairType, output_type: &Output) {
    let kp = KeyPair::new(kt.clone());
    match output_type {
        Output::Text => {
            println!(
                "Public Key: {}\nSeed: {}\n\nRemember that the seed is private, treat it as a secret.",
                kp.public_key(),
                kp.seed().unwrap()
            );
        }
        Output::JSON => {
            let output = json!({
                "public_key": kp.public_key(),
                "seed": kp.seed().unwrap(),
            });

            println!("{}", output.to_string());
        }
    }
}


fn didcomm(){
    let item = json!({
    "id": "urn:uuid:ef5a7369-f0b9-4143-a49d-2b9c7ee51117",
    "type": "didcomm",
    "from": "did:example:alice",
    "expiry": 1516239022,
    "time_stamp": 1516269022,
    "body": { "message": "Challenge!" }
});

//let response = serde_json::to_string(&item).unwrap();
println!("Payload is : {}", item);
}