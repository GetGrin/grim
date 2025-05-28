// Copyright 2025 The Grim Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use core::num::NonZeroU32;
use grin_util::{ToHex, ZeroingString};
use grin_wallet_impls::Error;
use rand::{rng, Rng};
use serde_derive::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::Write;

use ring::aead;
use ring::pbkdf2;

#[derive(Clone, Debug, PartialEq)]
pub struct WalletSeed(Vec<u8>);

impl WalletSeed {
    pub fn from_bytes(bytes: &[u8]) -> WalletSeed {
        WalletSeed(bytes.to_vec())
    }

    pub fn from_mnemonic(word_list: ZeroingString) -> Result<WalletSeed, Error> {
        let res = grin_keychain::mnemonic::to_entropy(&word_list);
        match res {
            Ok(s) => Ok(WalletSeed::from_bytes(&s)),
            Err(_) => Err(Error::Mnemonic.into()),
        }
    }

    pub fn init_file(
        seed_file_path: &str,
        recovery_phrase: ZeroingString,
        password: ZeroingString,
    ) -> Result<WalletSeed, Error> {
        let seed = WalletSeed::from_mnemonic(recovery_phrase)?;
        let enc_seed = EncryptedWalletSeed::from_seed(&seed, password)?;
        let enc_seed_json = serde_json::to_string_pretty(&enc_seed).map_err(|_| Error::Format)?;
        let mut file = File::create(seed_file_path).map_err(|_| Error::IO)?;
        file.write_all(&enc_seed_json.as_bytes())
            .map_err(|_| Error::IO)?;
        Ok(seed)
    }
}

/// Encrypted wallet seed, for storing on disk and decrypting with provided password.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EncryptedWalletSeed {
    encrypted_seed: String,
    pub salt: String,
    pub nonce: String,
}

impl EncryptedWalletSeed {
    /// Create a new encrypted seed from the given seed + password.
    pub fn from_seed(
        seed: &WalletSeed,
        password: ZeroingString,
    ) -> Result<EncryptedWalletSeed, Error> {
        let salt: [u8; 8] = rng().random();
        let nonce: [u8; 12] = rng().random();
        let password = password.as_bytes();
        let mut key = [0; 32];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA512,
            NonZeroU32::new(100).unwrap(),
            &salt,
            password,
            &mut key,
        );
        let content = seed.0.to_vec();
        let mut enc_bytes = content;
        let unbound_key = aead::UnboundKey::new(&aead::CHACHA20_POLY1305, &key).unwrap();
        let sealing_key: aead::LessSafeKey = aead::LessSafeKey::new(unbound_key);
        let aad = aead::Aad::from(&[]);
        let res = sealing_key.seal_in_place_append_tag(
            aead::Nonce::assume_unique_for_key(nonce),
            aad,
            &mut enc_bytes,
        );
        if let Err(_) = res {
            return Err(Error::Encryption);
        }
        Ok(EncryptedWalletSeed {
            encrypted_seed: enc_bytes.to_hex(),
            salt: salt.to_hex(),
            nonce: nonce.to_hex(),
        })
    }
}