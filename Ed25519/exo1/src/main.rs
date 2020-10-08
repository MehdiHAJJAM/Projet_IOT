///
/// Adapation du didcomm_jwm JWM in Rust
///

/// use
extern crate serde_json;
use std::collections::HashMap;
use std::str;
use serde_json::{Value, Map, Number};

//TODO 
//How can we use serde_json in the nkeys but not in here ?

fn main(){

    // Create the payload
    let mut body = Map::new();
    body.insert("message".to_string(), Value::String("Challenge!").to_string());

    let mut payload = Map::new();
    payload.insert("id".to_string(), Value::String("urn:uuid:ef5a7369-f0b9-4143-a49d-2b9c7ee51117").to_string());
    payload.insert("type".to_string(), Value::String("didcomm").to_string());
    payload.insert("from".to_string(), Value::String("did:example:alice").to_string());
    payload.insert("expiry".to_string(), Value::Number(1516239022).to_string());
    payload.insert("time_stamp".to_string(), Value::Number(1516269022).to_string());
    payload.insert("body", Value::Object(body));


    println!("{}", serde_json::to_string(&payload).unwrap());


}

