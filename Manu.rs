cd Documents/Projet_IOT/nkeys/src
cargo run --features="cli"
cd Documents/Projet_IOT/nkeys/target/debug
nk didcomm

===================================================================================
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
======================================================================================
fn didcomm() {
    let payload = json!({
    "id": "urn:uuid:ef5a7369-f0b9-4143-a49d-2b9c7ee51117",
    "type": "didcomm",
    "from": "did:example:alice",
    "expiry": 1516239022,
    "time_stamp": 1516269022,
    "body": { "message": "Challenge!" }
    });

    let payload_string = serde_json::to_vec(&payload).unwrap();
    //Check 
    //Check if file exists

    //Generate the sender key and write in the 2 files (priv et publ) if not :
    let kp = KeyPair::new(KeyPairType::User);
    println!("{}",kp.seed().unwrap());
    println!("{}",kp.public_key());

    //Read the seed from the file : TODO: move the file in a folder far from here
    /*let sender_seed = fs::read_to_string("alice.txt")
        .expect("Something went wrong reading the file");
    let sender_seed_str: &str = &sender_seed;
    println!("{}", sender_seed);*/
    let sender_seed_str = "SUAEQAVUZ7A7CBKQGTOCUGSB4T35Z6VTF3OXTPU2MUWUJZTMOLWVXHQXUY";

    //Sender signs using the keypair from seed
    let sender_kp = KeyPair::from_seed(&sender_seed_str).unwrap();
    let sig = sender_kp.sign(&payload_string).unwrap();

    //Receiver have access only to the public file
    let sender_pub_kp = KeyPair::from_public_key("UDTAPJ42PAKIVF2ZFSKRNT7WBQVFB62NHUY5CRYE6ZBR64KOXFXQHPBQ").unwrap();
    let res = sender_kp.verify(&payload_string,&sig.as_slice()).unwrap();
    //assert(res.is_ok());
    
    //let response = serde_json::to_string(&payload).unwrap();
    //println!("Payload is : {}", payload);
    
}