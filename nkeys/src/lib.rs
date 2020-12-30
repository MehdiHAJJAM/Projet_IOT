//! # nkeys
//!
//! The `nkeys` is a Rust port of the official NATS [Go](https://github.com/nats-io/nkeys) nkeys implementation.
//! While it is a port, every effort has been made to make this library feel like idiomatic Rust and to do things
//! in a "rusty" way.
//!
//! Nkeys provides library functions to create ed25519 keys using the special prefix encoding system used by
//! NATS 2.0+ security
//!
//! # Examples
//! ```
//! use nkeys::KeyPair;
//!
//! // Create a user key pair
//! let user = KeyPair::new_user();
//!
//! // Sign some data with the user's full key pair
//! let msg = "this is super secret".as_bytes();
//! let sig = user.sign(&msg).unwrap();
//! let res = user.verify(msg, sig.as_slice());
//! assert!(res.is_ok());
//!
//! // Access the encoded seed (the information that needs to be kept safe/secret)
//! let seed = user.seed().unwrap();
//! // Access the public key, which can be safely shared
//! let pk = user.public_key();
//!
//! // Create a full User who can sign and verify from a private seed.
//! let user = KeyPair::from_seed(&seed);
//!
//! // Create a user that can only verify and not sign
//! let user = KeyPair::from_public_key(&pk).unwrap();
//! assert!(user.seed().is_err());
//! ```

#![allow(dead_code)]

use crc::{extract_crc, push_crc, valid_checksum};
use rand::prelude::*;
use signatory::ed25519;
use signatory::ed25519::PublicKey;
use signatory::public_key::PublicKeyed;
use signatory::signature::{Signer, Verifier};
use signatory_dalek::{Ed25519Signer, Ed25519Verifier};
use std::fmt;
use std::fmt::Debug;
use std::fs;
use std::io::{Read,Write};
use std::net::{TcpListener,TcpStream};

const ENCODED_SEED_LENGTH: usize = 58;

const PREFIX_BYTE_SEED: u8 = 18 << 3;
const PREFIX_BYTE_PRIVATE: u8 = 15 << 3;
const PREFIX_BYTE_SERVER: u8 = 18 << 3;
/*const PREFIX_BYTE_CLUSTER: u8 = 2 << 3;
const PREFIX_BYTE_OPERATOR: u8 = 14 << 3;
const PREFIX_BYTE_MODULE: u8 = 12 << 3;
const PREFIX_BYTE_ACCOUNT: u8 = 0;*/
const PREFIX_BYTE_USER: u8 = 20 << 3;
const PREFIX_BYTE_UNKNOWN: u8 = 23 << 3;
const PREFIX_BYTE_DEVICE: u8 = 3 << 3;

const PUBLIC_KEY_PREFIXES: [u8; 3] = [
    //PREFIX_BYTE_ACCOUNT,
    //PREFIX_BYTE_CLUSTER,
    //PREFIX_BYTE_OPERATOR,
    PREFIX_BYTE_SERVER,
    PREFIX_BYTE_USER,
    //PREFIX_BYTE_MODULE,
    PREFIX_BYTE_DEVICE,
];

type Result<T> = std::result::Result<T, crate::error::Error>;

/// The main interface used for reading and writing _nkey-encoded_ key pairs, including
/// seeds and public keys.
#[derive(Clone)]
pub struct KeyPair {
    kp_type: KeyPairType,
    rawkey_kind: RawKeyKind,
    pk: ed25519::PublicKey,
}

impl Debug for KeyPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "KeyPair {{ kp_type: {:?}, rawkey_kind: {:?}, pk: (hidden) }}",
            self.kp_type, self.rawkey_kind
        )
    }
}

#[derive(Clone)]
enum RawKeyKind {
    Seed(ed25519::Seed),
    Public,
}

impl Debug for RawKeyKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RawKeyKind::Public => write!(f, "Public"),
            RawKeyKind::Seed(_) => write!(f, "Seed"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum KeyPairType {
    Server,
    //Cluster,
    //Operator,
    //Account,
    User,
    //Module,
    Device,
}

impl std::str::FromStr for KeyPairType {
    type Err = crate::error::Error;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        let tgt = s.to_uppercase();

        match tgt.as_ref() {
            "SERVER" => Ok(KeyPairType::Server),
            //"CLUSTER" => Ok(KeyPairType::Cluster),
            //"OPERATOR" => Ok(KeyPairType::Operator),
            //"ACCOUNT" => Ok(KeyPairType::Account),
            "USER" => Ok(KeyPairType::User),
            //"MODULE" => Ok(KeyPairType::Module),
            "DEVICE" => Ok(KeyPairType::Device),
            _ => Err(crate::error::Error::new(
                crate::error::ErrorKind::IncorrectKeyType,
                Some("Unknown keypair type"),
            )),
        }
    }
}

impl From<u8> for KeyPairType {
    fn from(prefix_byte: u8) -> KeyPairType {
        match prefix_byte {
            PREFIX_BYTE_SERVER => KeyPairType::Server,
            //PREFIX_BYTE_CLUSTER => KeyPairType::Cluster,
            //PREFIX_BYTE_OPERATOR => KeyPairType::Operator,
            //PREFIX_BYTE_ACCOUNT => KeyPairType::Account,
            PREFIX_BYTE_USER => KeyPairType::User,
            //PREFIX_BYTE_MODULE => KeyPairType::Module,
            PREFIX_BYTE_DEVICE => KeyPairType::Device,
            _ => KeyPairType::User,
        }
    }
}

impl KeyPair {
    pub fn new(kp_type: KeyPairType) -> KeyPair {
        let s = create_seed();
        KeyPair {
            kp_type,
            pk: pk_from_seed(&s),
            rawkey_kind: RawKeyKind::Seed(s),
        }
    }

    /// Creates a new user key pair with a seed that has a **U** prefix
    pub fn new_user() -> KeyPair {
        Self::new(KeyPairType::User)
    }

    /// Creates a new account key pair with a seed that has an **A** prefix
    /*pub fn new_account() -> KeyPair {
        Self::new(KeyPairType::Account)
    }*/

    /// Creates a new operator key pair with a seed that has an **O** prefix
    /*pub fn new_operator() -> KeyPair {
        Self::new(KeyPairType::Operator)
    }*/

    /// Creates a new cluster key pair with a seed that has the **C** prefix
    /*pub fn new_cluster() -> KeyPair {
        Self::new(KeyPairType::Cluster)
    }*/

    /// Creates a new server key pair with a seed that has the **S** prefix
    pub fn new_server() -> KeyPair {
        Self::new(KeyPairType::Server)
    }

    /// Creates a new server key pair with a seed that has the **D** prefix
    pub fn new_device() -> KeyPair {
        Self::new(KeyPairType::Device)
    }

    /// Creates a new module (e.g. WebAssembly) key pair with a seed that has the **M** prefix
    /*pub fn new_module() -> KeyPair {
        Self::new(KeyPairType::Module)
    }*/

    /// Returns the encoded public key of this key pair
    pub fn public_key(&self) -> String {
        let mut raw = vec![];

        raw.push(get_prefix_byte(&self.kp_type));

        raw.extend(self.pk.as_bytes());

        push_crc(&mut raw);
        data_encoding::BASE32_NOPAD.encode(&raw[..])
    }

    /// Attempts to sign the given input with the key pair's seed
    pub fn sign(&self, input: &[u8]) -> Result<Vec<u8>> {
        if let RawKeyKind::Seed(ref seed) = self.rawkey_kind {
            let signer = Ed25519Signer::from(seed);
            //let sig = signatory::ed25519::sign(&signer, input)?;
            let sig = signer.sign(input);
            Ok(sig.to_bytes().to_vec())
        } else {
            Err(err!(SignatureError, "Cannot sign without a seed key"))
        }
    }

    /// Attempts to verify that the given signature is valid for the given input
    pub fn verify(&self, input: &[u8], sig: &[u8]) -> Result<()> {
        let verifier = Ed25519Verifier::from(&self.pk);
        let mut fixedsig = [0; ed25519::SIGNATURE_SIZE];
        fixedsig.copy_from_slice(sig);
        let insig = ed25519::Signature::new(fixedsig);

        match verifier.verify(input, &insig) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    /// Attempts to return the encoded string for this key pair's seed. Remember that this
    /// value should be treated as a secret
    pub fn seed(&self) -> Result<String> {
        if let RawKeyKind::Seed(ref seed) = self.rawkey_kind {
            let mut raw = vec![];
            let prefix_byte = get_prefix_byte(&self.kp_type);

            let b1 = PREFIX_BYTE_SEED | prefix_byte >> 5;
            let b2 = (prefix_byte & 31) << 3;

            raw.push(b1);
            raw.push(b2);
            raw.extend(seed.as_secret_slice().iter());
            push_crc(&mut raw);

            Ok(data_encoding::BASE32_NOPAD.encode(&raw[..]))
        } else {
            Err(err!(IncorrectKeyType, "This keypair has no seed"))
        }
    }

    /// Attempts to produce a public-only key pair from the given encoded string
    pub fn from_public_key(source: &str) -> Result<KeyPair> {
        let source_bytes = source.as_bytes();
        let mut raw = decode_raw(source_bytes)?;

        let prefix = raw[0];
        if !valid_public_key_prefix(prefix) {
            Err(err!(
                InvalidPrefix,
                "Not a valid public key prefix: {}",
                raw[0]
            ))
        } else {
            raw.remove(0);
            match PublicKey::from_bytes(raw) {
                Some(pk) => Ok(KeyPair {
                    kp_type: KeyPairType::from(prefix),
                    pk,
                    rawkey_kind: RawKeyKind::Public,
                }),
                None => Err(err!(VerifyError, "Could not read public key")),
            }
        }
    }

    /// Attempts to produce a full key pair from the given encoded seed string
    pub fn from_seed(source: &str) -> Result<KeyPair> {
        if source.len() != ENCODED_SEED_LENGTH {
            let l = source.len();
            return Err(err!(InvalidSeedLength, "Bad seed length: {}", l));
        }

        let source_bytes = source.as_bytes();
        let raw = decode_raw(source_bytes)?;

        let b1 = raw[0] & 248;
        if b1 != PREFIX_BYTE_SEED {
            Err(err!(
                InvalidPrefix,
                "Incorrect byte prefix: {}",
                source.chars().nth(0).unwrap()
            ))
        } else {
            let b2 = (raw[0] & 7) << 5 | ((raw[1] & 248) >> 3);

            let kp_type = KeyPairType::from(b2);
            let mut seed_bytes = [0u8; 32];
            seed_bytes.copy_from_slice(&raw[2..]);
            let seed = ed25519::Seed::new(seed_bytes);

            Ok(KeyPair {
                kp_type,
                pk: pk_from_seed(&seed),
                rawkey_kind: RawKeyKind::Seed(seed),
            })
        }
    }
}

fn pk_from_seed(seed: &ed25519::Seed) -> PublicKey {
    let signer = Ed25519Signer::from(seed);
    signer.public_key().unwrap()
}

fn decode_raw(raw: &[u8]) -> Result<Vec<u8>> {
    let mut b32_decoded = data_encoding::BASE32_NOPAD.decode(raw)?;

    let checksum = extract_crc(&mut b32_decoded);
    let v_checksum = valid_checksum(&b32_decoded, checksum);
    if !v_checksum {
        Err(err!(ChecksumFailure, "Checksum mismatch"))
    } else {
        Ok(b32_decoded)
    }
}

fn create_seed() -> ed25519::Seed {
    let mut rng = rand::thread_rng();
    let rnd_bytes = rng.gen::<[u8; 32]>();

    ed25519::Seed::new(rnd_bytes)
}

fn get_prefix_byte(kp_type: &KeyPairType) -> u8 {
    match kp_type {
        KeyPairType::Server => PREFIX_BYTE_SERVER,
        //KeyPairType::Account => PREFIX_BYTE_ACCOUNT,
        //KeyPairType::Cluster => PREFIX_BYTE_CLUSTER,
        //KeyPairType::Operator => PREFIX_BYTE_OPERATOR,
        KeyPairType::User => PREFIX_BYTE_USER,
        //KeyPairType::Module => PREFIX_BYTE_MODULE,
        KeyPairType::Device => PREFIX_BYTE_DEVICE,
    }
}

fn valid_public_key_prefix(prefix: u8) -> bool {
    PUBLIC_KEY_PREFIXES.to_vec().contains(&prefix)
}

///Connects to a TCPStream and return the buffer read
pub fn receive_stream(addr: &str) -> Result<Vec<u8>> {
    let listener = TcpListener::bind(addr)?;
    println!("Server Listening...");
    let mut buffer = Vec::new();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mut stream1 = stream;
                println!("Reading incoming stream from {}", stream1.peer_addr().unwrap());
                stream1.read_to_end(&mut buffer)?;
                break;               
            },
            Err(e) => {
                return Err(err!(ConnectionError, "Cannot read incoming stream: {}", e));
            },
        }
    }
    Ok(buffer)
}

///Connects to a TCPStream and sends a packet
pub fn send_stream(addr: &str, packet: &Vec<u8>) -> Result<()> {
    match TcpStream::connect(addr){
        Ok(mut stream) => {
            stream.write(&packet)?;
            Ok(())
        },
        Err(e) => {
            Err(err!(ConnectionError, "Cannot write outgoing stream: {}", e))
        }
    }
}

///Verifies that the seed file exists, if not, generates a new keypair and stores the seed.
pub fn verify_seed(file_seed: &str) -> Result<()> {

    let res_seed = fs::read_to_string(&file_seed);
    match res_seed {
        Ok(_) => Ok(()),
        Err(e) => { 
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    let kp = KeyPair::new(KeyPairType::User);
                    fs::write(&file_seed, &kp.seed().unwrap()).expect("Cannot write the seed file");
                    Ok(())
                },
                _ => Err(err!(IOError, "I/O failure: {}", e)),
            }
        },       
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorKind;

    #[test]
    fn seed_encode_decode_round_trip() {
        let pair = KeyPair::new_user();
        let s = pair.seed().unwrap();
        let p = pair.public_key();

        let pair2 = KeyPair::from_seed(s.as_str()).unwrap();
        let s2 = pair2.seed().unwrap();

        assert_eq!(s, s2);
        assert_eq!(p, pair2.public_key());
    }

    #[test]
    fn roundtrip_encoding_go_compat() {
        // Seed and Public Key pair generated by Go nkeys library
        let seed = "SUAMLDHY5IDE6E2UNGZYHSV5ANAY6VKOA45OAIM3SQJVXOVCBEICETAGXE";
        let pk = "UDWCI7RVPLNUHT2OUO32YSKN3YZ33KKXNNCQ6HNU3IQYVGSPVXFR57A5";

        let pair = KeyPair::from_seed(seed).unwrap();

        assert_eq!(pair.seed().unwrap(), seed);
        assert_eq!(pair.public_key(), pk);
    }

    #[test]
    fn from_seed_rejects_bad_prefix() {
        let seed = "FAAPN4W3EG6KCJGUQTKTJ5GSB5NHK5CHAJL4DBGFUM3HHROI4XUEP4OBK4";
        let pair = KeyPair::from_seed(seed);
        assert!(pair.is_err());
        if let Err(e) = pair {
            assert_eq!(e.kind(), ErrorKind::InvalidPrefix);
        }
    }

    /*
    * TODO - uncomment this test when I can figure out how to encode a bad checksum
     * without first triggering a base32 decoding failure :)

        #[test]
        fn from_seed_rejects_bad_checksum() {
            let seed = "SAAPN4W3EG6KCJGUQTKTJ5GSB5NHK5CHAJL4DBGFUM3HHROI4XUEP4OBK4";
            let pair = KeyPair::from_seed(seed);
            assert!(pair.is_err());
            if let Err(e) = pair {
                assert_eq!(e.kind(), ErrorKind::ChecksumFailure);
            }
        }
    */

    #[test]
    fn from_seed_rejects_bad_length() {
        let seed = "SAAPN4W3EG6KCJGUQTKTJ5GSB5NHK5CHAJL4DBGFUM3SAAPN4W3EG6KCJGUQTKTJ5GSB5NHK5";
        let pair = KeyPair::from_seed(seed);
        assert!(pair.is_err());
        if let Err(e) = pair {
            assert_eq!(e.kind(), ErrorKind::InvalidSeedLength);
        }
    }

    #[test]
    fn from_seed_rejects_invalid_encoding() {
        let badseed = "SAAPN4W3EG6KCJGUQTKTJ5!#B5NHK5CHAJL4DBGFUM3HHROI4XUEP4OBK4";
        let pair = KeyPair::from_seed(badseed);
        assert!(pair.is_err());
        if let Err(e) = pair {
            assert_eq!(e.kind(), ErrorKind::CodecFailure);
        }
    }

    #[test]
    fn sign_and_verify() {
        let user = KeyPair::new_user();
        let msg = b"this is super secret";

        let sig = user.sign(msg).unwrap();

        let res = user.verify(msg, sig.as_slice());
        assert!(res.is_ok());
    }

    #[test]
    fn sign_and_verify_rejects_mismatched_sig() {
        let user = KeyPair::new_user();
        let msg = b"this is super secret";

        let sig = user.sign(msg).unwrap();
        let res = user.verify(b"this doesn't match the message", sig.as_slice());
        assert!(res.is_err());
    }

    #[test]
    fn public_key_round_trip() {
        let account =
            KeyPair::from_public_key("UDWCI7RVPLNUHT2OUO32YSKN3YZ33KKXNNCQ6HNU3IQYVGSPVXFR57A5")
                .unwrap();
        let pk = account.public_key();
        assert_eq!(
            pk,
            "UDWCI7RVPLNUHT2OUO32YSKN3YZ33KKXNNCQ6HNU3IQYVGSPVXFR57A5"
        );
    }

    #[test]
    fn user_has_proper_prefix() {
        let module = KeyPair::new_user();
        assert!(module.seed().unwrap().starts_with("SU"));
        assert!(module.public_key().starts_with('U'));
    }

    /* 
    TODO Update and use once Async TCP is used
    #[test]
    fn send_receive_stream() {
        let addr = "localhost:8000";
        let packet = "UCBX745OPN7ZXJ7V72BG6T3IEP5SNELS2YGT3PGLTYOOZUUN6A7CG4YS".to_bytes().to_vec();
        receive_stream(&addr);
        send_stream(&addr, &packet);
        receive_stream(&addr);
    } */

    #[test]
    fn verify_seed_round_trip() {
        let path = "test_seed.txt";
        fs::remove_file(path);
        verify_seed(path);
        let seed = fs::read_to_string(path);
        assert!(seed.is_ok());
        verify_seed(path);
        assert_eq!(seed.unwrap(), fs::read_to_string(path).unwrap());
        let f = fs::remove_file(path);
        assert!(f.is_ok());
    }
}

mod crc;
pub mod error;
