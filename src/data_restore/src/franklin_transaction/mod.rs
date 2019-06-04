use web3::futures::Future;
use web3::types::{H256, Transaction, TransactionId};

use helpers::*;
use blocks::LogBlockData;

#[derive(Debug, Copy, Clone)]
pub enum FranklinTransactionType {
    Deposit,
    Transfer,
    FullExit,
    Unknown
}

#[derive(Debug, Clone)]
pub struct FranklinTransaction {
    pub franklin_transaction_type: FranklinTransactionType,
    pub block_number: u32,
    pub ethereum_transaction: Transaction,
    pub commitment_data: Vec<u8>,
}

impl FranklinTransaction {
    pub fn get_transaction(config: &DataRestoreConfig, franklin_block: &LogBlockData) -> Option<Self> {
        let transaction = FranklinTransaction::get_ethereum_transaction(config, &franklin_block.transaction_hash)?;
        let input_data = FranklinTransaction::get_input_data_from_ethereum_transaction(&transaction);
        let tx_type = FranklinTransaction::get_transaction_type(&input_data);
        let commitment_data = FranklinTransaction::get_commitment_data_from_input_data(&input_data)?;
        let this = Self {
            franklin_transaction_type: tx_type,
            block_number: franklin_block.block_num,
            ethereum_transaction: transaction,
            commitment_data: commitment_data,
        };
        Some(this)
    }

    fn get_ethereum_transaction(config: &DataRestoreConfig, transaction_hash: &H256) -> Option<Transaction> {
        let tx_id = TransactionId::Hash(transaction_hash.clone());
        let (_eloop, transport) = web3::transports::Http::new(config.web3_endpoint.as_str()).ok()?;
        let web3 = web3::Web3::new(transport);
        let web3_transaction = web3.eth().transaction(tx_id).wait();
        let tx = match web3_transaction {
            Ok(tx) => {
                tx
            },
            Err(_) => { 
                None
            }
        };
        tx
    }

    fn get_input_data_from_ethereum_transaction(transaction: &Transaction) -> Vec<u8> {
        transaction.clone().input.0
    }

    fn get_commitment_data_from_input_data(input_data: &Vec<u8>) -> Option<Vec<u8>> {
        let input_data_contains_more_than_4_bytes = input_data.len() > 4;
        let commitment_data = match input_data_contains_more_than_4_bytes {
            true => Some(input_data[4..input_data.len()].to_vec()),
            false => None
        };
        commitment_data
    }

    fn get_transaction_type(input_data: &Vec<u8>) -> FranklinTransactionType {
        let input_data_contains_more_than_4_bytes = input_data.len() > 4;
        if input_data_contains_more_than_4_bytes == false {
            return FranklinTransactionType::Unknown
        }
        let deposit_method_bytes: Vec<u8> = vec![83, 61, 227, 10];
        let transaction_method_bytes: Vec<u8> = vec![244, 135, 178, 142];
        let full_exit_method_bytes: Vec<u8> = vec![121, 178, 173, 112];
        let method_bytes: Vec<u8> = input_data[0..4].to_vec();
        let method_type = match method_bytes {
            _ if method_bytes == deposit_method_bytes => {
                FranklinTransactionType::Deposit
            },
            _ if method_bytes == transaction_method_bytes => {
                FranklinTransactionType::Transfer
            },
            _ if method_bytes == full_exit_method_bytes => {
                FranklinTransactionType::FullExit
            },
            _ => {
                FranklinTransactionType::Unknown
            }
        };
        method_type
    }
}
