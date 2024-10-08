use crate::networks::SupportedNetworks;
use crate::ValidatorExports;
use anyhow::{anyhow, Result};
use bip32::PrivateKey;
use ethers::abi::{Contract, Function, Token};
use ethers::core::k256::ecdsa::SigningKey;
use ethers::middleware::Middleware;
use ethers::prelude::{LocalWallet, Signer, U256};
use ethers::providers::{Http, Provider, ProviderExt};
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::types::{Address, Bytes, TransactionRequest};
use ethers::utils::keccak256;
use once_cell::sync::Lazy;
use std::time::Duration;

// this abi from:
//  https://github.com/ethereum/consensus-specs/blob/dev/solidity_deposit_contract/deposit_contract.json
static DEPOSIT_FUNC: Lazy<Function> = Lazy::new(|| {
    let json = r#"
    [{
	"inputs": [],
	"stateMutability": "nonpayable",
	"type": "constructor"
}, {
	"anonymous": false,
	"inputs": [{
		"indexed": false,
		"internalType": "bytes",
		"name": "pubkey",
		"type": "bytes"
	}, {
		"indexed": false,
		"internalType": "bytes",
		"name": "withdrawal_credentials",
		"type": "bytes"
	}, {
		"indexed": false,
		"internalType": "bytes",
		"name": "amount",
		"type": "bytes"
	}, {
		"indexed": false,
		"internalType": "bytes",
		"name": "signature",
		"type": "bytes"
	}, {
		"indexed": false,
		"internalType": "bytes",
		"name": "index",
		"type": "bytes"
	}],
	"name": "DepositEvent",
	"type": "event"
}, {
	"inputs": [{
		"internalType": "bytes",
		"name": "pubkey",
		"type": "bytes"
	}, {
		"internalType": "bytes",
		"name": "withdrawal_credentials",
		"type": "bytes"
	}, {
		"internalType": "bytes",
		"name": "signature",
		"type": "bytes"
	}, {
		"internalType": "bytes32",
		"name": "deposit_data_root",
		"type": "bytes32"
	}],
	"name": "deposit",
	"outputs": [],
	"stateMutability": "payable",
	"type": "function"
}, {
	"inputs": [],
	"name": "get_deposit_count",
	"outputs": [{
		"internalType": "bytes",
		"name": "",
		"type": "bytes"
	}],
	"stateMutability": "view",
	"type": "function"
}, {
	"inputs": [],
	"name": "get_deposit_root",
	"outputs": [{
		"internalType": "bytes32",
		"name": "",
		"type": "bytes32"
	}],
	"stateMutability": "view",
	"type": "function"
}, {
	"inputs": [{
		"internalType": "bytes4",
		"name": "interfaceId",
		"type": "bytes4"
	}],
	"name": "supportsInterface",
	"outputs": [{
		"internalType": "bool",
		"name": "",
		"type": "bool"
	}],
	"stateMutability": "pure",
	"type": "function"
}]
    "#;

    let deserialized: Contract = serde_json::from_str(json).unwrap();
    deserialized.functions.get("deposit").unwrap()[0].clone()
});

pub fn public_key_address(public_key_bytes: &[u8]) -> [u8; 20] {
    let hash = keccak256(&public_key_bytes[1..]);

    let mut address = [0u8; 20];
    address.copy_from_slice(&hash[12..]);
    address
}

pub async fn staking(
    rpc: &str,
    network: &SupportedNetworks,
    validator_exports: &ValidatorExports,
    from_path: &str,
) -> Result<()> {
    println!("staking start...");

    // - gen cli
    let provider = Provider::<Http>::connect(rpc).await;

    // - check rpc and network is match
    {
        let rpc_chain_id = provider.get_chainid().await?;

        if rpc_chain_id.as_u64() != network.chain_id() {
            return Err(anyhow!(
                "rpc_chain_id is not the same as chain_id, {}:{}",
                rpc_chain_id.as_u64(),
                network.chain_id()
            ));
        }
    }

    // - get private key
    let (wallet, from) = {
        let key_str = tokio::fs::read_to_string(from_path).await?;
        let key_str = key_str.trim();

        let key_bytes = if key_str.contains("0x") {
            hex::decode(&key_str[2..])?
        } else {
            hex::decode(&key_str)?
        };

        let private_key = SigningKey::from_slice(&key_bytes)?;

        let public_key = private_key.public_key();
        let address = Address::from_slice(&public_key_address(
            public_key.to_encoded_point(false).as_bytes(),
        ));

        let wallet = LocalWallet::from(private_key).with_chain_id(network.chain_id());

        (wallet, address)
    };

    // - gen tx data
    let tx_data = {
        let withdrawal_credentials =
            hex::decode(&validator_exports.deposit_data[0].withdrawal_credentials)?;
        let signature = hex::decode(&validator_exports.deposit_data[0].signature)?;
        let deposit_data_root = hex::decode(&validator_exports.deposit_data[0].deposit_data_root)?;
        let pk = hex::decode(&validator_exports.deposit_data[0].pubkey)?;

        DEPOSIT_FUNC.encode_input(&[
            Token::Bytes(pk),
            Token::Bytes(withdrawal_credentials),
            Token::Bytes(signature),
            Token::FixedBytes(deposit_data_root),
        ])?
    };

    // - preprocessing
    let tx_bytes = {
        let mut tx = TransactionRequest {
            from: Some(from),
            to: Some(network.staking_address()?.into()),
            gas: None,
            gas_price: None,
            value: None,
            data: Some(Bytes::from(tx_data)),
            nonce: None,
            chain_id: None,
        };

        tx.value = Some(U256::from_dec_str("32000000000000000000")?.into());

        let gas_price = provider.get_gas_price().await?;
        tx.gas_price = Some(gas_price);

        let gas_limit = provider
            .estimate_gas(&TypedTransaction::Legacy(tx.clone()), None)
            .await?;
        tx.gas = Some(U256::from(gas_limit));

        let nonce = provider.get_transaction_count(from, None).await?;
        tx.nonce = Some(nonce);

        println!("tx: {tx:?}");

        let sign = wallet
            .sign_transaction(&TypedTransaction::Legacy(tx.clone()))
            .await?;

        tx.rlp_signed(&sign)
    };

    // - send tx
    {
        let pending = {
            let pending = provider.send_raw_transaction(tx_bytes).await?;

            let pending = pending.retries(3);

            let pending = pending.interval(Duration::from_secs(5));

            pending
        };

        println!("pending: {:?}", pending);

        if let Some(receipt) = pending.await? {
            println!("tx hash: {:?}", receipt.transaction_hash);

            if let Some(code) = receipt.status {
                println!("get receipt status: [{}]", code.as_u64());
            } else {
                println!("get receipt status is null, receipt: [{receipt:?}]");
            }

            println!("receipt: {:?}", receipt);
        } else {
            println!("get receipt is null");
        };
    }

    Ok(())
}
