use std::convert::TryInto;

use near_primitives::borsh::BorshDeserialize;

#[derive(
    Debug,
    Clone,
    strum_macros::IntoStaticStr,
    strum_macros::EnumString,
    strum_macros::EnumVariantNames,
    smart_default::SmartDefault,
)]
#[strum(serialize_all = "snake_case")]
pub enum OutputFormat {
    #[default]
    Plaintext,
    Json,
}

#[derive(Debug, Clone)]
pub struct TransactionAsBase64 {
    pub inner: near_primitives::transaction::Transaction,
}

impl std::str::FromStr for TransactionAsBase64 {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            inner: near_primitives::transaction::Transaction::try_from_slice(
                &near_primitives::serialize::from_base64(s)
                    .map_err(|err| format!("base64 transaction sequence is invalid: {}", err))?,
            )
            .map_err(|err| format!("transaction could not be parsed: {}", err))?,
        })
    }
}

impl std::fmt::Display for TransactionAsBase64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Transaction {}", self.inner.get_hash_and_size().0)
    }
}

#[derive(Debug, Clone)]
pub struct SignedTransactionAsBase64 {
    pub inner: near_primitives::transaction::SignedTransaction,
}

impl std::str::FromStr for SignedTransactionAsBase64 {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            inner: near_primitives::transaction::SignedTransaction::try_from_slice(
                &near_primitives::serialize::from_base64(s).map_err(|err| {
                    format!("base64 signed transaction sequence is invalid: {}", err)
                })?,
            )
            .map_err(|err| format!("signed transaction could not be parsed: {}", err))?,
        })
    }
}

impl std::fmt::Display for SignedTransactionAsBase64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Signed Transaction {}", self.inner.get_hash())
    }
}

#[derive(Debug, Clone)]
pub struct BlockHashAsBase58 {
    pub inner: near_primitives::hash::CryptoHash,
}

impl std::str::FromStr for BlockHashAsBase58 {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            inner: near_primitives::serialize::from_base(s)
                .map_err(|err| format!("base block hash sequence is invalid: {}", err))?
                .as_slice()
                .try_into()
                .map_err(|err| format!("block hash could not be collected: {}", err))?,
        })
    }
}

impl std::fmt::Display for BlockHashAsBase58 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlockHash {}", self.inner)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AvailableRpcServerUrl {
    pub inner: url::Url,
}

impl std::str::FromStr for AvailableRpcServerUrl {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url: url::Url =
            url::Url::parse(s).map_err(|err| format!("URL is not parsed: {}", err))?;
        actix::System::new()
            .block_on(async {
                near_jsonrpc_client::new_client(&url.as_str())
                    .status()
                    .await
            })
            .map_err(|err| format!("AvailableRpcServerUrl: {:?}", err))?;
        Ok(Self { inner: url })
    }
}

impl std::fmt::Display for AvailableRpcServerUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Url {}", self.inner)
    }
}

const ONE_NEAR: u128 = 10u128.pow(24);

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NearBalance {
    yoctonear_amount: u128,
}

impl NearBalance {
    pub fn from_yoctonear(yoctonear_amount: u128) -> Self {
        Self { yoctonear_amount }
    }

    pub fn to_yoctonear(&self) -> u128 {
        self.yoctonear_amount
    }
}

impl std::fmt::Display for NearBalance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.yoctonear_amount == 0 {
            write!(f, "0 NEAR")
        } else if self.yoctonear_amount < ONE_NEAR / 1_000 {
            write!(
                f,
                "less than 0.001 NEAR ({} yoctoNEAR)",
                self.yoctonear_amount
            )
        } else {
            write!(
                f,
                "{}.{:0>3} NEAR",
                self.yoctonear_amount / ONE_NEAR,
                self.yoctonear_amount / (ONE_NEAR / 1_000) % 1_000
            )
        }
    }
}

impl std::str::FromStr for NearBalance {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let num = s.trim().trim_end_matches(char::is_alphabetic).trim();
        let currency = s.trim().trim_start_matches(&num).trim().to_uppercase();
        let yoctonear_amount = match currency.as_str() {
            "N" | "NEAR" => {
                let res_split: Vec<&str> = num.split('.').collect();
                match res_split.len() {
                    2 => {
                        let num_int_yocto = res_split[0]
                            .parse::<u128>()
                            .map_err(|err| format!("Near Balance: {}", err))?
                            .checked_mul(10u128.pow(24))
                            .ok_or_else(|| "Near Balance: underflow or overflow happens")?;
                        let len_fract = res_split[1].len() as u32;
                        let num_fract_yocto = if len_fract <= 24 {
                            res_split[1]
                                .parse::<u128>()
                                .map_err(|err| format!("Near Balance: {}", err))?
                                .checked_mul(10u128.pow(24 - res_split[1].len() as u32))
                                .ok_or_else(|| "Near Balance: underflow or overflow happens")?
                        } else {
                            return Err(
                                "Near Balance: too large fractional part of a number".to_string()
                            );
                        };
                        num_int_yocto
                            .checked_add(num_fract_yocto)
                            .ok_or_else(|| "Near Balance: underflow or overflow happens")?
                    }
                    1 => res_split[0]
                        .parse::<u128>()
                        .map_err(|err| format!("Near Balance: {}", err))?
                        .checked_mul(10u128.pow(24))
                        .ok_or_else(|| "Near Balance: underflow or overflow happens")?,
                    _ => return Err("Near Balance: incorrect number entered".to_string()),
                }
            }
            "YN" | "YNEAR" | "YOCTONEAR" | "YOCTON" => num
                .parse::<u128>()
                .map_err(|err| format!("Near Balance: {}", err))?,
            _ => return Err("Near Balance: incorrect currency value entered".to_string()),
        };
        Ok(NearBalance { yoctonear_amount })
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NearGas {
    pub inner: u64,
}

impl std::fmt::Display for NearGas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} TeraGas", self.inner / 1000000000000)
    }
}

impl std::str::FromStr for NearGas {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let num = s.trim().trim_end_matches(char::is_alphabetic).trim();
        let currency = s.trim().trim_start_matches(&num).trim().to_uppercase();
        let number = match currency.as_str() {
            "T" | "TGAS" | "TERAGAS" => NearGas::into_tera_gas(num)?,
            "GIGAGAS" | "GGAS" => NearGas::into_tera_gas(num)? / 1000,
            _ => return Err("Near Gas: incorrect currency value entered".to_string()),
        };
        Ok(NearGas { inner: number })
    }
}

impl NearGas {
    fn into_tera_gas(num: &str) -> Result<u64, String> {
        let res_split: Vec<&str> = num.split('.').collect();
        match res_split.len() {
            2 => {
                let num_int_gas: u64 = res_split[0]
                    .parse::<u64>()
                    .map_err(|err| format!("Near Gas: {}", err))?
                    .checked_mul(10u64.pow(12))
                    .ok_or_else(|| "Near Gas: underflow or overflow happens")?;
                let len_fract = res_split[1].len() as u32;
                let num_fract_gas = if len_fract <= 12 {
                    res_split[1]
                        .parse::<u64>()
                        .map_err(|err| format!("Near Gas: {}", err))?
                        .checked_mul(10u64.pow(12 - res_split[1].len() as u32))
                        .ok_or_else(|| "Near Gas: underflow or overflow happens")?
                } else {
                    return Err("Near Gas: too large fractional part of a number".to_string());
                };
                Ok(num_int_gas
                    .checked_add(num_fract_gas)
                    .ok_or_else(|| "Near Gas: underflow or overflow happens")?)
            }
            1 => Ok(res_split[0]
                .parse::<u64>()
                .map_err(|err| format!("Near Gas: {}", err))?
                .checked_mul(10u64.pow(12))
                .ok_or_else(|| "Near Gas: underflow or overflow happens")?),
            _ => return Err("Near Gas: incorrect number entered".to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConnectionConfig {
    Testnet,
    Mainnet,
    Betanet,
    Custom { url: url::Url },
}

impl ConnectionConfig {
    pub fn rpc_url(&self) -> url::Url {
        match self {
            Self::Testnet => crate::consts::TESTNET_API_SERVER_URL.parse().unwrap(),
            Self::Mainnet => crate::consts::MAINNET_API_SERVER_URL.parse().unwrap(),
            Self::Betanet => crate::consts::BETANET_API_SERVER_URL.parse().unwrap(),
            Self::Custom { url } => url.clone(),
        }
    }

    pub fn archival_rpc_url(&self) -> url::Url {
        match self {
            Self::Testnet => crate::consts::TESTNET_ARCHIVAL_API_SERVER_URL
                .parse()
                .unwrap(),
            Self::Mainnet => crate::consts::MAINNET_ARCHIVAL_API_SERVER_URL
                .parse()
                .unwrap(),
            Self::Betanet => crate::consts::BETANET_ARCHIVAL_API_SERVER_URL
                .parse()
                .unwrap(),
            Self::Custom { url } => url.clone(),
        }
    }

    pub fn wallet_url(&self) -> url::Url {
        match self {
            Self::Testnet => crate::consts::TESTNET_WALLET_URL.parse().unwrap(),
            Self::Mainnet => crate::consts::MAINNET_WALLET_URL.parse().unwrap(),
            Self::Betanet => crate::consts::BETANET_WALLET_URL.parse().unwrap(),
            Self::Custom { url } => url.clone(),
        }
    }

    pub fn dir_name(&self) -> &str {
        match self {
            Self::Testnet => crate::consts::DIR_NAME_TESTNET,
            Self::Mainnet => crate::consts::DIR_NAME_MAINNET,
            Self::Betanet => crate::consts::DIR_NAME_BETANET,
            Self::Custom { url: _ } => crate::consts::DIR_NAME_CUSTOM,
        }
    }
}

#[derive(Debug)]
pub struct KeyPairProperties {
    pub seed_phrase_hd_path: slip10::BIP32Path,
    pub master_seed_phrase: String,
    pub implicit_account_id: String,
    pub public_key_str: String,
    pub secret_keypair_str: String,
}

pub async fn generate_keypair(
    master_seed_phrase: Option<&str>,
    new_master_seed_phrase_words_count: usize,
    seed_phrase_hd_path: slip10::BIP32Path,
) -> color_eyre::eyre::Result<KeyPairProperties> {
    let (master_seed_phrase, master_seed) = if let Some(master_seed_phrase) = master_seed_phrase {
        (
            master_seed_phrase.to_owned(),
            bip39::Mnemonic::parse(master_seed_phrase)?.to_seed(""),
        )
    } else {
        let mnemonic = bip39::Mnemonic::generate(new_master_seed_phrase_words_count)?;
        let master_seed_phrase = mnemonic.word_iter().collect::<Vec<&str>>().join(" ");
        (master_seed_phrase, mnemonic.to_seed(""))
    };

    let derived_private_key =
        slip10::derive_key_from_path(&master_seed, slip10::Curve::Ed25519, &seed_phrase_hd_path)
            .map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Failed to derive a key from the master key: {}",
                    err
                ))
            })?;

    let secret_keypair = {
        let secret = ed25519_dalek::SecretKey::from_bytes(&derived_private_key.key)?;
        let public = ed25519_dalek::PublicKey::from(&secret);
        ed25519_dalek::Keypair { secret, public }
    };

    let implicit_account_id = hex::encode(&secret_keypair.public);
    let public_key_str = format!(
        "ed25519:{}",
        bs58::encode(&secret_keypair.public).into_string()
    );
    let secret_keypair_str = format!(
        "ed25519:{}",
        bs58::encode(secret_keypair.to_bytes()).into_string()
    );
    let key_pair_properties: KeyPairProperties = KeyPairProperties {
        seed_phrase_hd_path,
        master_seed_phrase,
        implicit_account_id,
        public_key_str,
        secret_keypair_str,
    };
    Ok(key_pair_properties)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn near_balance_from_str_currency_near() {
        assert_eq!(
            NearBalance::from_str("10 near").unwrap(),
            NearBalance {
                yoctonear_amount: 10000000000000000000000000
            }
        ); // 26 number
        assert_eq!(
            NearBalance::from_str("10.055NEAR").unwrap(),
            NearBalance {
                yoctonear_amount: 10055000000000000000000000
            }
        ); // 26 number
    }
    #[test]
    fn near_balance_from_str_currency_n() {
        assert_eq!(
            NearBalance::from_str("10 n").unwrap(),
            NearBalance {
                yoctonear_amount: 10000000000000000000000000
            }
        ); // 26 number
        assert_eq!(
            NearBalance::from_str("10N ").unwrap(),
            NearBalance {
                yoctonear_amount: 10000000000000000000000000
            }
        ); // 26 number
    }
    #[test]
    fn near_balance_from_str_f64_near() {
        assert_eq!(
            NearBalance::from_str("0.000001 near").unwrap(),
            NearBalance {
                yoctonear_amount: 1000000000000000000
            }
        ); // 18 number
    }
    #[test]
    fn near_balance_from_str_f64_near_without_int() {
        let near_balance = NearBalance::from_str(".055NEAR");
        assert_eq!(
            near_balance,
            Err("Near Balance: cannot parse integer from empty string".to_string())
        );
    }
    #[test]
    fn near_balance_from_str_currency_ynear() {
        assert_eq!(
            NearBalance::from_str("100 ynear").unwrap(),
            NearBalance {
                yoctonear_amount: 100
            }
        );
        assert_eq!(
            NearBalance::from_str("100YNEAR ").unwrap(),
            NearBalance {
                yoctonear_amount: 100
            }
        );
    }
    #[test]
    fn near_balance_from_str_currency_yn() {
        assert_eq!(
            NearBalance::from_str("9000 YN  ").unwrap(),
            NearBalance {
                yoctonear_amount: 9000
            }
        );
        assert_eq!(
            NearBalance::from_str("0 yn").unwrap(),
            NearBalance {
                yoctonear_amount: 0
            }
        );
    }
    #[test]
    fn near_balance_from_str_currency_yoctonear() {
        assert_eq!(
            NearBalance::from_str("111YOCTONEAR").unwrap(),
            NearBalance {
                yoctonear_amount: 111
            }
        );
        assert_eq!(
            NearBalance::from_str("333 yoctonear").unwrap(),
            NearBalance {
                yoctonear_amount: 333
            }
        );
    }
    #[test]
    fn near_balance_from_str_currency_yocton() {
        assert_eq!(
            NearBalance::from_str("10YOCTON").unwrap(),
            NearBalance {
                yoctonear_amount: 10
            }
        );
        assert_eq!(
            NearBalance::from_str("10 yocton      ").unwrap(),
            NearBalance {
                yoctonear_amount: 10
            }
        );
    }
    #[test]
    fn near_balance_from_str_f64_ynear() {
        let near_balance = NearBalance::from_str("0.055yNEAR");
        assert_eq!(
            near_balance,
            Err("Near Balance: invalid digit found in string".to_string())
        );
    }
    #[test]
    fn near_balance_from_str_without_currency() {
        let near_balance = NearBalance::from_str("100");
        assert_eq!(
            near_balance,
            Err("Near Balance: incorrect currency value entered".to_string())
        );
    }
    #[test]
    fn near_balance_from_str_incorrect_currency() {
        let near_balance = NearBalance::from_str("100 UAH");
        assert_eq!(
            near_balance,
            Err("Near Balance: incorrect currency value entered".to_string())
        );
    }
    #[test]
    fn near_balance_from_str_invalid_double_dot() {
        let near_balance = NearBalance::from_str("100.55.");
        assert_eq!(
            near_balance,
            Err("Near Balance: incorrect currency value entered".to_string())
        );
    }
    #[test]
    fn near_balance_from_str_large_fractional_part() {
        let near_balance = NearBalance::from_str("100.1111122222333334444455555 n"); // 25 symbols after "."
        assert_eq!(
            near_balance,
            Err("Near Balance: too large fractional part of a number".to_string())
        );
    }
    #[test]
    fn near_balance_from_str_large_int_part() {
        let near_balance = NearBalance::from_str("1234567890123456.0 n");
        assert_eq!(
            near_balance,
            Err("Near Balance: underflow or overflow happens".to_string())
        );
    }
    #[test]
    fn near_balance_from_str_without_fractional_part() {
        let near_balance = NearBalance::from_str("100. n");
        assert_eq!(
            near_balance,
            Err("Near Balance: cannot parse integer from empty string".to_string())
        );
    }
    #[test]
    fn near_balance_from_str_negative_value() {
        let near_balance = NearBalance::from_str("-100 n");
        assert_eq!(
            near_balance,
            Err("Near Balance: invalid digit found in string".to_string())
        );
    }

    #[test]
    fn near_balance_from_str_currency_tgas() {
        assert_eq!(
            NearGas::from_str("10 tgas").unwrap(),
            NearGas {
                inner: 10000000000000
            }
        ); // 14 number
        assert_eq!(
            NearGas::from_str("10.055TERAGAS").unwrap(),
            NearGas {
                inner: 10055000000000
            }
        ); // 14 number
    }
    #[test]
    fn near_gas_from_str_currency_gigagas() {
        assert_eq!(
            NearGas::from_str("10 gigagas").unwrap(),
            NearGas { inner: 10000000000 }
        ); // 11 number
        assert_eq!(
            NearGas::from_str("10GGAS ").unwrap(),
            NearGas { inner: 10000000000 }
        ); // 11 number
    }
    #[test]
    fn near_gas_from_str_f64_tgas() {
        assert_eq!(
            NearGas::from_str("0.000001 tgas").unwrap(),
            NearGas { inner: 1000000 }
        ); // 7 number
    }
    #[test]
    fn near_gas_from_str_f64_gas_without_int() {
        let near_gas = NearGas::from_str(".055ggas");
        assert_eq!(
            near_gas,
            Err("Near Gas: cannot parse integer from empty string".to_string())
        );
    }
    #[test]
    fn near_gas_from_str_without_currency() {
        let near_gas = NearGas::from_str("100");
        assert_eq!(
            near_gas,
            Err("Near Gas: incorrect currency value entered".to_string())
        );
    }
    #[test]
    fn near_gas_from_str_incorrect_currency() {
        let near_gas = NearGas::from_str("100 UAH");
        assert_eq!(
            near_gas,
            Err("Near Gas: incorrect currency value entered".to_string())
        );
    }
    #[test]
    fn near_gas_from_str_invalid_double_dot() {
        let near_gas = NearGas::from_str("100.55.");
        assert_eq!(
            near_gas,
            Err("Near Gas: incorrect currency value entered".to_string())
        );
    }
    #[test]
    fn near_gas_from_str_large_fractional_part() {
        let near_gas = NearGas::from_str("100.1111122222333 ggas"); // 13 symbols after "."
        assert_eq!(
            near_gas,
            Err("Near Gas: too large fractional part of a number".to_string())
        );
    }
    #[test]
    fn near_gas_from_str_large_int_part() {
        let near_gas = NearGas::from_str("200123456789123.0 tgas");
        assert_eq!(
            near_gas,
            Err("Near Gas: underflow or overflow happens".to_string())
        );
    }
    #[test]
    fn near_gas_from_str_negative_value() {
        let near_gas = NearGas::from_str("-100 ggas");
        assert_eq!(
            near_gas,
            Err("Near Gas: invalid digit found in string".to_string())
        );
    }
}
