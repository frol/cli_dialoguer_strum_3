use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod offline_mode;
mod online_mode;

/// инструмент выбора режима online/offline
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliOperationMode {
    #[clap(subcommand)]
    mode: Option<CliMode>,
}

#[derive(Debug, Clone)]
pub struct OperationMode {
    pub mode: Mode,
}

impl CliOperationMode {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        self.mode
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default()
    }
}

impl From<OperationMode> for CliOperationMode {
    fn from(item: OperationMode) -> Self {
        Self {
            mode: Some(item.mode.into()),
        }
    }
}

impl OperationMode {
    pub fn from(item: CliOperationMode) -> color_eyre::eyre::Result<Self> {
        let mode = match item.mode {
            Some(cli_mode) => Mode::from(cli_mode)?,
            None => Mode::choose_mode()?,
        };
        Ok(Self { mode })
    }
}

impl OperationMode {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        self.mode.process(prepopulated_unsigned_transaction).await
    }
}

#[derive(Debug, Clone, clap::Clap)]
pub enum CliMode {
    /// Prepare and, optionally, submit a new transaction with online mode
    Network(self::online_mode::CliNetworkArgs),
    /// Prepare and, optionally, submit a new transaction with offline mode
    Offline(self::offline_mode::CliOfflineArgs),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum Mode {
    #[strum_discriminants(strum(message = "Yes, I keep it simple"))]
    Network(self::online_mode::NetworkArgs),
    #[strum_discriminants(strum(
        message = "No, I want to work in no-network (air-gapped) environment"
    ))]
    Offline(self::offline_mode::OfflineArgs),
}

impl CliMode {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::Network(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("network".to_owned());
                args
            }
            Self::Offline(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("offline".to_owned());
                args
            }
        }
    }
}

impl From<Mode> for CliMode {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Network(network_args) => {
                Self::Network(self::online_mode::CliNetworkArgs::from(network_args))
            }
            Mode::Offline(offline_args) => {
                Self::Offline(self::offline_mode::CliOfflineArgs::from(offline_args))
            }
        }
    }
}

impl Mode {
    fn from(item: CliMode) -> color_eyre::eyre::Result<Self> {
        match item {
            CliMode::Network(cli_network_args) => Ok(Self::Network(
                self::online_mode::NetworkArgs::from(cli_network_args)?,
            )),
            CliMode::Offline(cli_offline_args) => Ok(Self::Offline(
                self::offline_mode::OfflineArgs::from(cli_offline_args)?,
            )),
        }
    }
}

impl Mode {
    fn choose_mode() -> color_eyre::eyre::Result<Self> {
        println!();
        let variants = ModeDiscriminants::iter().collect::<Vec<_>>();
        let modes = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selected_mode = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(
                "To construct a transaction you will need to provide information about sender (signer) and receiver accounts, and actions that needs to be performed.
                 \nDo you want to derive some information required for transaction construction automatically querying it online?"
            )
            .items(&modes)
            .default(0)
            .interact()
            .unwrap();
        let cli_mode = match variants[selected_mode] {
            ModeDiscriminants::Network => CliMode::Network(Default::default()),
            ModeDiscriminants::Offline => CliMode::Offline(Default::default()),
        };
        Ok(Self::from(cli_mode)?)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        match self {
            Self::Network(network_args) => {
                network_args
                    .process(prepopulated_unsigned_transaction)
                    .await
            }
            Self::Offline(offline_args) => {
                offline_args
                    .process(prepopulated_unsigned_transaction)
                    .await
            }
        }
    }
}
