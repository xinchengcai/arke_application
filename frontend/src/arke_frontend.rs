use ark_bw6_761::BW6_761;
use ark_bls12_377::{Bls12_377, Parameters, Fq12Parameters};
use ark_ec::bls12::Bls12;
use ark_serialize::CanonicalSerialize;
use ark_std::One;
use rand::{thread_rng, CryptoRng, Rng};
use arke_core::{
    BlindIDCircuitParameters, BlindPartialSecretKey, IssuancePublicParameters,
    IssuerPublicKey, IssuerSecretKey, PartialSecretKey, RegistrarPublicKey, RegistrarSecretKey,
    ThresholdObliviousIdNIKE, UserID, UserSecretKey, 

    UnlinkableHandshake, SIZE_SYMMETRIC_KEYS_IN_BYTES, StoreKey,
};
use tiny_keccak::{Keccak, Hasher};
use bincode;
use ark_ff::{QuadExtField, Fp12ParamsWrapper};

type ArkeIdNIKE = ThresholdObliviousIdNIKE<Bls12_377, BW6_761>;

/// Total number of participants
const NUMBER_OF_PARTICIPANTS: usize = 10;
/// Maximum number of dishonest key-issuing authorities that the system can tolerate
const THRESHOLD: usize = 3;
/// Domain identifier for the registration authority of this example
const REGISTRAR_DOMAIN: &'static [u8] = b"registration";


pub struct Arke {
    pub _alice_id_string: String,
    pub _bob_id_string: String,
    pub _alice_sk: UserSecretKey<Bls12<Parameters>>,
    pub _bob_sk: UserSecretKey<Bls12<Parameters>>,
    pub _alice_computes_shared_seed: QuadExtField<Fp12ParamsWrapper<Fq12Parameters>>,
    pub _bob_computes_shared_seed: QuadExtField<Fp12ParamsWrapper<Fq12Parameters>>,
    pub _symmetric_key: Vec<u8>,
    pub _alice_write_tag: StoreKey,
    pub _bob_write_tag: StoreKey,
    pub _alice_read_tag: StoreKey,
    pub _bob_read_tag: StoreKey,
}

impl Arke {
    pub fn id_nike_and_handshake(alice_id_string: String, bob_id_string: String) -> Self {
        /* Arke ID-NIKE */ 
        let mut rng = thread_rng();
        // Generate a random user ID
        let alice_id = UserID::new(&alice_id_string);
        // Generate a random user ID
        let bob_id = UserID::new(&bob_id_string);
        let num_of_domain_sep_bytes = REGISTRAR_DOMAIN.len();
        let num_of_identifier_bytes = alice_id.0.as_bytes().len();
        let num_of_blinding_factor_bits = ark_bls12_377::Fr::one().serialized_size() * 8;
        // Simulate the SNARK trusted setup
        println!("Running trusted setup");
        let pp_zk = ArkeIdNIKE::setup_blind_id_proof(
            num_of_domain_sep_bytes,
            num_of_identifier_bytes,
            num_of_blinding_factor_bits,
            &mut rng,
        )
        .unwrap();

        // Simulate the DKG between issuers
        println!("Running DKG");
        let (pp_issuance, honest_issuers_secret_keys, honest_issuers_public_keys) =
            ArkeIdNIKE::simulate_issuers_DKG(THRESHOLD, NUMBER_OF_PARTICIPANTS, &mut rng).unwrap();

        // Create a registration authority
        println!("Setup registration authority");
        let (_pp_registration, registrar_secret_key, registrar_public_key) =
            ArkeIdNIKE::setup_registration(&mut rng);

        // Compute Alice and Bob's respective user secret keys
        println!("Your private keys:");
        let alice_sk = Self::get_user_secret_key(
            &pp_zk,
            &pp_issuance,
            &alice_id,
            THRESHOLD,
            &registrar_secret_key,
            &registrar_public_key,
            REGISTRAR_DOMAIN,
            &honest_issuers_secret_keys,
            &honest_issuers_public_keys,
            &mut rng,
        );

        println!("Your contact's private keys:");
        let bob_sk = Self::get_user_secret_key(
            &pp_zk,
            &pp_issuance,
            &bob_id,
            THRESHOLD,
            &registrar_secret_key,
            &registrar_public_key,
            REGISTRAR_DOMAIN,
            &honest_issuers_secret_keys,
            &honest_issuers_public_keys,
            &mut rng,
        );

        // Compute a shared seed
        let alice_computes_shared_seed =
            ArkeIdNIKE::shared_key(&alice_sk, &alice_id, &bob_id, REGISTRAR_DOMAIN).unwrap();
        let mut alice_seed_bytes = Vec::new();
        alice_computes_shared_seed
            .serialize(&mut alice_seed_bytes)
            .unwrap();
        println!("You computes shared seed: {:?}\n", alice_seed_bytes);

        let bob_computes_shared_seed =
            ArkeIdNIKE::shared_key(&bob_sk, &bob_id, &alice_id, REGISTRAR_DOMAIN).unwrap();
        let mut bob_seed_bytes = Vec::new();
        bob_computes_shared_seed
            .serialize(&mut bob_seed_bytes)
            .unwrap();
        println!("Your contact computes shared seed: {:?}\n", bob_seed_bytes);

        assert_eq!(alice_computes_shared_seed, bob_computes_shared_seed);
        println!("The seeds match!\n");


        /* Arke handshake */
        let rng = &mut thread_rng();
        let shared_seed = alice_computes_shared_seed;
    
        // Derive symmertric key from the shared seed
        let symmetric_key = UnlinkableHandshake::derive_symmetric_key(&shared_seed).unwrap();
        assert_eq!(SIZE_SYMMETRIC_KEYS_IN_BYTES, symmetric_key.len());
        println!("You and your contact derive a symmetric key: {:?}", symmetric_key);
    
        // Compute Write and Read tags
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
    
        // Verify Write
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

        Arke{
            _alice_id_string: alice_id_string,
            _bob_id_string: bob_id_string,
            _alice_sk: alice_sk,
            _bob_sk: bob_sk,
            _alice_computes_shared_seed: alice_computes_shared_seed,
            _bob_computes_shared_seed: bob_computes_shared_seed,
            _symmetric_key: symmetric_key,
            _alice_write_tag: alice_write_tag,
            _bob_write_tag: bob_write_tag,
            _alice_read_tag: alice_read_tag,
            _bob_read_tag: bob_read_tag,
        }
    }


    fn get_user_secret_key<R: Rng + CryptoRng>(
        pp_zk: &BlindIDCircuitParameters<BW6_761>,
        issuance_pp: &IssuancePublicParameters<Bls12_377>,
        user_id: &UserID,
        threshold: usize,
        registrar_secret_key: &RegistrarSecretKey<Bls12_377>,
        registrar_public_key: &RegistrarPublicKey<Bls12_377>,
        registrar_domain: &[u8],
        issuers_secret_keys: &[IssuerSecretKey<Bls12_377>],
        issuers_public_keys: &[IssuerPublicKey<Bls12_377>],
        rng: &mut R,
    ) -> UserSecretKey<Bls12_377> {
        println!("    Registration");
        // Register our user
        let reg_attestation =
            ArkeIdNIKE::register(&registrar_secret_key, &user_id, registrar_domain).unwrap();
    
        // Blind the identifier and token
        println!("    Blinding (and proof)");
        let (blinding_factor, blind_id, blind_reg_attestation) =
            ArkeIdNIKE::blind(pp_zk, user_id, registrar_domain, &reg_attestation, rng).unwrap();
    
        // Obtain blind partial secret keys from t+1 honest authorities
        println!("    BlindPartialExtract (verify reg and proof)");
        let blind_partial_user_keys: Vec<BlindPartialSecretKey<Bls12_377>> = issuers_secret_keys
            .iter()
            .zip(issuers_public_keys.iter())
            .map(|(secret_key, _public_key)| {
                ArkeIdNIKE::blind_partial_extract(
                    &issuance_pp,
                    pp_zk,
                    &registrar_public_key,
                    secret_key,
                    &blind_id,
                    &blind_reg_attestation,
                    registrar_domain,
                )
                .unwrap()
            })
            .collect();
    
        // Unblind each partial key
        println!("    Unblind");
        let partial_user_keys: Vec<PartialSecretKey<Bls12_377>> = blind_partial_user_keys
            .iter()
            .map(|blind_partial_sk| ArkeIdNIKE::unblind(blind_partial_sk, &blinding_factor))
            .collect();
    
        // Combine the partial keys to obtain a user secret key
        println!("    Combine");
        let user_secret_key = ArkeIdNIKE::combine(&partial_user_keys, threshold).unwrap();
    
        user_secret_key
    }


    pub fn to_address(a: &StoreKey) -> [u8; 20] {
        #![allow(non_snake_case)]
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