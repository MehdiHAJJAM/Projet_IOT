cd Documents/Projet_IOT/nkeys/src
cargo run --features="cli"
cd Documents/Projet_IOT/nkeys/target/debug
nk didcomm

======================================================================================
fn didcomm() {
    
    //Paths - Comment the ones you don't use 
    let path_seed = "C:\\Users\\emmanuel\\Documents\\Projet_IOT\\nkeys\\alice_seed.txt";
    let path_pk = "C:\\Users\\emmanuel\\Documents\\Projet_IOT\\nkeys\\alice_pk.txt";
    //let path = ""; 

    /* TODO find a way to concat path + filenames to clean it up
    //Names of the 2 files that will store the keys
    let filename_seed = "alice_seed.txt";
    let filename_pk = "alice_pk.txt";
    */

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
    let mut sender_seed = fs::read_to_string(path_seed);
    if sender_seed.is_err() { 
        //If it doesn't exist, we create it and update/create the public one
        let kp = KeyPair::new(KeyPairType::User);
        fs::write(path_seed, &kp.seed().unwrap());
        fs::write(path_pk, &kp.public_key());
        sender_seed = fs::read_to_string(path_seed);
        if sender_seed.is_err(){
            //TODO deal with the argument to raise an Error
            println!("Cannot find the seed file. Exiting...");
            return ();
        }
    }

    let sender_seed_str: &str = &sender_seed.unwrap();

    //Sender signs using the keypair from seed
    let sender_kp = KeyPair::from_seed(&sender_seed_str).unwrap();        
    let sig = sender_kp.sign(&payload_string).unwrap();

    //Receiver have access only to the public file
    let sender_pk = fs::read_to_string(path_pk);
    if sender_pk.is_err() {
        //Err(err!(FileNotFound,"The public key cannot be found"));
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