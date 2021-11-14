use async_recursion::async_recursion;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod full_access_type;
mod function_call_type;

/// добавление ключа пользователю
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliAddAccessKeyAction {
    public_key: Option<near_crypto::PublicKey>,
    nonce: Option<u64>,
    #[clap(subcommand)]
    permission: Option<CliAccessKeyPermission>,
}

#[derive(Debug, Clone)]
pub struct AddAccessKeyAction {
    pub public_key: near_crypto::PublicKey,
    pub nonce: near_primitives::types::Nonce,
    pub permission: AccessKeyPermission,
}

impl CliAddAccessKeyAction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .permission
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        if let Some(public_key) = &self.public_key {
            args.push_front(public_key.to_string());
        };
        args
    }
}

impl From<AddAccessKeyAction> for CliAddAccessKeyAction {
    fn from(add_access_key_action: AddAccessKeyAction) -> Self {
        Self {
            public_key: Some(add_access_key_action.public_key),
            nonce: Some(add_access_key_action.nonce),
            permission: Some(add_access_key_action.permission.into()),
        }
    }
}

impl AddAccessKeyAction {
    pub fn from(
        item: CliAddAccessKeyAction,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        let public_key: near_crypto::PublicKey = match item.public_key {
            Some(cli_public_key) => cli_public_key,
            None => AddAccessKeyAction::input_public_key(),
        };
        let nonce: near_primitives::types::Nonce = match item.nonce {
            Some(cli_nonce) => near_primitives::types::Nonce::from(cli_nonce),
            None => AddAccessKeyAction::input_access_key_nonce(),
        };
        let permission: AccessKeyPermission = match item.permission {
            Some(cli_permission) => {
                AccessKeyPermission::from(cli_permission, connection_config, sender_account_id)?
            }
            None => AccessKeyPermission::choose_permission(connection_config, sender_account_id)?,
        };
        Ok(Self {
            public_key,
            nonce,
            permission,
        })
    }
}

impl AddAccessKeyAction {
    fn input_access_key_nonce() -> near_primitives::types::Nonce {
        Input::new()
            .with_prompt("Enter the nonce for this access key")
            .interact_text()
            .unwrap()
    }

    fn input_public_key() -> near_crypto::PublicKey {
        Input::new()
            .with_prompt("Enter a public key for this access key")
            .interact_text()
            .unwrap()
    }

    #[async_recursion(?Send)]
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        match self.permission {
            AccessKeyPermission::GrantFullAccess(full_access_type) => {
                full_access_type
                    .process(
                        self.nonce,
                        prepopulated_unsigned_transaction,
                        network_connection_config,
                        self.public_key,
                    )
                    .await
            }
            AccessKeyPermission::GrantFunctionCallAccess(function_call_type) => {
                function_call_type
                    .process(
                        self.nonce,
                        prepopulated_unsigned_transaction,
                        network_connection_config,
                        self.public_key,
                    )
                    .await
            }
        }
    }
}

#[derive(Debug, Clone, clap::Clap)]
pub enum CliAccessKeyPermission {
    /// Предоставьте данные для ключа с function call
    GrantFunctionCallAccess(self::function_call_type::CliFunctionCallType),
    /// Предоставьте данные для ключа с полным доступом
    GrantFullAccess(self::full_access_type::CliFullAccessType),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum AccessKeyPermission {
    #[strum_discriminants(strum(message = "A permission with function call"))]
    GrantFunctionCallAccess(self::function_call_type::FunctionCallType),
    #[strum_discriminants(strum(message = "A permission with full access"))]
    GrantFullAccess(self::full_access_type::FullAccessType),
}

impl CliAccessKeyPermission {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::GrantFunctionCallAccess(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("grant-function-call-access".to_owned());
                args
            }
            Self::GrantFullAccess(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("grant-full-access".to_owned());
                args
            }
        }
    }
}

impl From<AccessKeyPermission> for CliAccessKeyPermission {
    fn from(access_key_permission: AccessKeyPermission) -> Self {
        match access_key_permission {
            AccessKeyPermission::GrantFunctionCallAccess(function_call_type) => {
                Self::GrantFunctionCallAccess(function_call_type.into())
            }
            AccessKeyPermission::GrantFullAccess(full_access_type) => {
                Self::GrantFullAccess(full_access_type.into())
            }
        }
    }
}

impl AccessKeyPermission {
    pub fn from(
        item: CliAccessKeyPermission,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        match item {
            CliAccessKeyPermission::GrantFunctionCallAccess(cli_function_call_type) => {
                let function_call_type = self::function_call_type::FunctionCallType::from(
                    cli_function_call_type,
                    connection_config,
                    sender_account_id,
                )?;
                Ok(AccessKeyPermission::GrantFunctionCallAccess(
                    function_call_type,
                ))
            }
            CliAccessKeyPermission::GrantFullAccess(cli_full_access_type) => {
                let full_access_type = self::full_access_type::FullAccessType::from(
                    cli_full_access_type,
                    connection_config,
                    sender_account_id,
                )?;
                Ok(AccessKeyPermission::GrantFullAccess(full_access_type))
            }
        }
    }
}

impl AccessKeyPermission {
    pub fn choose_permission(
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        let variants = AccessKeyPermissionDiscriminants::iter().collect::<Vec<_>>();
        let permissions = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let select_permission = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a permission that you want to add to the access key:")
            .items(&permissions)
            .default(0)
            .interact()
            .unwrap();
        match variants[select_permission] {
            AccessKeyPermissionDiscriminants::GrantFunctionCallAccess => Ok(Self::from(
                CliAccessKeyPermission::GrantFunctionCallAccess(Default::default()),
                connection_config,
                sender_account_id,
            )?),
            AccessKeyPermissionDiscriminants::GrantFullAccess => Ok(Self::from(
                CliAccessKeyPermission::GrantFullAccess(Default::default()),
                connection_config,
                sender_account_id,
            )?),
        }
    }
}
