use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod access_key;
mod account;

/// инструмент выбора to delete action
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliDeleteAction {
    #[clap(subcommand)]
    action: Option<CliAction>,
}

#[derive(Debug, Clone)]
pub struct DeleteAction {
    pub action: Action,
}

impl CliDeleteAction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        self.action
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default()
    }
}

impl From<DeleteAction> for CliDeleteAction {
    fn from(delete_action: DeleteAction) -> Self {
        Self {
            action: Some(delete_action.action.into()),
        }
    }
}

impl DeleteAction {
    pub fn from(item: CliDeleteAction) -> color_eyre::eyre::Result<Self> {
        let action = match item.action {
            Some(cli_action) => Action::from(cli_action)?,
            None => Action::choose_action()?,
        };
        Ok(Self { action })
    }
}

impl DeleteAction {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        self.action.process(prepopulated_unsigned_transaction).await
    }
}

#[derive(Debug, Clone, clap::Clap)]
pub enum CliAction {
    /// Delete an access key for an account
    AccessKey(self::access_key::operation_mode::CliOperationMode),
    /// Delete this account
    Account(self::account::operation_mode::CliOperationMode),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum Action {
    #[strum_discriminants(strum(message = "Delete an access key for this account"))]
    AccessKey(self::access_key::operation_mode::OperationMode),
    #[strum_discriminants(strum(message = "Delete this account"))]
    Account(self::account::operation_mode::OperationMode),
}

impl CliAction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::AccessKey(subcommand) => {
                let mut command = subcommand.to_cli_args();
                command.push_front("access-key".to_owned());
                command
            }
            Self::Account(subcommand) => {
                let mut command = subcommand.to_cli_args();
                command.push_front("account".to_owned());
                command
            }
        }
    }
}

impl From<Action> for CliAction {
    fn from(action: Action) -> Self {
        match action {
            Action::AccessKey(operation_mode) => Self::AccessKey(operation_mode.into()),
            Action::Account(operation_mode) => Self::Account(operation_mode.into()),
        }
    }
}

impl Action {
    fn from(item: CliAction) -> color_eyre::eyre::Result<Self> {
        match item {
            CliAction::AccessKey(cli_operation_mode) => Ok(Action::AccessKey(
                self::access_key::operation_mode::OperationMode::from(cli_operation_mode).unwrap(),
            )),
            CliAction::Account(cli_operation_mode) => Ok(Action::Account(
                self::account::operation_mode::OperationMode::from(cli_operation_mode).unwrap(),
            )),
        }
    }
}

impl Action {
    fn choose_action() -> color_eyre::eyre::Result<Self> {
        println!();
        let variants = ActionDiscriminants::iter().collect::<Vec<_>>();
        let actions = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selected_action = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Сhoose what you want to delete")
            .items(&actions)
            .default(0)
            .interact()
            .unwrap();
        let cli_action = match variants[selected_action] {
            ActionDiscriminants::AccessKey => CliAction::AccessKey(Default::default()),
            ActionDiscriminants::Account => CliAction::Account(Default::default()),
        };
        Ok(Self::from(cli_action)?)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        match self {
            Action::AccessKey(operation_mode) => {
                operation_mode
                    .process(prepopulated_unsigned_transaction)
                    .await
            }
            Action::Account(operation_mode) => {
                operation_mode
                    .process(prepopulated_unsigned_transaction)
                    .await
            }
        }
    }
}
