use crate::{key_material::KdfVariant, networks::SupportedNetworks, output, Validators};
use clap::{arg, Parser};
use serde_derive::Deserialize;
use serde_with::{serde_as, NoneAsEmptyString};
use std::fs::read_to_string;

#[serde_as]
#[derive(Parser, Clone, Deserialize, Debug)]
pub struct NewMnemonicSubcommandOpts {
    /// The name of Ethereum PoS chain you are targeting.
    ///
    /// Use "mainnet" if you are
    /// depositing ETH
    #[arg(value_enum, long)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub chain: Option<SupportedNetworks>,

    /// The number of new validator keys you want to
    /// generate.
    ///
    /// You can always generate more later
    #[arg(long, visible_alias = "num_validators", default_value_t = 0)]
    pub num_validators: u32,

    /// The password that will secure your keystores.
    ///
    /// You will need to re-enter this to
    /// decrypt them when you setup your Ethereum
    /// validators. If omitted, keystores will not be generated.
    #[arg(long, visible_alias = "keystore_password")]
    pub keystore_password: Option<String>,

    /// The index of the first validator's keys you wish to generate the address for
    // e.g. if you generated 3 keys before (index #0, index #1, index #2)
    // and you want to generate for the 2nd validator,
    // the validator_start_index would be 1.
    // If no index specified, it will be set to 0.
    #[arg(long, visible_alias = "validator_start_index")]
    pub validator_start_index: Option<u32>,

    /// If this field is set and valid, the given
    /// value will be used to set the
    /// withdrawal credentials. Otherwise, it will
    /// generate withdrawal credentials with the
    /// mnemonic-derived withdrawal public key. Valid formats are
    /// ^(0x[a-fA-F0-9]{40})$ for execution addresses,
    /// ^(0x01[0]{22}[a-fA-F0-9]{40})$ for execution withdrawal credentials
    /// and ^(0x00[a-fA-F0-9]{62})$ for BLS withdrawal credentials.
    #[arg(long, visible_alias = "withdrawal_credentials")]
    pub withdrawal_credentials: Option<String>,

    /// Use this argument to select the key derivation function for the keystores.
    ///
    /// Depending on your use case with `scrypt` using higher security parameters
    /// and consequently slower performance vs `pbkdf2`,
    /// achieving better performance with lower security parameters compared to `scrypt`
    #[arg(long)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub kdf: Option<KdfVariant>,

    /// Path to a custom Eth PoS chain config
    #[arg(long, visible_alias = "testnet_config")]
    #[serde_as(as = "NoneAsEmptyString")]
    pub testnet_config: Option<String>,

    /// A version of CLI to include into generated deposit data
    #[arg(long, visible_alias = "deposit_cli_version", default_value = "2.7.0")]
    pub deposit_cli_version: String,

    #[arg(long, default_value = "./")]
    pub output: String,
}

impl NewMnemonicSubcommandOpts {
    pub fn run(&self) {
        let mut opt = self.clone();

        if let Ok(config_path) = std::env::var("new_mnemonic_config") {
            let str = read_to_string(config_path).unwrap();
            opt = toml::from_str(&str).unwrap();
        }

        let chain = if opt.chain.is_some() && opt.testnet_config.is_some() {
            panic!("should only pass one of testnet_config or chain")
        } else if opt.testnet_config.is_some() {
            // Signalizes custom testnet config will be used
            None
        } else {
            opt.chain.clone()
        };

        let password = opt
            .keystore_password
            .clone()
            .map(|p| p.as_bytes().to_owned());

        let validators = Validators::new(
            None,
            password,
            Some(opt.num_validators),
            None,
            self.withdrawal_credentials.is_none(),
            self.kdf.clone(),
        );
        let export = validators
            .export(
                chain,
                opt.withdrawal_credentials.clone(),
                32_000_000_000,
                opt.deposit_cli_version.clone(),
                opt.testnet_config.clone(),
            )
            .unwrap();
        let export_json: serde_json::Value = export
            .clone()
            .try_into()
            .expect("could not serialise validator export");
        let export_json =
            serde_json::to_string_pretty(&export_json).expect("could not parse validator export");
        println!("{}", export_json);

        output::output(&export, &opt.output).unwrap();
    }
}
