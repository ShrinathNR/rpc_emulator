use std::path::PathBuf;

use litesvm::LiteSVM;
use solana_program::address_lookup_table::instruction::create_lookup_table;
use solana_program::address_lookup_table_account::AddressLookupTableAccount;
use solana_program::message::VersionedMessage;
use solana_program::{
    address_lookup_table::instruction::extend_lookup_table,
    instruction::{AccountMeta, Instruction},
    message::{v0::Message as MessageV0, Message},
    pubkey::Pubkey,
    rent::Rent,
};
use solana_sdk::transaction::{TransactionError, VersionedTransaction};
use solana_sdk::{
    account::Account,
    pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};
const NUM_GREETINGS: u8 = 255;

fn read_counter_program() -> Vec<u8> {
    let mut so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    so_path.push("counter/target/deploy/counter.so");
    std::fs::read(so_path).unwrap()
}

#[test]
pub fn integration_test() {
    let mut svm = LiteSVM::new();
    let payer_kp = Keypair::new();
    let payer_pk = payer_kp.pubkey();
    let program_id = pubkey!("GtdambwDgHWrDJdVPBkEHGhCwokqgAoch162teUjJse2");
    svm.add_program(program_id, &read_counter_program());
    svm.airdrop(&payer_pk, 1000000000).unwrap();
    let blockhash = svm.latest_blockhash();
    let counter_address = pubkey!("J39wvrFY2AkoAUCke5347RMNk3ditxZfVidoZ7U6Fguf");
    let _ = svm.set_account(
        counter_address,
        Account {
            lamports: 5,
            data: vec![0_u8; std::mem::size_of::<u32>()],
            owner: program_id,
            ..Default::default()
        },
    );
    assert_eq!(
        svm.get_account(&counter_address).unwrap().data,
        0u32.to_le_bytes().to_vec()
    );
    let num_greets = 2u8;
    for deduper in 0..num_greets {
        let tx = make_tx(
            program_id,
            counter_address,
            &payer_pk,
            blockhash,
            &payer_kp,
            deduper,
        );
        println!("transaction : {:#?}", tx);
        let tx_res = svm.send_transaction(tx).unwrap();
        println!("transaction result : {:#?}", tx_res);

    }
    assert_eq!(
        svm.get_account(&counter_address).unwrap().data,
        (num_greets as u32).to_le_bytes().to_vec()
    );
    println!("counter account data : {:#?}", svm.get_account(&counter_address).unwrap().data);
}

fn make_tx(
    program_id: Pubkey,
    counter_address: Pubkey,
    payer_pk: &Pubkey,
    blockhash: solana_program::hash::Hash,
    payer_kp: &Keypair,
    deduper: u8,
) -> Transaction {
    let msg = Message::new_with_blockhash(
        &[Instruction {
            program_id,
            accounts: vec![AccountMeta::new(counter_address, false)],
            data: vec![0, deduper],
        }],
        Some(payer_pk),
        &blockhash,
    );
    Transaction::new(&[payer_kp], msg, blockhash)
}

