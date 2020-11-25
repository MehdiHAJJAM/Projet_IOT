extern crate serde_json;

use nkeys::{self, KeyPair, KeyPairType};
use serde_json::json;
use std::error::Error;
use std::fmt;
use std::fs;
use std::fs::File;
use std::str::FromStr;
use structopt::clap::AppSettings;
use structopt::StructOpt;
use std::net::{TcpListener,TcpStream, Shutdown};
use std::io::{Read,Write};
use std::str::from_utf8;

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

fn didserver(command_type: &CommandServer) {

    //PARAMS
    let path = "C:\\Users\\emmanuel\\Documents\\Projet_IOT\\nkeys\\keys\\Bob\\";
    let filename_pk = "alice_pk.txt"; //File storing the public key
    let addr = "localhost:8000";

    let file_pk = format!("{}{}", path, filename_pk);

    // Receive the payload from the client
    let listener = TcpListener::bind(addr).unwrap();
    println!("Server Listening");
    let mut buffer = [0;1024];
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mut stream1 = stream;
                stream1.read(&mut buffer).unwrap();
                println!("Message: {} from {}", String::from_utf8_lossy(&buffer[..]), stream1.peer_addr().unwrap());
                break;                
            }
            Err(e) => {
                println!("Connection Error: {}", e);
            }
        }
    }

    match command_type{

        CommandServer::GetPk => {
            //TODO : receive the public key
            let pk_sender = std::str::from_utf8(&buffer).unwrap(); //why is std:: needed ? and why import isn't used

            //Write it in the public key file
            let f2 = fs::write(&file_pk, &pk_sender);
            if f2.is_err() {
                panic!("Cannot write the public key file");
            };
            println!("Wrote {} in the file : {} successfully", &pk_sender, &file_pk);
        }

        CommandServer::GetMessage => {
            //Receiver have access only to the public file
            let pk_sender = fs::read_to_string(&file_pk);
            if pk_sender.is_err() {
                panic!("Cannot find the public key file");
            }
            let sender_pub_kp = KeyPair::from_public_key(&pk_sender.unwrap()).unwrap();

            //TODO receive payload_string and sig from the client terminal

            //Check the signature
            //let res = sender_pub_kp.verify(&payload_string,&sig.as_slice());


        }    
    }




}

fn didclient(command_type: &CommandClient){

    // First we deal with the keys and files storing them

    //Paths
    let path = "C:\\Users\\emmanuel\\Documents\\Projet_IOT\\nkeys\\keys\\Alice\\";
    //let path = ""; 

    //Names of the 2 files that will store the keys
    let filename_seed = "alice_seed.txt";
    let filename_pk = "alice_pk.txt";

    let file_seed = format!("{}{}", path, filename_seed);
    let file_pk = format!("{}{}", path, filename_pk);

    //Try to read the file to get the seed 
    let mut seed_sender = fs::read_to_string(&file_seed);
    let mut pk_sender = fs::read_to_string(&file_pk);
    if seed_sender.is_err() { 
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

        //Now that's it's written, we need to update seed_sender and pk_sender
        seed_sender = fs::read_to_string(&file_seed);
        pk_sender = fs::read_to_string(&file_pk);
        if seed_sender.is_err() | pk_sender.is_err(){
            panic!("Cannot read the seed or pk file");
        }
    } else if pk_sender.is_err() {
        pk_sender = Ok(KeyPair::from_seed(&seed_sender.unwrap()).unwrap().public_key());
        let f2 = fs::write(&file_pk, &pk_sender.unwrap());
        if f2.is_err() {
            panic!("Cannot write the public key file from the seed");
        };
    }


    // Send the commanded packet (public key or message)
    let mut packet = Vec::new(); 
    match command_type {

        CommandClient::SendPk => {
            let mut f = File::open(&file_pk).unwrap();
            f.read_to_end(&mut packet);
            println!("You're sending your public key: {}", std::str::from_utf8(&packet).unwrap());
            //packet = pk_sender.unwrap(); 

        }

        CommandClient::SendMessage => {
            /*
            let payload = json!({
                "id": "urn:uuid:ef5a7369-f0b9-4143-a49d-2b9c7ee51117",
                "type": "didcomm",
                "from": "did:example:alice",
                "expiry": 1516239022,
                "time_stamp": 1516269022,
                "body": { "message": "Challenge!" }
                });
            
            let payload_string = serde_json::to_vec(&payload).unwrap();
            */
            println!("You're willing to send a signed message");
            //payload = b"message with kid etc..."
        }
    }

    match TcpStream::connect("localhost:8000"){
        Ok(mut stream) => {
            stream.write(&packet);
        },
        Err(e) => {
            panic!("Falied to connect: {}", e);
        }
    }
}