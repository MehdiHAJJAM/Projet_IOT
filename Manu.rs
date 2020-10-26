cd Documents/Projet_IOT/nkeys/src
cargo run --features="cli"
cd Documents/Projet_IOT/nkeys/target/debug
nk didcomm

======================================================================================
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