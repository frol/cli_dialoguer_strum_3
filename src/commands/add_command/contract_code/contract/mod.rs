use dialoguer::{theme::ColorfulTheme, Input, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod initialize_mode;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliContract {
    /// Add a contract file
    ContractFile(CliContractFile),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum Contract {
    #[strum_discriminants(strum(message = "Add a contract file"))]
    ContractFile(ContractFile),
}

impl CliContract {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::ContractFile(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("contract-file".to_owned());
                args
            }
        }
    }
}

impl From<Contract> for CliContract {
    fn from(contract: Contract) -> Self {
        match contract {
            Contract::ContractFile(contract_file) => Self::ContractFile(contract_file.into()),
        }
    }
}

impl Contract {
    pub fn from(
        item: CliContract,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        match item {
            CliContract::ContractFile(cli_contract_file) => Ok(Contract::ContractFile(
                ContractFile::from(cli_contract_file, connection_config, sender_account_id)?,
            )),
        }
    }
}

impl Contract {
    pub fn choose_contract(
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        println!();
        let variants = ContractDiscriminants::iter().collect::<Vec<_>>();
        let contracts = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selected_contract = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("To deploy contract code you will need to choose next action")
            .items(&contracts)
            .default(0)
            .interact()
            .unwrap();
        let cli_contract = match variants[selected_contract] {
            ContractDiscriminants::ContractFile => CliContract::ContractFile(Default::default()),
        };
        Ok(Self::from(
            cli_contract,
            connection_config,
            sender_account_id,
        )?)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        match self {
            Contract::ContractFile(contract_file) => {
                contract_file
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}

/// add contract file
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliContractFile {
    file_path: Option<std::path::PathBuf>,
    #[clap(subcommand)]
    next_action: Option<self::initialize_mode::CliNextAction>,
}

#[derive(Debug, Clone)]
pub struct ContractFile {
    pub file_path: std::path::PathBuf,
    next_action: self::initialize_mode::NextAction,
}

impl CliContractFile {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .next_action
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        if let Some(file_path) = &self.file_path {
            args.push_front(file_path.as_path().display().to_string());
        }
        args
    }
}

impl From<ContractFile> for CliContractFile {
    fn from(contract_file: ContractFile) -> Self {
        Self {
            file_path: Some(contract_file.file_path),
            next_action: Some(contract_file.next_action.into()),
        }
    }
}

impl ContractFile {
    fn from(
        item: CliContractFile,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        let file_path = match item.file_path {
            Some(cli_file_path) => cli_file_path,
            None => ContractFile::input_file_path(),
        };
        let next_action = match item.next_action {
            Some(cli_next_action) => self::initialize_mode::NextAction::from(
                cli_next_action,
                connection_config,
                sender_account_id,
            )?,
            None => self::initialize_mode::NextAction::choose_next_action(
                connection_config,
                sender_account_id,
            )?,
        };
        Ok(ContractFile {
            file_path,
            next_action,
        })
    }
}

impl ContractFile {
    fn input_file_path() -> std::path::PathBuf {
        println!();
        let input_file_path: String = Input::new()
            .with_prompt("What is a file location of the contract?")
            .interact_text()
            .unwrap();
        input_file_path.into()
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let code = std::fs::read(&self.file_path.clone())
            .map_err(|err| color_eyre::Report::msg(format!("Failed to open file: {:?}", err)))?;
        let action = near_primitives::transaction::Action::DeployContract(
            near_primitives::transaction::DeployContractAction { code },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        self.next_action
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}
