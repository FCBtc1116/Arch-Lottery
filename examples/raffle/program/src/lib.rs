use arch_program::{
    account::AccountInfo,
    entrypoint,
    helper::get_state_transition_tx,
    input_to_sign::InputToSign,
    instruction::Instruction,
    msg,
    program::{
        get_account_script_pubkey, get_bitcoin_tx, get_network_xonly_pubkey, invoke,
        next_account_info, set_return_data, set_transaction_to_sign, validate_utxo_ownership,
    },
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction::SystemInstruction,
    transaction_to_sign::TransactionToSign,
    utxo::UtxoMeta,
};
use std::collections::HashSet;
use bitcoin::{self, Transaction};
use borsh::{BorshDeserialize, BorshSerialize};

fn generate_random_numbers(seed: u64) -> Vec<u64> {
    let mut unique_numbers = HashSet::new();
    let mut rng_state = seed;

    // Generate 5 unique numbers in the range of 1 to 64
    while unique_numbers.len() < 5 {
        rng_state = rng_state.wrapping_mul(48271) % 0x7fffffff; // LCG parameters
        let num = rng_state % 64 + 1; // Get a number between 1 and 64
        unique_numbers.insert(num);
    }

    // Convert HashSet to Vec and sort it
    let mut numbers: Vec<u64> = unique_numbers.into_iter().collect();
    numbers.sort_unstable();

    // Generate one number in the range of 1 to 23
    rng_state = rng_state.wrapping_mul(48271) % 0x7fffffff; // Continue LCG
    let additional_num: u64 = rng_state % 23 + 1; // Get a number between 1 and 23
    numbers.push(additional_num);

    numbers
}

entrypoint!(lottery_bid);
pub fn lottery_bid(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> Result<(), ProgramError> {
    if accounts.len() != 1 {
        return Err(ProgramError::Custom(501));
    }

    let account_iter = &mut accounts.iter();
    let account = next_account_info(account_iter)?;

    let params: LotteryBidParams = borsh::from_slice(instruction_data).unwrap();
    let user_tx: Transaction = bitcoin::consensus::deserialize(&params.user_psbt).unwrap();

    let new_data = format!("Address {}, Number {:?}", params.address, params.user_select_number);

    // Borrow the account data
    let account_data = account.data.try_borrow().unwrap();

    // Create a new string that combines the existing data with the new data
    let existing_data = String::from_utf8_lossy(&account_data);
    let combined_data = format!("{}; {}", existing_data, new_data);

    msg!("Combined Data {:?}", combined_data);

    if combined_data.as_bytes().len() > account_data.len() {
        account.realloc(combined_data.len(), true)?;
    }

    let script_pubkey = get_account_script_pubkey(account.key);
    msg!("script_pubkey {:?}", script_pubkey);

    account
        .data
        .try_borrow_mut()
        .unwrap()
        .copy_from_slice(combined_data.as_bytes());

    msg!("Account Data {:?}", account_data);

    let mut tx = get_state_transition_tx(accounts);
    tx.input.push(user_tx.input[0].clone());

    let tx_to_sign = TransactionToSign {
        tx_bytes: &bitcoin::consensus::serialize(&tx),
        inputs_to_sign: &[InputToSign {
            index: 0,
            signer: account.key.clone(),
        }],
    };

    msg!("tx_to_sign{:?}", tx_to_sign);

    set_transaction_to_sign(accounts, tx_to_sign);

    Ok(())
}

pub fn select_winner(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> Result<(), ProgramError> {
    let random_numbers = generate_random_numbers(8012);
    println!("{:?}", random_numbers);

    Ok(())
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct LotteryBidParams {
    pub address: String,
    pub user_select_number: Vec<u8>,
    pub user_psbt: Vec<u8>,
}
