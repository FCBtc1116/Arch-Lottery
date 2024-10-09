#[cfg(test)]
mod tests {
    use arch_program::{
        account::AccountMeta, instruction::Instruction, pubkey::Pubkey,
        system_instruction::SystemInstruction, utxo::UtxoMeta,
    };
    use bitcoincore_rpc::{Auth, Client};
    use borsh::{BorshDeserialize, BorshSerialize};
    use common::constants::*;
    use common::helper::*;
    use common::models::*;
    // use serial_test::serial;
    use std::fs;
    use std::str::FromStr;
    use std::thread;

    use env_logger;
    use log::{debug, error, info, warn};

    fn setup() {
        env_logger::init();
    }

    #[derive(Clone, BorshSerialize, BorshDeserialize)]
    pub struct LotteryBidParams {
        pub address: String,
        pub user_select_number: Vec<u8>,
        pub user_psbt: Vec<u8>,
    }
    
    #[test]
    fn test_deploy_call() {
        setup();

        info!("Starting Test");
        
        // 2. Get program and caller key pairs
        let (program_keypair, program_pubkey) =
            with_secret_key_file(PROGRAM_FILE_PATH).expect("Failed to get program key pair");
        let (caller_keypair, caller_pubkey) =
            with_secret_key_file(CALLER_FILE_PATH).expect("Failed to get caller key pair");

        // 3. Send UTXO for program account
        let (txid, vout) = send_utxo(program_pubkey.clone());
        info!(
            "UTXO sent: {}:{} for program pubkey: {:?}",
            txid,
            vout,
            hex::encode(program_pubkey)
        );
        
        // 4. Create program account
        let (txid, instruction_hash) = sign_and_send_instruction(
            SystemInstruction::new_create_account_instruction(
                hex::decode(txid).unwrap().try_into().unwrap(),
                vout,
                program_pubkey.clone(),
            ),
            vec![program_keypair.clone()],
        )
        .expect("Failed to sign and send create account instruction");
        
        let processed_tx = get_processed_transaction(NODE1_ADDRESS, txid.clone())
            .expect("Failed to get processed transaction");
        debug!(
            "Processed transaction for account creation: {:?}",
            processed_tx
        );

        // 5. Deploy program
        let txids = deploy_program_txs(
            program_keypair.clone(),
            "program/target/sbf-solana-solana/release/raffleprogram.so",
        );
        info!("Program deployed with transaction IDs: {:?}", txids);

        // 6. Set program as executable
        let (txid, instruction_hash) = sign_and_send_instruction(
            Instruction {
                program_id: Pubkey::system_program(),
                accounts: vec![AccountMeta {
                    pubkey: program_pubkey.clone(),
                    is_signer: true,
                    is_writable: true,
                }],
                data: vec![2],
            },
            vec![program_keypair],
        )
        .expect("Failed to sign and send set executable instruction");

        let processed_tx = get_processed_transaction(NODE1_ADDRESS, txid.clone())
            .expect("Failed to get processed transaction");
        debug!(
            "Processed transaction for setting executable: {:?}",
            processed_tx
        );

        // 7. Verify program is executable
        assert!(
            read_account_info(NODE1_ADDRESS, program_pubkey.clone())
                .expect("Failed to read account info")
                .is_executable,
            "Program should be marked as executable"
        );

        // 8. Create caller account
        let (txid, vout) = send_utxo(caller_pubkey.clone());
        info!(
            "UTXO sent: {}:{} for caller pubkey: {:?}",
            txid,
            vout,
            hex::encode(caller_pubkey)
        );

        let (txid, instruction_hash) = sign_and_send_instruction(
            SystemInstruction::new_create_account_instruction(
                hex::decode(txid).unwrap().try_into().unwrap(),
                vout,
                caller_pubkey.clone(),
            ),
            vec![caller_keypair.clone()],
        )
        .expect("Failed to sign and send create caller account instruction");

        let processed_tx = get_processed_transaction(NODE1_ADDRESS, txid.clone())
            .expect("Failed to get processed transaction");
        debug!(
            "Processed transaction for caller account creation: {:?}",
            processed_tx
        );

        // 9. Assign ownership of caller account to program

        let mut instruction_data = vec![3];
        instruction_data.extend(program_pubkey.serialize());

        let (txid, instruction_hash) = sign_and_send_instruction(
            Instruction {
                program_id: Pubkey::system_program(),
                accounts: vec![AccountMeta {
                    pubkey: caller_pubkey.clone(),
                    is_signer: true,
                    is_writable: true
                }],
                data: instruction_data
            },
            vec![caller_keypair.clone()],
        )
        .expect("Failed to sign and send Assign ownership of caller account instruction");

        let processed_tx = get_processed_transaction(NODE1_ADDRESS, txid.clone())
            .expect("Failed to get processed transaction");
        debug!(
            "Processed transaction for caller account ownership assignment: {:?}",
            processed_tx
        );

        // 10. Verify that the program is owner of caller account
        assert_eq!(
            read_account_info(NODE1_ADDRESS, caller_pubkey.clone()).unwrap().owner, 
            program_pubkey,
            "Program should be owner of caller account"
        );

        let user_select_numbers = vec![1,2,3,4,6,7];

        // 11. Call Program
        let (txid, instruction_hash) = sign_and_send_instruction(
            Instruction {
                program_id: program_pubkey.clone(),
                accounts: vec![AccountMeta {
                    pubkey: caller_pubkey.clone(),
                    is_signer: true,
                    is_writable: true
                }],
                data: borsh::to_vec(&LotteryBidParams {
                    address: ("tb1qjv4jes2cm4sq2fva37kc6edpklanyrtr6z96kq").to_string(),
                    user_select_number: user_select_numbers,
                    user_psbt: hex::decode(prepare_fees()).unwrap(),
                }).unwrap()
            },
            vec![caller_keypair],
        ).expect("Failed to sign and send program call instruction");

        let processed_tx = get_processed_transaction(NODE1_ADDRESS, txid.clone())
            .expect("Failed to get processed transaction");
        debug!("Processed transaction for program call: {:?}", processed_tx);

        // 12. Check results
        let caller_account_info = read_account_info(NODE1_ADDRESS, caller_pubkey.clone())
            .expect("Failed to read caller account info");
        info!(
            "Caller account info after program call: {:?}",
            caller_account_info
        );
    }
}
