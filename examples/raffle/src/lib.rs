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
    use serial_test::serial;
    use std::fs;
    use std::str::FromStr;
    use std::thread;

    use env_logger;
    use log::{debug, error, info, warn};

    fn setup() {
        env_logger::init();
    }
}
