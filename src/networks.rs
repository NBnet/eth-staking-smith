use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use serde_derive::Deserialize;
use serde_with::serde_as;
use std::collections::HashMap;
use std::str::FromStr;
use types::{Address, Hash256};

#[serde_as]
#[derive(clap::ValueEnum, Clone, Hash, Eq, PartialEq, Deserialize, Debug)]
pub enum SupportedNetworks {
    Mainnet,
    Holesky,
    // These are legacy networks they are supported on best effort basis
    Prater,
    Goerli,
    Custom,
}

impl FromStr for SupportedNetworks {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mainnet" => Ok(SupportedNetworks::Mainnet),
            "holesky" => Ok(SupportedNetworks::Holesky),
            "goerli" => Ok(SupportedNetworks::Goerli),
            "prater" => Ok(SupportedNetworks::Goerli),
            "custom" => Ok(SupportedNetworks::Custom),
            _ => Err(format!("{} is not a supported SupportedNetworks", s)),
        }
    }
}

impl std::fmt::Display for SupportedNetworks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SupportedNetworks::Mainnet => "mainnet",
            SupportedNetworks::Holesky => "holesky",
            SupportedNetworks::Prater => "goerli",
            SupportedNetworks::Goerli => "goerli",
            SupportedNetworks::Custom => "custom",
        };
        write!(f, "{}", s)
    }
}

impl SupportedNetworks {
    pub fn chain_id(&self) -> u64 {
        match self {
            SupportedNetworks::Mainnet => 1,
            SupportedNetworks::Holesky => 17000,
            SupportedNetworks::Prater => 5,
            SupportedNetworks::Goerli => 5,
            SupportedNetworks::Custom => 0,
        }
    }

    pub fn staking_address(&self) -> Result<Address> {
        let address_str = match self {
            SupportedNetworks::Mainnet => "0x00000000219ab540356cBB839Cbe05303d7705Fa".to_string(),
            SupportedNetworks::Holesky => "0x4242424242424242424242424242424242424242".to_string(),
            SupportedNetworks::Prater => return Err(anyhow!("not support prater")),
            SupportedNetworks::Goerli => return Err(anyhow!("not support goerli")),
            SupportedNetworks::Custom => "0x0000000000000000000000000000000000000000".to_string(),
        };

        let address = Address::from_str(&address_str)?;
        Ok(address)
    }
}

fn decode_genesis_validators_root(hex_value: &str) -> Hash256 {
    Hash256::from_slice(hex::decode(hex_value).unwrap().as_slice())
}

// Genesis validators root values are not present in chain spec,
// but instead acquired from genesis. The values below are well-known
// and taken from repositories in https://github.com/eth-clients organization.
lazy_static! {
    pub static ref GENESIS_VALIDATORS_ROOT_MAINNET: Hash256 = decode_genesis_validators_root(
        "4b363db94e286120d76eb905340fdd4e54bfe9f06bf33ff6cf5ad27f511bfe95"
    );
    pub static ref GENESIS_VALIDATORS_ROOT_HOLESKY: Hash256 = decode_genesis_validators_root(
        "9143aa7c615a7f7115e2b6aac319c03529df8242ae705fba9df39b79c59fa8b1"
    );
    pub static ref GENESIS_VALIDATORS_ROOT_GOERLI: Hash256 = decode_genesis_validators_root(
        "043db0d9a83813551ee2f33450d23797757d430911a9320530ad8a0eabc43efb"
    );
    pub static ref GENESIS_VALIDATOR_ROOT: HashMap<SupportedNetworks, Hash256> = HashMap::from([
        (
            SupportedNetworks::Mainnet,
            GENESIS_VALIDATORS_ROOT_MAINNET.to_owned()
        ),
        (
            SupportedNetworks::Prater,
            GENESIS_VALIDATORS_ROOT_GOERLI.to_owned()
        ),
        (
            SupportedNetworks::Goerli,
            GENESIS_VALIDATORS_ROOT_GOERLI.to_owned()
        ),
        (
            SupportedNetworks::Holesky,
            GENESIS_VALIDATORS_ROOT_HOLESKY.to_owned()
        ),
    ]);
}

pub(crate) fn validators_root_for(network: &SupportedNetworks) -> Hash256 {
    *GENESIS_VALIDATOR_ROOT.get(network).unwrap()
}
