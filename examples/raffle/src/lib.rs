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


    }
}
