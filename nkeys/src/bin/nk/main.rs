extern crate serde_json;

use nkeys::{self, KeyPair, KeyPairType, receive_stream, send_stream, verify_seed};
use serde_json::json;
use serde_json::{Value};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::fs;
use std::str::{FromStr, from_utf8};
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

    #[structopt(name = "didserver", about = "Open the server receving from the client")]
    Didserver{
        #[structopt(help = "Specify your command (getpk or getmessage)")]
        cmd: CommandServer,
    },
    #[structopt(name = "didclient", about = "Open the client sending to the server")]
    Didclient{
        #[structopt(help = "Specify your command (sendpk or sendmessage)")]
        cmd: CommandClient,

    }
}

//commands used by the client
#[derive(StructOpt, Debug, Clone)]
enum CommandClient {
    SendPk,
    SendMessage,
}

impl FromStr for CommandClient {
    type Err = OutputParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sendpk" => Ok(CommandClient::SendPk),
            "sendmessage" => Ok(CommandClient::SendMessage),
            _ => Err(OutputParseErr),
        }
    }
}

//commands used by the server
#[derive(StructOpt, Debug, Clone)]
enum CommandServer {
    GetPk,
    GetMessage,
}

impl FromStr for CommandServer {
    type Err = OutputParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "getpk" => Ok(CommandServer::GetPk),
            "getmessage" => Ok(CommandServer::GetMessage),
            _ => Err(OutputParseErr),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CommandParseErr;

impl Error for CommandParseErr {}

impl fmt::Display for CommandParseErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            "error parsing command type, see help for the list of accepted commands"
        )
    }
}

#[derive(StructOpt, Debug, Clone)]
enum Output {
    Text,
    JSON,
}

impl FromStr for Output {
    type Err = CommandParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(Output::JSON),
            "text" => Ok(Output::Text),
            _ => Err(CommandParseErr),
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

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    kid : String,
    payload : Value,
    signature : Vec<u8>,
}

fn main() {
    let args = Cli::from_args();
    let cmd = &args.cmd;
    env_logger::init();

    match cmd {
        Command::Gen { keytype, output } => {
            generate(keytype, output);
        }

        Command::Didserver { cmd } => {
            didserver(cmd);
        }

        Command::Didclient { cmd } => {
            didclient(cmd);
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

fn didserver(command_type: &CommandServer) {

    //PARAMS
    let path = "C:\\Users\\emmanuel\\Documents\\Projet_IOT\\nkeys\\keys\\Bob\\";
    //let path = "/Users/Mehdi/Desktop/Rust/Projet_IOT/nkeys/keys/Bob/";
    let filename_pk = "alice_pk.txt"; //File storing the public key
    let addr = "localhost:8000";

    let file_pk = format!("{}{}", path, filename_pk);
    
    // Receive the payload from the client
    let buffer = receive_stream(&addr).unwrap();

    match command_type{

        CommandServer::GetPk => {
            println!("Received a public key : {}", from_utf8(&buffer).unwrap());
            fs::write(&file_pk, &buffer).expect("Storing the public key failed");
        }

        CommandServer::GetMessage => {
            //Generate Keypair from stored public key
            let pk_sender = fs::read_to_string(&file_pk).expect("Reading the public key failed");
            let sender_pub_kp = KeyPair::from_public_key(&pk_sender).unwrap();

            //Unwrap the message    
            let message : Message = serde_json::from_slice(&buffer).unwrap();
            let payload_vec = serde_json::to_vec(&message.payload).unwrap();
            let payload_display = serde_json::to_string_pretty(&message.payload).unwrap();

            //Check the signature
            let res = sender_pub_kp.verify(&payload_vec,&message.signature);

            //Result
            if res.is_ok(){
                println!("You received a verified message signed with {} : \n {}", &message.kid, &payload_display);
            } else {
                println!("You received an unverified message (signature {} doesn't match) ", &message.kid);
            }
        }    
    }
}

fn didclient(command_type: &CommandClient){

    //PARAMS
    let path = "C:\\Users\\emmanuel\\Documents\\Projet_IOT\\nkeys\\keys\\Alice\\";
    //let path = "/Users/Mehdi/Desktop/Rust/Projet_IOT/nkeys/keys/Alice/";
    let filename_seed = "alice_seed.txt"; 
    let addr = "localhost:8000";

    let file_seed = format!("{}{}", path, filename_seed);
    verify_seed(&file_seed).expect("Verifying seed file failed");
    let seed_sender = fs::read_to_string(&file_seed).expect("Reading seed failed");
    let kp_sender = KeyPair::from_seed(&seed_sender).unwrap();
    let packet;

    match command_type {

        CommandClient::SendPk => {
            packet = kp_sender.public_key().as_bytes().to_vec();
            println!("You're sending your public key: {}", from_utf8(&packet).unwrap());
        }

        CommandClient::SendMessage => {
            //Define the payload
            let payload = json!({
                "id": "urn:uuid:ef5a7369-f0b9-4143-a49d-2b9c7ee51117",
                "type": "didcomm",
                "from": "did:example:alice",
                "expiry": 1516239022,
                "time_stamp": 1516269022,
                "body": { "message": "Challenge!" }
            });
            
            //Sign the payload
            let payload_vec = serde_json::to_vec(&payload).unwrap();
            let sig = kp_sender.sign(&payload_vec).unwrap();

            //Wrap the message
            let message = json!({
                "kid": "ed25519",
                "payload" : payload,
                "signature": sig, 
            });
            packet = serde_json::to_vec(&message).unwrap();

            println!("You're sending a signed message");
        }
    }
    send_stream(&addr, &packet).expect("Sending the message failed");    
}