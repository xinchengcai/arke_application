// ---------------------------------------
// File: discovery_info.rs
// Date: 11 Sept 2023
// Description: Contact discovery (client-side)
// ---------------------------------------
use ark_bw6_761::BW6_761;
use ark_bls12_377::{Bls12_377, Parameters, Fq12Parameters};
use ark_ec::bls12::Bls12;
use ark_serialize::CanonicalSerialize;
use rand::{thread_rng, Rng};
use arke_core::{
    ThresholdObliviousIdNIKE, UserID, UserSecretKey, 
    UnlinkableHandshake, SIZE_SYMMETRIC_KEYS_IN_BYTES, StoreKey,
};
use tiny_keccak::{Keccak, Hasher};
use bincode;
use ark_ff::{QuadExtField, Fp12ParamsWrapper};
type ArkeIdNIKE = ThresholdObliviousIdNIKE<Bls12_377, BW6_761>;
/// Domain identifier for the registration authority of this example
const REGISTRAR_DOMAIN: &'static [u8] = b"registration";

// Alice is used to denote the user for who discover
// Bob is used to denote the user being discovered 
pub struct DiscoveryInfo {
    pub alice_id_string: String,
    pub bob_id_string: String,
    pub alice_sk: UserSecretKey<Bls12<Parameters>>,
    pub alice_computes_shared_seed: QuadExtField<Fp12ParamsWrapper<Fq12Parameters>>,
    pub symmetric_key: Vec<u8>,
    pub alice_write_tag: StoreKey,
    pub alice_read_tag: StoreKey,
}

impl DiscoveryInfo {
    // Contact discovery
    pub fn id_nike_and_handshake(alice_id_string: String, bob_id_string: String, alice_sk: UserSecretKey<Bls12<Parameters>>) -> Self {
        let alice_id = UserID::new(&alice_id_string);
        let bob_id = UserID::new(&bob_id_string);

        // Run ID-NIKE.SharedKey
        let alice_computes_shared_seed =
            ArkeIdNIKE::shared_key(&alice_sk, &alice_id, &bob_id, REGISTRAR_DOMAIN).unwrap();
        let mut alice_seed_bytes = Vec::new();
        alice_computes_shared_seed
            .serialize(&mut alice_seed_bytes)
            .unwrap();
        println!("- You computes shared seed: {:?}", alice_seed_bytes);
        let shared_seed = alice_computes_shared_seed;

        // Run Handshake.DeriveWrite
        let symmetric_key = UnlinkableHandshake::derive_symmetric_key(&shared_seed).unwrap();
        assert_eq!(SIZE_SYMMETRIC_KEYS_IN_BYTES, symmetric_key.len());
        println!("- You derives a symmetric key: {:?}", symmetric_key);
        let (alice_write_tag, _alice_exponent) =
        UnlinkableHandshake::derive_write_tag(&shared_seed, &alice_id, &bob_id).unwrap();
        let (bob_write_tag, _bob_exponent) =
        UnlinkableHandshake::derive_write_tag(&shared_seed, &bob_id, &alice_id).unwrap();
        let alice_read_tag =
        UnlinkableHandshake::derive_read_tag(&shared_seed, &alice_id, &bob_id).unwrap();
        let bob_read_tag =
        UnlinkableHandshake::derive_read_tag(&shared_seed, &bob_id, &alice_id).unwrap();
        assert_eq!(alice_write_tag, bob_read_tag);
        assert_eq!(alice_read_tag, bob_write_tag);
    
        /* 
        Run DLOG.Prove and DLOG.Verify
        DLOG.Prove should be run by the user and
        DLOG.Verify should be run by the storage authority
        each time for making a Write transaction.
        */
        /* 
        We run DLOG.Prove and DLOG.Verify here in contact discovery process
        only once only for demonstration purposes. Saving exponent locally
        is complex as the type TagExponent does not implement Serialize and
        Deserialize traits.
        */
        let rng = &mut thread_rng();
        let mut session_id = [0u8; 4];
        rng.fill(&mut session_id);
        let proof = UnlinkableHandshake::prove_write_location(
            &alice_write_tag,
            &_alice_exponent,
            &session_id,
            rng,
        )
        .unwrap();
        UnlinkableHandshake::verify_write_location(&alice_write_tag, &proof, &session_id).unwrap();

        println!("✓ Finished contact discovery"); 
        DiscoveryInfo{
            alice_id_string: alice_id_string,
            bob_id_string: bob_id_string,
            alice_sk: alice_sk,
            alice_computes_shared_seed: alice_computes_shared_seed,
            symmetric_key: symmetric_key,
            alice_write_tag: alice_write_tag,
            alice_read_tag: alice_read_tag,
        }
    }


    pub fn to_address(a: &StoreKey) -> [u8; 20] {
        // Serialize struct
        let serialized_struct = bincode::serialize(&a).unwrap();
        let BLOCKCHAIN_CONSTANT = "constant";
        // Serialize constant
        let serialized_constant = bincode::serialize(&BLOCKCHAIN_CONSTANT).unwrap();
        // Concatenate
        let mut combined = serialized_struct.clone();
        combined.extend_from_slice(&serialized_constant);   
        // Hash
        let mut hasher = Keccak::v256();
        let mut output = [0u8; 32];
        hasher.update(&combined);
        hasher.finalize(&mut output);
        // Take the first 20 bytes
        let mut address: [u8; 20] = [0; 20];
        address.copy_from_slice(&output[0..20]);
        address
    }
}