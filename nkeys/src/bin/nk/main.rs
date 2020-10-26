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

    //Paths
    let path = "C:\\Users\\emmanuel\\Documents\\Projet_IOT\\nkeys\\";
    //let path = ""; 

    //Names of the 2 files that will store the keys
    let filename_seed = "alice_seed.txt";
    let filename_pk = "alice_pk.txt";

    let file_seed = format!("{}{}", path, filename_seed);
    let file_pk = format!("{}{}", path, filename_pk);
    

    let payload = json!({
    "id": "urn:uuid:ef5a7369-f0b9-4143-a49d-2b9c7ee51117",
    "type": "didcomm",
    "from": "did:example:alice",
    "expiry": 1516239022,
    "time_stamp": 1516269022,
    "body": { "message": "Challenge!" }
    });

    let payload_string = serde_json::to_vec(&payload).unwrap();

    //Try to read the file to get the seed 
    let mut sender_seed = fs::read_to_string(&file_seed);
    if sender_seed.is_err() { 
        //If it doesn't exist, we create it and update/create the public one
        let kp = KeyPair::new(KeyPairType::User);

        let f1 = fs::write(&file_seed, &kp.seed().unwrap());
        if f1.is_err() {
            panic!("Cannot write the seed file");
        };

        let f2 = fs::write(&file_pk, &kp.public_key());
        if f2.is_err() {
            panic!("Cannot write the public key file");
        };

        //Now that's it's written, we need to update sender_seed
        sender_seed = fs::read_to_string(&file_seed);
        if sender_seed.is_err(){
            panic!("Cannot find the seed file");
        }
    }

    let sender_seed_str: &str = &sender_seed.unwrap();

    //Sender signs using the keypair from seed
    let sender_kp = KeyPair::from_seed(&sender_seed_str).unwrap();        
    let sig = sender_kp.sign(&payload_string).unwrap();

    //Receiver have access only to the public file
    let sender_pk = fs::read_to_string(&file_pk);
    if sender_pk.is_err() {
        panic!("Cannot find the public key file");
    }
    let sender_pub_kp = KeyPair::from_public_key(&sender_pk.unwrap()).unwrap();
    //wrong public key if needed : UCVLXNOAAD72JVJBZ67OETRJKTPJ6FVAZXXMTKDBGHFYFAD32LJQE246

    //Check the signature
    let res = sender_pub_kp.verify(&payload_string,&sig.as_slice());
    match res {
        Ok(()) => println!("The message is from Alice"),
        Err(_e) => println!("The message is not from Alice or the public key isn't updated"), 
    }
    
}