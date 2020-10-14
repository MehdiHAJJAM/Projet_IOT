
use std::fs;


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
    let path = "/Users/Mehdi/Desktop/Rust/Projet_IOT/nkeys/src/bin/nk/";

    let contents = fs::read_to_string(path.to_owned() + "alice.txt")
        .expect("Something went wrong reading the file");
    println!("{}", contents)
}