use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod public_key_mode;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliFullAccessKey {
    /// Specify a full access key for the sub-account
    SubAccountFullAccess(CliSubAccountFullAccess),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum FullAccessKey {
    #[strum_discriminants(strum(message = "Add a full access key for the sub-account"))]
    SubAccountFullAccess(SubAccountFullAccess),
}

impl CliFullAccessKey {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::SubAccountFullAccess(subcommand) => {
                let mut command = subcommand.to_cli_args();
                command.push_front("sub-account-full-access".to_owned());
                command
            }
        }
    }
}

impl From<FullAccessKey> for CliFullAccessKey {
    fn from(full_access_key: FullAccessKey) -> Self {
        match full_access_key {
            FullAccessKey::SubAccountFullAccess(sub_account_full_access) => {
                Self::SubAccountFullAccess(sub_account_full_access.into())
            }
        }
    }
}

impl FullAccessKey {
    pub fn from(
        item: CliFullAccessKey,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        match item {
            CliFullAccessKey::SubAccountFullAccess(cli_sub_account_full_access) => Ok(
                FullAccessKey::SubAccountFullAccess(SubAccountFullAccess::from(
                    cli_sub_account_full_access,
                    connection_config,
                    sender_account_id,
                )?),
            ),
        }
    }
}

impl FullAccessKey {
    pub fn choose_full_access_key(
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        println!();
        let variants = FullAccessKeyDiscriminants::iter().collect::<Vec<_>>();
        let actions = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selected_action = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Сhoose what you want to add")
            .items(&actions)
            .default(0)
            .interact()
            .unwrap();
        let cli_action = match variants[selected_action] {
            FullAccessKeyDiscriminants::SubAccountFullAccess => {
                CliFullAccessKey::SubAccountFullAccess(Default::default())
            }
        };
        Ok(Self::from(
            cli_action,
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
            FullAccessKey::SubAccountFullAccess(sub_account_full_access) => {
                sub_account_full_access
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}

/// данные о ключе доступа
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliSubAccountFullAccess {
    #[clap(subcommand)]
    public_key_mode: Option<self::public_key_mode::CliPublicKeyMode>,
}

#[derive(Debug, Clone)]
pub struct SubAccountFullAccess {
    pub public_key_mode: self::public_key_mode::PublicKeyMode,
}

impl CliSubAccountFullAccess {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let args = self
            .public_key_mode
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        args
    }
}

impl From<SubAccountFullAccess> for CliSubAccountFullAccess {
    fn from(sub_account_full_access: SubAccountFullAccess) -> Self {
        Self {
            public_key_mode: Some(sub_account_full_access.public_key_mode.into()),
        }
    }
}

impl SubAccountFullAccess {
    fn from(
        item: CliSubAccountFullAccess,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        let public_key_mode = match item.public_key_mode {
            Some(cli_public_key_mode) => self::public_key_mode::PublicKeyMode::from(
                cli_public_key_mode,
                connection_config,
                sender_account_id,
            )?,
            None => self::public_key_mode::PublicKeyMode::choose_public_key_mode(
                connection_config,
                sender_account_id,
            )?,
        };
        Ok(Self { public_key_mode })
    }
}

impl SubAccountFullAccess {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        self.public_key_mode
            .process(prepopulated_unsigned_transaction, network_connection_config)
            .await
    }
}
