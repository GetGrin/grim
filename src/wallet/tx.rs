// Copyright 2023 The Grim Developers
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

use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use ed25519_dalek::{Signer, Verifier};
use ed25519_dalek::Keypair as DalekKeypair;
use ed25519_dalek::PublicKey as DalekPublicKey;
use ed25519_dalek::SecretKey as DalekSecretKey;
use ed25519_dalek::Signature as DalekSignature;
use grin_core::consensus::valid_header_version;
use grin_core::core::FeeFields;
use grin_core::core::HeaderVersion;
use grin_keychain::{Identifier, Keychain};
use grin_util::Mutex;
use grin_util::secp::key::SecretKey;
use grin_util::secp::pedersen;
use grin_wallet_libwallet::{Context, NodeClient, StoredProofInfo, TxLogEntryType, WalletBackend};
use grin_wallet_libwallet::{address, Error};
use grin_wallet_libwallet::InitTxArgs;
use grin_wallet_libwallet::Slate;
use grin_wallet_util::OnionV3Address;
use lazy_static::lazy_static;
use log::trace;
use uuid::Uuid;

use crate::wallet::selection::{build_recipient_output, build_send_tx, select_coins_and_fee};
use crate::wallet::updater::{cancel_tx_and_outputs, refresh_outputs, retrieve_outputs, retrieve_txs};

/// Static value to increment UUIDs of slates.
lazy_static! {
	static ref SLATE_COUNTER: Mutex<u8> = Mutex::new(0);
}

/// Creates a new slate for a transaction, can be called by anyone involved in
/// the transaction (sender(s), receiver(s)).
pub fn new_tx_slate<'a, T: ?Sized, C, K>(
    wallet: &mut T,
    amount: u64,
    is_invoice: bool,
    num_participants: u8,
    use_test_rng: bool,
    ttl_blocks: Option<u64>,
) -> Result<Slate, Error>
    where
        T: WalletBackend<'a, C, K>,
        C: NodeClient + 'a,
        K: Keychain + 'a,
{
    let current_height = wallet.w2n_client().get_chain_tip()?.0;
    let mut slate = Slate::blank(num_participants, is_invoice);
    if let Some(b) = ttl_blocks {
        slate.ttl_cutoff_height = current_height + b;
    }
    if use_test_rng {
        {
            let sc = SLATE_COUNTER.lock();
            let bytes = [4, 54, 67, 12, 43, 2, 98, 76, 32, 50, 87, 5, 1, 33, 43, *sc];
            slate.id = Uuid::from_slice(&bytes).unwrap();
        }
        *SLATE_COUNTER.lock() += 1;
    }
    slate.amount = amount;

    if valid_header_version(current_height, HeaderVersion(1)) {
        slate.version_info.block_header_version = 1;
    }

    if valid_header_version(current_height, HeaderVersion(2)) {
        slate.version_info.block_header_version = 2;
    }

    if valid_header_version(current_height, HeaderVersion(3)) {
        slate.version_info.block_header_version = 3;
    }

    // Set the features explicitly to 0 here.
    // This will generate a Plain kernel (rather than a HeightLocked kernel).
    slate.kernel_features = 0;

    Ok(slate)
}

/// Add inputs to the slate (effectively becoming the sender).
pub fn add_inputs_to_slate<'a, T: ?Sized, C, K>(
    wallet: &mut T,
    keychain_mask: Option<&SecretKey>,
    slate: &mut Slate,
    current_height: u64,
    minimum_confirmations: u64,
    max_outputs: usize,
    num_change_outputs: usize,
    selection_strategy_is_use_all: bool,
    parent_key_id: &Identifier,
    is_initiator: bool,
    use_test_rng: bool,
    amount_includes_fee: bool,
) -> Result<Context, Error>
    where
        T: WalletBackend<'a, C, K>,
        C: NodeClient + 'a,
        K: Keychain + 'a,
{
    // sender should always refresh outputs
    refresh_outputs(wallet, keychain_mask, parent_key_id, false)?;

    // Sender selects outputs into a new slate and save our corresponding keys in
    // a transaction context. The secret key in our transaction context will be
    // randomly selected. This returns the public slate, and a closure that locks
    // our inputs and outputs once we're convinced the transaction exchange went
    // according to plan
    // This function is just a big helper to do all of that, in theory
    // this process can be split up in any way
    let mut context = build_send_tx(
        wallet,
        &wallet.keychain(keychain_mask)?,
        keychain_mask,
        slate,
        current_height,
        minimum_confirmations,
        max_outputs,
        num_change_outputs,
        selection_strategy_is_use_all,
        None,
        parent_key_id.clone(),
        use_test_rng,
        is_initiator,
        amount_includes_fee,
    )?;

    // Generate a kernel offset and subtract from our context's secret key. Store
    // the offset in the slate's transaction kernel, and adds our public key
    // information to the slate
    slate.fill_round_1(&wallet.keychain(keychain_mask)?, &mut context)?;

    context.initial_sec_key = context.sec_key.clone();

    if !is_initiator {
        // perform partial sig
        slate.fill_round_2(
            &wallet.keychain(keychain_mask)?,
            &context.sec_key,
            &context.sec_nonce,
        )?;
    }

    Ok(context)
}

/// Add receiver output to the slate.
pub fn add_output_to_slate<'a, T: ?Sized, C, K>(
    wallet: &mut T,
    keychain_mask: Option<&SecretKey>,
    slate: &mut Slate,
    current_height: u64,
    parent_key_id: &Identifier,
    is_initiator: bool,
    use_test_rng: bool,
) -> Result<Context, Error>
    where
        T: WalletBackend<'a, C, K>,
        C: NodeClient + 'a,
        K: Keychain + 'a,
{
    let keychain = wallet.keychain(keychain_mask)?;
    // create an output using the amount in the slate
    let (_, mut context, mut tx) = build_recipient_output(
        wallet,
        keychain_mask,
        slate,
        current_height,
        parent_key_id.clone(),
        use_test_rng,
        is_initiator,
    )?;

    // fill public keys
    slate.fill_round_1(&keychain, &mut context)?;

    context.initial_sec_key = context.sec_key.clone();

    if !is_initiator {
        // perform partial sig
        slate.fill_round_2(&keychain, &context.sec_key, &context.sec_nonce)?;
        // update excess in stored transaction
        let mut batch = wallet.batch(keychain_mask)?;
        tx.kernel_excess = Some(slate.calc_excess(keychain.secp())?);
        batch.save_tx_log_entry(tx.clone(), &parent_key_id)?;
        batch.commit()?;
    }

    Ok(context)
}

/// Create context, without adding inputs to slate.
pub fn create_late_lock_context<'a, T: ?Sized, C, K>(
    wallet: &mut T,
    keychain_mask: Option<&SecretKey>,
    slate: &mut Slate,
    current_height: u64,
    init_tx_args: &InitTxArgs,
    parent_key_id: &Identifier,
    use_test_rng: bool,
) -> Result<Context, Error>
    where
        T: WalletBackend<'a, C, K>,
        C: NodeClient + 'a,
        K: Keychain + 'a,
{
    // sender should always refresh outputs
    refresh_outputs(wallet, keychain_mask, parent_key_id, false)?;

    // we're just going to run a selection to get the potential fee,
    // but this won't be locked
    let (_coins, _total, _amount, fee) = select_coins_and_fee(
        wallet,
        init_tx_args.amount,
        init_tx_args.amount_includes_fee.unwrap_or(false),
        current_height,
        init_tx_args.minimum_confirmations,
        init_tx_args.max_outputs as usize,
        init_tx_args.num_change_outputs as usize,
        init_tx_args.selection_strategy_is_use_all,
        &parent_key_id,
    )?;
    slate.fee_fields = FeeFields::new(0, fee)?;

    let keychain = wallet.keychain(keychain_mask)?;

    // Create our own private context
    let mut context = Context::new(keychain.secp(), &parent_key_id, use_test_rng, true);
    context.fee = Some(slate.fee_fields);
    context.amount = slate.amount;
    context.late_lock_args = Some(init_tx_args.clone());

    // Generate a blinding factor for the tx and add
    //  public key info to the slate
    slate.fill_round_1(&wallet.keychain(keychain_mask)?, &mut context)?;

    Ok(context)
}

/// Complete a transaction.
pub fn complete_tx<'a, T: ?Sized, C, K>(
    wallet: &mut T,
    keychain_mask: Option<&SecretKey>,
    slate: &mut Slate,
    context: &Context,
) -> Result<(), Error>
    where
        T: WalletBackend<'a, C, K>,
        C: NodeClient + 'a,
        K: Keychain + 'a,
{
    // when self sending invoice tx, use initiator nonce to finalize
    let (sec_key, sec_nonce) = {
        if context.initial_sec_key != context.sec_key
            && context.initial_sec_nonce != context.sec_nonce
        {
            (
                context.initial_sec_key.clone(),
                context.initial_sec_nonce.clone(),
            )
        } else {
            (context.sec_key.clone(), context.sec_nonce.clone())
        }
    };
    slate.fill_round_2(&wallet.keychain(keychain_mask)?, &sec_key, &sec_nonce)?;

    // Final transaction can be built by anyone at this stage
    trace!("Slate to finalize is: {}", slate);
    slate.finalize(&wallet.keychain(keychain_mask)?)?;
    Ok(())
}

/// Rollback outputs associated with a transaction in the wallet.
pub fn cancel_tx<'a, T: ?Sized, C, K>(
    wallet: &mut T,
    keychain_mask: Option<&SecretKey>,
    parent_key_id: &Identifier,
    tx_id: Option<u32>,
    tx_slate_id: Option<Uuid>,
) -> Result<(), Error>
    where
        T: WalletBackend<'a, C, K>,
        C: NodeClient + 'a,
        K: Keychain + 'a,
{
    let mut tx_id_string = String::new();
    if let Some(tx_id) = tx_id {
        tx_id_string = tx_id.to_string();
    } else if let Some(tx_slate_id) = tx_slate_id {
        tx_id_string = tx_slate_id.to_string();
    }
    let tx_vec = retrieve_txs(
        wallet,
        tx_id,
        tx_slate_id,
        None,
        Some(&parent_key_id),
        false,
    )?;
    if tx_vec.len() != 1 {
        return Err(Error::TransactionDoesntExist(tx_id_string));
    }
    let tx = tx_vec[0].clone();
    match tx.tx_type {
        TxLogEntryType::TxSent | TxLogEntryType::TxReceived | TxLogEntryType::TxReverted => {}
        _ => return Err(Error::TransactionNotCancellable(tx_id_string)),
    }
    if tx.confirmed {
        return Err(Error::TransactionNotCancellable(tx_id_string));
    }
    // get outputs associated with tx
    let res = retrieve_outputs(
        wallet,
        keychain_mask,
        false,
        Some(tx.id),
        Some(&parent_key_id),
    )?;
    let outputs = res.iter().map(|m| m.output.clone()).collect();
    cancel_tx_and_outputs(wallet, keychain_mask, tx, outputs, parent_key_id)?;
    Ok(())
}

/// Update the stored transaction (this update needs to happen when the TX is finalised).
pub fn update_stored_tx<'a, T: ?Sized, C, K>(
    wallet: &mut T,
    keychain_mask: Option<&SecretKey>,
    context: &Context,
    slate: &Slate,
    is_invoiced: bool,
) -> Result<(), Error>
    where
        T: WalletBackend<'a, C, K>,
        C: NodeClient + 'a,
        K: Keychain + 'a,
{
    // finalize command
    let tx_vec = retrieve_txs(wallet, None, Some(slate.id), None, None, false)?;
    let mut tx = None;
    // don't want to assume this is the right tx, in case of self-sending
    for t in tx_vec {
        if t.tx_type == TxLogEntryType::TxSent && !is_invoiced {
            tx = Some(t);
            break;
        }
        if t.tx_type == TxLogEntryType::TxReceived && is_invoiced {
            tx = Some(t);
            break;
        }
    }
    let mut tx = match tx {
        Some(t) => t,
        None => return Err(Error::TransactionDoesntExist(slate.id.to_string())),
    };
    let parent_key = tx.parent_key_id.clone();
    {
        let keychain = wallet.keychain(keychain_mask)?;
        tx.kernel_excess = Some(slate.calc_excess(keychain.secp())?);
    }

    if let Some(ref p) = slate.clone().payment_proof {
        let derivation_index = match context.payment_proof_derivation_index {
            Some(i) => i,
            None => 0,
        };
        let keychain = wallet.keychain(keychain_mask)?;
        let parent_key_id = wallet.parent_key_id();
        let excess = slate.calc_excess(keychain.secp())?;
        let sender_key =
            address::address_from_derivation_path(&keychain, &parent_key_id, derivation_index)?;
        let sender_address = OnionV3Address::from_private(&sender_key.0)?;
        let sig =
            create_payment_proof_signature(slate.amount, &excess, p.sender_address, sender_key)?;
        tx.payment_proof = Some(StoredProofInfo {
            receiver_address: p.receiver_address,
            receiver_signature: p.receiver_signature,
            sender_address_path: derivation_index,
            sender_address: sender_address.to_ed25519()?,
            sender_signature: Some(sig),
        })
    }

    wallet.store_tx(&format!("{}", tx.tx_slate_id.unwrap()), slate.tx_or_err()?)?;

    let mut batch = wallet.batch(keychain_mask)?;
    batch.save_tx_log_entry(tx, &parent_key)?;
    batch.commit()?;
    Ok(())
}

pub fn payment_proof_message(
    amount: u64,
    kernel_commitment: &pedersen::Commitment,
    sender_address: DalekPublicKey,
) -> Result<Vec<u8>, Error> {
    let mut msg = Vec::new();
    msg.write_u64::<BigEndian>(amount)?;
    msg.append(&mut kernel_commitment.0.to_vec());
    msg.append(&mut sender_address.to_bytes().to_vec());
    Ok(msg)
}

pub fn _decode_payment_proof_message(
    msg: &[u8],
) -> Result<(u64, pedersen::Commitment, DalekPublicKey), Error> {
    let mut rdr = Cursor::new(msg);
    let amount = rdr.read_u64::<BigEndian>()?;
    let mut commit_bytes = [0u8; 33];
    for i in 0..33 {
        commit_bytes[i] = rdr.read_u8()?;
    }
    let mut sender_address_bytes = [0u8; 32];
    for i in 0..32 {
        sender_address_bytes[i] = rdr.read_u8()?;
    }

    Ok((
        amount,
        pedersen::Commitment::from_vec(commit_bytes.to_vec()),
        DalekPublicKey::from_bytes(&sender_address_bytes).unwrap(),
    ))
}

/// Create a payment proof.
pub fn create_payment_proof_signature(
    amount: u64,
    kernel_commitment: &pedersen::Commitment,
    sender_address: DalekPublicKey,
    sec_key: SecretKey,
) -> Result<DalekSignature, Error> {
    let msg = payment_proof_message(amount, kernel_commitment, sender_address)?;
    let d_skey = match DalekSecretKey::from_bytes(&sec_key.0) {
        Ok(k) => k,
        Err(e) => {
            return Err(Error::ED25519Key(format!("{}", e)));
        }
    };
    let pub_key: DalekPublicKey = (&d_skey).into();
    let keypair = DalekKeypair {
        public: pub_key,
        secret: d_skey,
    };
    Ok(keypair.sign(&msg))
}

/// Verify all aspects of a completed payment proof on the current slate.
pub fn verify_slate_payment_proof<'a, T: ?Sized, C, K>(
    wallet: &mut T,
    keychain_mask: Option<&SecretKey>,
    parent_key_id: &Identifier,
    context: &Context,
    slate: &Slate,
) -> Result<(), Error>
    where
        T: WalletBackend<'a, C, K>,
        C: NodeClient + 'a,
        K: Keychain + 'a,
{
    let tx_vec = retrieve_txs(
        wallet,
        None,
        Some(slate.id),
        None,
        Some(parent_key_id),
        false,
    )?;
    if tx_vec.is_empty() {
        return Err(Error::PaymentProof(
            "TxLogEntry with original proof info not found (is account correct?)".to_owned(),
        ));
    }

    let orig_proof_info = tx_vec[0].clone().payment_proof;

    if orig_proof_info.is_some() && slate.payment_proof.is_none() {
        return Err(Error::PaymentProof(
            "Expected Payment Proof for this Transaction is not present".to_owned(),
        ));
    }

    if let Some(ref p) = slate.clone().payment_proof {
        let orig_proof_info = match orig_proof_info {
            Some(p) => p.clone(),
            None => {
                return Err(Error::PaymentProof(
                    "Original proof info not stored in tx".to_owned(),
                ));
            }
        };
        let keychain = wallet.keychain(keychain_mask)?;
        let index = match context.payment_proof_derivation_index {
            Some(i) => i,
            None => {
                return Err(Error::PaymentProof(
                    "Payment proof derivation index required".to_owned(),
                ));
            }
        };
        let orig_sender_sk =
            address::address_from_derivation_path(&keychain, parent_key_id, index)?;
        let orig_sender_address = OnionV3Address::from_private(&orig_sender_sk.0)?;
        if p.sender_address != orig_sender_address.to_ed25519()? {
            return Err(Error::PaymentProof(
                "Sender address on slate does not match original sender address".to_owned(),
            ));
        }

        if orig_proof_info.receiver_address != p.receiver_address {
            return Err(Error::PaymentProof(
                "Recipient address on slate does not match original recipient address".to_owned(),
            ));
        }
        let msg = payment_proof_message(
            slate.amount,
            &slate.calc_excess(&keychain.secp())?,
            orig_sender_address.to_ed25519()?,
        )?;
        let sig = match p.receiver_signature {
            Some(s) => s,
            None => {
                return Err(Error::PaymentProof(
                    "Recipient did not provide requested proof signature".to_owned(),
                ));
            }
        };

        if p.receiver_address.verify(&msg, &sig).is_err() {
            return Err(Error::PaymentProof("Invalid proof signature".to_owned()));
        };
    }
    Ok(())
}