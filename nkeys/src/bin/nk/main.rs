extern crate serde_json;

use nkeys::{self, KeyPair, KeyPairType};
use serde_json::json;
use serde_json::{Value};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::fs;
use std::fs::File;
use std::str::FromStr;
use structopt::clap::AppSettings;
use structopt::StructOpt;
use std::net::{TcpListener,TcpStream};
use std::io::{Read,Write};
//use std::str::from_utf8; // why isn't it necessary ?! oO

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
    let path = "C:\\Users\\emmanuel\\Documents\\Projet_IOT\\nkeys\\keys\\Alice\\";
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
    //let sender_pub_kp = KeyPair::from_public_key("UCVLXNOAAD72JVJBZ67OETRJKTPJ6FVAZXXMTKDBGHFYFAD32LJQE246").unwrap();

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
    //let path = "/Users/Mehdi/Desktop/Rust/Projet_IOT/nkeys/keys/Bob/";
    let filename_pk = "alice_pk.txt"; //File storing the public key
    let addr = "localhost:8000";

    let file_pk = format!("{}{}", path, filename_pk);

    // Receive the payload from the client
    let listener = TcpListener::bind(addr).unwrap();
    println!("Server Listening");

    //This is a try to deal with the buffer issue when registering the key.
    /*let &mut buffer = &[];
    match command_type{
        CommandServer::GetPk => {
            const N :usize = 56;
            buffer = [0;N];
        } 
        CommandServer::GetMessage => {
            const N :usize = 1024;
            buffer = [0;N];
        }
    }*/

    let mut buffer = Vec::new();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mut stream1 = stream;
                stream1.read_to_end(&mut buffer).unwrap();
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
            //let buffer1 = Vec::from_utf8(&buffer);
            let pk_sender = std::str::from_utf8(&buffer).unwrap(); //why is std:: needed ? and why import isn't used
            //let pk_sender = &buffer;

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

            //TODO deal with the NULL characters in the pk file.
            let sender_pub_kp = KeyPair::from_public_key(&pk_sender.unwrap()).unwrap();

            //let test = sender_pub_kp.unwrap();

            let message : Message = serde_json::from_str(std::str::from_utf8(&buffer).unwrap()).unwrap();
            println!("received a message with the kid {}", message.kid);
            let kid = &message.kid;
            let payload = &message.payload;
            let payload_vec = serde_json::to_vec(&payload).unwrap();
            let payload_display = std::str::from_utf8(&payload_vec).unwrap();
            let sig = message.signature;//.as_array().unwrap();
            //let sig_2 : Vec<u8> = sig.clone();
            //let sig_vec = serde_json::to_vec(&sig).unwrap();
    
            //for debug only
            //let kp_sender = KeyPair::from_seed("SUAIKCWLDWFLVXQBXGRL522LFG2S3R77WZ3WXOTNAXGCFHY6BAOEJTEOAM").unwrap();
            //let sig_test = kp_sender.sign(&payload_vec).unwrap();
            //let sig_display = std::str::from_utf8(&sig_vec).unwrap();


            //Check the signature
            //println!("received a verified message from alice signed with {} : \n {}", &kid, &sig.l;
            let res = sender_pub_kp.verify(&payload_vec,&sig);
            if res.is_ok(){
                println!("received a verified message from Alice signed with {} : \n {}", &kid, &payload_display);
            } else {
                println!("you received a message that has been corrupted (signature {} doesn't match) ", &kid);
            }


        }    
    }
}

fn didclient(command_type: &CommandClient){

    // First we deal with the keys and files storing them

    //PARAMS
    let path = "C:\\Users\\emmanuel\\Documents\\Projet_IOT\\nkeys\\keys\\Alice\\";
    //let path = "/Users/Mehdi/Desktop/Rust/Projet_IOT/nkeys/keys/Alice/";
    let filename_pk = "alice_pk.txt"; //File storing the public key
    let filename_seed = "alice_seed.txt"; 
    let addr = "localhost:8000";


    let file_seed = format!("{}{}", path, filename_seed);
    let file_pk = format!("{}{}", path, filename_pk);

    //Try to read the files
    let mut res_seed_sender = fs::read_to_string(&file_seed);
    let mut res_pk_sender = fs::read_to_string(&file_pk);
    let mut seed_sender = String::new(); //gonna be the unwrap() of res_seed_sender (deals with issues about unwrap() moving variables)
    let mut _foo = &seed_sender;
    let mut _pk_sender = String::new();

    if res_seed_sender.is_ok() {
        seed_sender = res_seed_sender.unwrap();
        if res_pk_sender.is_err() {
            //Just need to rewrite the public key as the seed still exists   
            res_pk_sender = Ok(KeyPair::from_seed(&seed_sender).unwrap().public_key());
            let f2 = fs::write(&file_pk, &res_pk_sender.unwrap());
            if f2.is_err() {
                panic!("Cannot write the public key file from the seed");
            };
        }
    } else {
        //If seed file doesn't exist, we create it and update/create the public one
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
        res_seed_sender = fs::read_to_string(&file_seed);
        res_pk_sender = fs::read_to_string(&file_pk);
        if res_seed_sender.is_err() | res_pk_sender.is_err(){
            panic!("Cannot read the seed or pk file");
        }
        seed_sender = res_seed_sender.unwrap();
    }

    let seed_sender_str: &str = &seed_sender;
    let kp_sender = KeyPair::from_seed(seed_sender_str).unwrap(); 
    // Send the commanded packet (public key or message)
    let mut packet = Vec::new(); 
    match command_type {

        CommandClient::SendPk => {
            //Read the public key from the file and here is the packet !
            let mut f = File::open(&file_pk).unwrap();
            let f0 = f.read_to_end(&mut packet);
            if f0.is_err() {
                panic!("Cannot read the pk file to send it");
            }
            println!("You're sending your public key: {}", std::str::from_utf8(&packet).unwrap());
        }

        CommandClient::SendMessage => {
            //Payload that the sender will sign (and encrypt ?) before sending to the server
            let payload = json!({
                "id": "urn:uuid:ef5a7369-f0b9-4143-a49d-2b9c7ee51117",
                "type": "didcomm",
                "from": "did:example:alice",
                "expiry": 1516239022,
                "time_stamp": 1516269022,
                "body": { "message": "Challenge!" }
            });
            
            //Sender signs using a keypair from seed
            let payload_vec = serde_json::to_vec(&payload).unwrap();
       
            let sig = kp_sender.sign(&payload_vec).unwrap();

            //Format the message
            let message = json!({
                "kid": "ed25519",
                "payload" : payload,
                "signature": sig, 
            });
            packet = serde_json::to_vec(&message).unwrap();

            println!("You're sending a signed message with signature with length : {}", &sig.len());
        }
    }

    //Connect and send the packet (pk or message) through a TcpStream
    match TcpStream::connect(addr){
        Ok(mut stream) => {
            let f0 = stream.write(&packet);
            if f0.is_err() {
                panic!("Cannot write the packet");
            }
        },
        Err(e) => {
            panic!("Failed to connect: {}", e);
        }
    }
}