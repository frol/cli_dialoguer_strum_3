use dialoguer::Input;

pub mod operation_mode;

#[derive(Debug, Default, Clone, clap::Clap)]
pub struct CliTransaction {
    transaction: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    transaction: String,
}

impl CliTransaction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = std::collections::VecDeque::new();
        if let Some(transaction) = &self.transaction {
            args.push_front(transaction.to_string());
        }
        args
    }
}

impl From<Transaction> for CliTransaction {
    fn from(transaction: Transaction) -> Self {
        Self {
            transaction: Some(transaction.transaction),
        }
    }
}

impl From<CliTransaction> for Transaction {
    fn from(item: CliTransaction) -> Self {
        let transaction = match item.transaction {
            Some(transaction) => transaction,
            None => Transaction::input_transaction(),
        };
        Self { transaction }
    }
}

impl Transaction {
    fn input_transaction() -> String {
        Input::new()
            .with_prompt("Enter the signed transaction hash you want to send")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        println!("Transaction sent ...");
        let json_rcp_client =
            near_jsonrpc_client::new_client(network_connection_config.rpc_url().as_str());
        let transaction_info = loop {
            let transaction_info_result = json_rcp_client
                .broadcast_tx_commit(self.transaction.clone())
                .await;
            match transaction_info_result {
                Ok(response) => {
                    break response;
                }
                Err(err) => {
                    if let Some(serde_json::Value::String(data)) = &err.data {
                        if data.contains("Timeout") {
                            println!("Timeout error transaction.\nPlease wait. The next try to send this transaction is happening right now ...");
                            continue;
                        }
                    }
                    return Err(color_eyre::Report::msg(format!(
                        "Error transaction: {:?}",
                        err
                    )));
                }
            };
        };
        crate::common::print_transaction_status(transaction_info, Some(network_connection_config))
            .await;
        Ok(())
    }
}
