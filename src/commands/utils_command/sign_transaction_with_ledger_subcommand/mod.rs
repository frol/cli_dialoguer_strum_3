use dialoguer::Input;
use near_primitives::borsh::BorshSerialize;

/// Utility to sign transaction on Ledger
#[derive(Debug, Default, Clone, clap::Clap)]
pub struct CliSignTransactionWithLedger {
    #[clap(long)]
    seed_phrase_hd_path: Option<slip10::BIP32Path>,
    #[clap(long)]
    unsigned_transaction: Option<crate::common::TransactionAsBase64>,
}

#[derive(Debug, Clone)]
pub struct SignTransactionWithLedger {
    pub seed_phrase_hd_path: slip10::BIP32Path,
    pub unsigned_transaction: near_primitives::transaction::Transaction,
}

impl CliSignTransactionWithLedger {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = std::collections::VecDeque::new();
        if let Some(unsigned_transaction) = &self.unsigned_transaction {
            let unsigned_transaction_serialized_to_base64 = near_primitives::serialize::to_base64(
                unsigned_transaction
                    .inner
                    .try_to_vec()
                    .expect("Transaction is not expected to fail on serialization"),
            );
            args.push_front(unsigned_transaction_serialized_to_base64);
            args.push_front("--unsigned-transaction".to_string());
        }
        if let Some(seed_phrase_hd_path) = &self.seed_phrase_hd_path {
            args.push_front(seed_phrase_hd_path.to_string());
            args.push_front("--seed-phrase-hd-path".to_string());
        }
        args
    }
}

impl From<SignTransactionWithLedger> for CliSignTransactionWithLedger {
    fn from(sign_transaction_with_ledger: SignTransactionWithLedger) -> Self {
        Self {
            seed_phrase_hd_path: Some(sign_transaction_with_ledger.seed_phrase_hd_path),
            unsigned_transaction: Some(crate::common::TransactionAsBase64 {
                inner: sign_transaction_with_ledger.unsigned_transaction,
            }),
        }
    }
}

impl From<CliSignTransactionWithLedger> for SignTransactionWithLedger {
    fn from(item: CliSignTransactionWithLedger) -> Self {
        let seed_phrase_hd_path = match item.seed_phrase_hd_path {
            Some(hd_path) => hd_path,
            None => SignTransactionWithLedger::input_seed_phrase_hd_path(),
        };
        let unsigned_transaction: near_primitives::transaction::Transaction =
            match item.unsigned_transaction {
                Some(cli_unsigned_transaction) => cli_unsigned_transaction.inner,
                None => SignTransactionWithLedger::input_unsigned_transaction(),
            };
        SignTransactionWithLedger {
            seed_phrase_hd_path,
            unsigned_transaction,
        }
    }
}

impl SignTransactionWithLedger {
    pub fn input_unsigned_transaction() -> near_primitives::transaction::Transaction {
        let input: crate::common::TransactionAsBase64 = Input::new()
            .with_prompt("Enter an unsigned transaction")
            .interact_text()
            .unwrap();
        input.inner
    }

    pub fn input_seed_phrase_hd_path() -> slip10::BIP32Path {
        Input::new()
            .with_prompt("Enter seed phrase HD Path (if you not sure leave blank for default)")
            .with_initial_text("44'/397'/0'/0'/1'")
            .interact_text()
            .unwrap()
    }

    pub async fn process(self) -> crate::CliResult {
        println!("\nGoing to sign transaction:");
        crate::common::print_transaction(self.unsigned_transaction.clone());
        println!(
            "Please confirm transaction signing on Ledger Device (HD Path {})",
            self.seed_phrase_hd_path.to_string()
        );
        let signature = match near_ledger::sign_transaction(
            self.unsigned_transaction
                .try_to_vec()
                .expect("Transaction is not expected to fail on serialization"),
            self.seed_phrase_hd_path,
        )
        .await
        {
            Ok(signature) => {
                near_crypto::Signature::from_parts(near_crypto::KeyType::ED25519, &signature)
                    .expect("Signature is not expected to fail on deserialization")
            }
            Err(near_ledger_error) => {
                return Err(color_eyre::Report::msg(format!(
                    "Error occurred while signing the transaction: {:?}",
                    near_ledger_error
                )));
            }
        };

        let signed_transaction = near_primitives::transaction::SignedTransaction::new(
            signature,
            self.unsigned_transaction,
        );

        println!("\nSigned transaction:\n");
        crate::common::print_transaction(signed_transaction.transaction.clone());
        println!("{:<13} {}", "signature:", signed_transaction.signature);

        let serialize_to_base64 = near_primitives::serialize::to_base64(
            signed_transaction
                .try_to_vec()
                .expect("Signed transaction is not expected to fail on serialization"),
        );
        println!(
            "Base64-encoded signed transaction:\n{}",
            serialize_to_base64
        );
        Ok(())
    }
}
