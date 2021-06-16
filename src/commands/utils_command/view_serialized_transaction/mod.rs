use dialoguer::Input;
use near_primitives::borsh::BorshDeserialize;

#[derive(Debug, Clone)]
pub enum SignedOrNonsignedTransactionAsBase64 {
    Transaction(crate::common::TransactionAsBase64),
    SignedTransaction(crate::common::SignedTransactionAsBase64),
}

impl std::str::FromStr for SignedOrNonsignedTransactionAsBase64 {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(signed_transaction) = s.parse() {
            Ok(Self::SignedTransaction(signed_transaction))
        } else {
            Ok(Self::Transaction(s.parse()?))
        }
    }
}

impl std::fmt::Display for SignedOrNonsignedTransactionAsBase64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignedOrNonsignedTransactionAsBase64::Transaction(t) => t.fmt(f),
            SignedOrNonsignedTransactionAsBase64::SignedTransaction(t) => t.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum SignedOrNonsignedTransaction {
    Transaction(near_primitives::transaction::Transaction),
    SignedTransaction(near_primitives::transaction::SignedTransaction),
}

impl std::convert::From<SignedOrNonsignedTransactionAsBase64> for SignedOrNonsignedTransaction {
    fn from(value: SignedOrNonsignedTransactionAsBase64) -> Self {
        match value {
            SignedOrNonsignedTransactionAsBase64::Transaction(t) => Self::Transaction(t.inner),
            SignedOrNonsignedTransactionAsBase64::SignedTransaction(t) => {
                Self::SignedTransaction(t.inner)
            }
        }
    }
}

/// Using this utility, you can view the contents of a serialized transaction (signed or not).
#[derive(Debug, Default, clap::Clap)]
pub struct CliViewSerializedTransaction {
    transaction: Option<SignedOrNonsignedTransactionAsBase64>,
}

#[derive(Debug)]
pub struct ViewSerializedTransaction {
    transaction: SignedOrNonsignedTransaction,
}

impl From<CliViewSerializedTransaction> for ViewSerializedTransaction {
    fn from(item: CliViewSerializedTransaction) -> Self {
        let transaction = match item.transaction {
            Some(transaction) => transaction.into(),
            None => ViewSerializedTransaction::input_transaction(),
        };
        Self { transaction }
    }
}

impl ViewSerializedTransaction {
    fn input_transaction() -> SignedOrNonsignedTransaction {
        let transaction: SignedOrNonsignedTransactionAsBase64 = Input::new()
            .with_prompt("Enter the hash of the transaction")
            .interact_text()
            .unwrap();
        transaction.into()
    }

    pub async fn process(self) -> crate::CliResult {
        match self.transaction {
            SignedOrNonsignedTransaction::Transaction(transaction) => {
                println!("{:#?}", transaction)
            }
            SignedOrNonsignedTransaction::SignedTransaction(transaction) => {
                println!("{:#?}", transaction)
            }
        }
        Ok(())
    }
}
