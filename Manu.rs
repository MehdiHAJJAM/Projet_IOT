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
//Check 
//Check if file exists

//Generate the sender key and write in the file if not :
let kp = KeyPair::new(KeyPairType::User);
println!("{}",kp.seed().unwrap());
//
//let response = serde_json::to_string(&payload).unwrap();
//println!("Payload is : {}", payload);
}