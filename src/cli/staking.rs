use crate::networks::SupportedNetworks;
use crate::staking::staking;
use crate::ValidatorExports;
use clap::Parser;
use serde_derive::Deserialize;
use serde_with::{serde_as, NoneAsEmptyString};
use std::fs::read_to_string;
#[serde_as]
#[derive(Parser, Clone, Deserialize, Debug)]
pub struct StakingCommandOpt {
    #[arg(value_enum, long)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub chain: Option<SupportedNetworks>,

    #[arg(long)]
    pub rpc: Option<String>,

    #[arg(long)]
    pub from_path: Option<String>,

    #[arg(long)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub staking_address: Option<String>,

    #[arg(long)]
    pub export_path: Option<String>,
}

impl StakingCommandOpt {
    pub fn run(&self) {
        let mut opt = self.clone();

        if let Ok(config_path) = std::env::var("staking_config") {
            let str = read_to_string(config_path).unwrap();
            opt = toml::from_str(&str).unwrap();
        }

        let export_str = read_to_string(&opt.export_path.unwrap()).unwrap();
        let export = serde_json::from_str::<ValidatorExports>(&export_str).unwrap();

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async move {
            let network = opt.chain.as_ref().unwrap();

            if let Err(e) = staking(
                &opt.rpc.unwrap(),
                network,
                &export,
                &opt.from_path.unwrap(),
                opt.staking_address,
            )
            .await
            {
                eprintln!("staking err {e:?}");
            }
        });
    }
}
