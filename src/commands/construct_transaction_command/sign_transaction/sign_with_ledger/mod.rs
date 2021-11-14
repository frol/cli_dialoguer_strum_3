use dialoguer::Input;
use near_primitives::borsh::BorshSerialize;

/// Sign constructed transaction with Ledger
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliSignLedger {
    #[clap(long)]
    seed_phrase_hd_path: Option<slip10::BIP32Path>,
    #[clap(long)]
    nonce: Option<u64>,
    #[clap(long)]
    block_hash: Option<near_primitives::hash::CryptoHash>,
    #[clap(subcommand)]
    submit: Option<super::Submit>,
}

#[derive(Debug, Clone)]
pub struct SignLedger {
    pub seed_phrase_hd_path: slip10::BIP32Path,
    pub signer_public_key: near_crypto::PublicKey,
    nonce: Option<u64>,
    block_hash: Option<near_primitives::hash::CryptoHash>,
    pub submit: Option<super::Submit>,
}

impl CliSignLedger {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .submit
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        if let Some(nonce) = &self.nonce {
            args.push_front(nonce.to_string());
            args.push_front("--nonce".to_owned())
        }
        if let Some(block_hash) = &self.block_hash {
            args.push_front(block_hash.to_string());
            args.push_front("--block-hash".to_owned())
        }
        args
    }
}

impl From<SignLedger> for CliSignLedger {
    fn from(sign_ledger: SignLedger) -> Self {
        Self {
            seed_phrase_hd_path: Some(sign_ledger.seed_phrase_hd_path),
            nonce: sign_ledger.nonce,
            block_hash: sign_ledger.block_hash,
            submit: sign_ledger.submit.into(),
        }
    }
}

impl SignLedger {
    pub fn from(
        item: CliSignLedger,
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Self> {
        let seed_phrase_hd_path = match item.seed_phrase_hd_path {
            Some(hd_path) => hd_path,
            None => SignLedger::input_seed_phrase_hd_path(),
        };
        println!(
            "Please allow getting the PublicKey on Ledger device (HD Path: {})",
            seed_phrase_hd_path
        );
        let public_key = actix::System::new()
            .block_on(async { near_ledger::get_public_key(seed_phrase_hd_path.clone()).await })
            .map_err(|near_ledger_error| {
                color_eyre::Report::msg(format!(
                    "An error occurred while trying to get PublicKey from Ledger device: {:?}",
                    near_ledger_error
                ))
            })?;
        let signer_public_key = near_crypto::PublicKey::ED25519(
            near_crypto::ED25519PublicKey::from(public_key.to_bytes()),
        );
        let submit: Option<super::Submit> = item.submit;
        match connection_config {
            Some(_) => Ok(Self {
                seed_phrase_hd_path,
                signer_public_key,
                nonce: None,
                block_hash: None,
                submit,
            }),
            None => {
                let nonce: u64 = match item.nonce {
                    Some(cli_nonce) => cli_nonce,
                    None => super::input_access_key_nonce(&signer_public_key.to_string().clone()),
                };
                let block_hash = match item.block_hash {
                    Some(cli_block_hash) => cli_block_hash,
                    None => super::input_block_hash(),
                };
                Ok(Self {
                    seed_phrase_hd_path,
                    signer_public_key,
                    nonce: Some(nonce),
                    block_hash: Some(block_hash),
                    submit,
                })
            }
        }
    }
}

impl SignLedger {
    fn rpc_client(self, selected_server_url: &str) -> near_jsonrpc_client::JsonRpcClient {
        near_jsonrpc_client::new_client(&selected_server_url)
    }

    pub fn input_seed_phrase_hd_path() -> slip10::BIP32Path {
        Input::new()
            .with_prompt("Enter seed phrase HD Path (if you not sure leave blank for default)")
            .with_initial_text("44'/397'/0'/0'/1'")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
        let seed_phrase_hd_path = self.seed_phrase_hd_path.clone();
        let public_key = self.signer_public_key.clone();
        let nonce = self.nonce.unwrap_or_default().clone();
        let block_hash = self.block_hash.unwrap_or_default().clone();
        let submit: Option<super::Submit> = self.submit.clone();
        match connection_config.clone() {
            None => {
                let unsigned_transaction = near_primitives::transaction::Transaction {
                    public_key,
                    nonce,
                    block_hash,
                    ..prepopulated_unsigned_transaction
                };
                println!("\nUnsigned transaction:\n");
                crate::common::print_transaction(unsigned_transaction.clone());
                println!(
                    "Confirm transaction signing on your Ledger device (HD Path: {})",
                    seed_phrase_hd_path,
                );
                let signature = match near_ledger::sign_transaction(
                    unsigned_transaction
                        .try_to_vec()
                        .expect("Transaction is not expected to fail on serialization"),
                    seed_phrase_hd_path,
                )
                .await
                {
                    Ok(signature) => near_crypto::Signature::from_parts(
                        near_crypto::KeyType::ED25519,
                        &signature,
                    )
                    .expect("Signature is not expected to fail on deserialization"),
                    Err(near_ledger_error) => {
                        return Err(color_eyre::Report::msg(format!(
                            "Error occurred while signing the transaction: {:?}",
                            near_ledger_error
                        )));
                    }
                };

                let signed_transaction = near_primitives::transaction::SignedTransaction::new(
                    signature,
                    unsigned_transaction,
                );
                let serialize_to_base64 = near_primitives::serialize::to_base64(
                    signed_transaction
                        .try_to_vec()
                        .expect("Transaction is not expected to fail on serialization"),
                );
                println!("Your transaction was signed successfully.");
                match submit {
                    Some(submit) => submit.process_offline(serialize_to_base64),
                    None => {
                        let submit = super::Submit::choose_submit(connection_config.clone());
                        submit.process_offline(serialize_to_base64)
                    }
                }
            }
            Some(network_connection_config) => {
                let online_signer_access_key_response = self
                    .rpc_client(network_connection_config.rpc_url().as_str())
                    .query(near_jsonrpc_primitives::types::query::RpcQueryRequest {
                        block_reference: near_primitives::types::Finality::Final.into(),
                        request: near_primitives::views::QueryRequest::ViewAccessKey {
                            account_id: prepopulated_unsigned_transaction.signer_id.clone(),
                            public_key: public_key.clone(),
                        },
                    })
                    .await
                    .map_err(|err| {
                        color_eyre::Report::msg(format!(
                            "Failed to fetch public key information for nonce: {:?}",
                            err
                        ))
                    })?;
                let current_nonce =
                    if let near_jsonrpc_primitives::types::query::QueryResponseKind::AccessKey(
                        online_signer_access_key,
                    ) = online_signer_access_key_response.kind
                    {
                        online_signer_access_key.nonce
                    } else {
                        return Err(color_eyre::Report::msg(format!("Error current_nonce")));
                    };
                let unsigned_transaction = near_primitives::transaction::Transaction {
                    public_key,
                    block_hash: online_signer_access_key_response.block_hash,
                    nonce: current_nonce + 1,
                    ..prepopulated_unsigned_transaction
                };
                println!("\nUnsigned transaction:\n");
                crate::common::print_transaction(unsigned_transaction.clone());
                println!(
                    "Confirm transaction signing on your Ledger device (HD Path: {})",
                    seed_phrase_hd_path,
                );
                let signature = match near_ledger::sign_transaction(
                    unsigned_transaction
                        .try_to_vec()
                        .expect("Transaction is not expected to fail on serialization"),
                    seed_phrase_hd_path,
                )
                .await
                {
                    Ok(signature) => near_crypto::Signature::from_parts(
                        near_crypto::KeyType::ED25519,
                        &signature,
                    )
                    .expect("Signature is not expected to fail on deserialization"),
                    Err(near_ledger_error) => {
                        return Err(color_eyre::Report::msg(format!(
                            "Error occurred while signing the transaction: {:?}",
                            near_ledger_error
                        )));
                    }
                };

                let signed_transaction = near_primitives::transaction::SignedTransaction::new(
                    signature,
                    unsigned_transaction,
                );
                let serialize_to_base64 = near_primitives::serialize::to_base64(
                    signed_transaction
                        .try_to_vec()
                        .expect("Transaction is not expected to fail on serialization"),
                );
                println!("Your transaction was signed successfully.");
                match submit {
                    None => {
                        let submit = super::Submit::choose_submit(connection_config);
                        submit
                            .process_online(
                                network_connection_config,
                                signed_transaction,
                                serialize_to_base64,
                            )
                            .await
                    }
                    Some(submit) => {
                        submit
                            .process_online(
                                network_connection_config,
                                signed_transaction,
                                serialize_to_base64,
                            )
                            .await
                    }
                }
            }
        }
    }
}
