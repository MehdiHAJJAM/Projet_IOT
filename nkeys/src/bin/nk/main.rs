extern crate serde_json;

use nkeys::{self, KeyPair, KeyPairType};
use serde_json::json;
use std::error::Error;
use std::fmt;
use std::fs;
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


fn didcomm() {
    let payload = json!({
    "id": "urn:uuid:ef5a7369-f0b9-4143-a49d-2b9c7ee51117",
    "type": "didcomm",
    "from": "did:example:alice",
    "expiry": 1516239022,
    "time_stamp": 1516269022,
    "body": { "message": "Challenge!" }
    });

    //Check 
    //Check if file exists

    //Generate the sender key and write in the 2 files (priv et publ) if not :
    let kp = KeyPair::new(KeyPairType::User);
    println!("{}",kp.seed().unwrap());
    println!("{}",kp.public_key());

    //Read the seed from the file : TODO: move the file in a folder far from here
    let sender_seed = fs::read_to_string("alice.txt")
        .expect("Something went wrong reading the file");
    let sender_seed_str: &str = &sender_seed;
    println!("{}", sender_seed);
    //let sender_seed = "SUAEQAVUZ7A7CBKQGTOCUGSB4T35Z6VTF3OXTPU2MUWUJZTMOLWVXHQXUY";

    //Sender signs using the keypair from seed
    let sender_kp = KeyPair::from_seed(sender_seed_str).unwrap();
    let msg = b"this will be the payload in the future";
    let sig = sender_kp.sign(msg).unwrap();

    //Receiver have access only to the public file
    let sender_pub_kp = KeyPair::from_public_key("UDTAPJ42PAKIVF2ZFSKRNT7WBQVFB62NHUY5CRYE6ZBR64KOXFXQHPBQ").unwrap();
    let res = sender_kp.verify(msg,sig.as_slice()).unwrap();
    //assert(res.is_ok());
    
    //let response = serde_json::to_string(&payload).unwrap();
    //println!("Payload is : {}", payload);
    
}