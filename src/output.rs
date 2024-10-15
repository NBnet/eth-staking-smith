use crate::ValidatorExports;
use anyhow::{anyhow, Result};
use std::fs;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

pub const VOTING_KEYSTORE_FILE: &str = "voting-keystore.json";
pub const DEPOSIT_KEYSTORE_FILE: &str = "deposit-data.json";
pub const EXPORT_FILE: &str = "export.json";
pub const MNEMONIC_FILE: &str = "mnemonic.json";
pub const PRIVATE_KEY_FILE: &str = "private-key";
pub const VALIDATOR_PREFIX: &str = "validator";
pub const VALIDATORS_PREFIX: &str = "validators";

pub fn output(export: &ValidatorExports, output_dir_str: &str) -> Result<()> {
    let check_output_dir = PathBuf::from(output_dir_str);
    if !is_dir(check_output_dir.as_path())? {
        return Err(anyhow!("output directory is not a directory"));
    }

    let output_dir = check_output_dir.join(VALIDATORS_PREFIX);
    create_dir_all(output_dir.clone())?;

    // save export
    let export_file = output_dir.join(EXPORT_FILE);
    let export_json = serde_json::to_string_pretty(export)?;
    fs::write(export_file, export_json)?;

    for (idx, deposit_data) in export.deposit_data.iter().enumerate() {
        // gen validator dir
        let validator_dir_name = format!("{VALIDATOR_PREFIX}-{idx}");
        let validator_dir = output_dir.join(validator_dir_name);
        create_dir_all(validator_dir.clone())?;

        // save deposit data
        let deposit_file = validator_dir.join(DEPOSIT_KEYSTORE_FILE);
        let deposit_json = serde_json::to_string_pretty(&deposit_data)?;
        fs::write(deposit_file, deposit_json)?;

        // save keystore
        let keystore_file = validator_dir.join(VOTING_KEYSTORE_FILE);
        let keystore = export
            .keystores
            .get(idx)
            .ok_or(anyhow!("index: {idx}, not match keystore"))?;
        let keystore_json = serde_json::to_string_pretty(&keystore)?;
        fs::write(keystore_file, keystore_json)?;

        // save private key
        let private_key_file = validator_dir.join(PRIVATE_KEY_FILE);
        let private_key = export
            .private_keys
            .get(idx)
            .ok_or(anyhow!("index: {idx}, not match private_key"))?;
        fs::write(private_key_file, private_key)?;

        // save mnemonic
        let mnemonic_file = validator_dir.join(MNEMONIC_FILE);
        let mnemonic_json = serde_json::to_string_pretty(&export.mnemonic)?;
        fs::write(mnemonic_file, mnemonic_json)?;
    }

    Ok(())
}

fn is_dir(path: &Path) -> Result<bool> {
    let metadata = path.metadata()?;
    Ok(metadata.is_dir())
}
