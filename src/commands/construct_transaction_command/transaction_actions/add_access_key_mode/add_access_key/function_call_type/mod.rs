use async_recursion::async_recursion;
use dialoguer::{console::Term, theme::ColorfulTheme, Input, Select};
use std::vec;

/// данные для определения ключа с function call
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliFunctionCallType {
    #[clap(long)]
    allowance: Option<crate::common::NearBalance>,
    #[clap(long)]
    receiver_id: Option<near_primitives::types::AccountId>,
    #[clap(long)]
    method_names: Option<String>,
    #[clap(subcommand)]
    next_action: Option<super::super::super::CliSkipNextAction>,
}

#[derive(Debug, Clone)]
pub struct FunctionCallType {
    pub allowance: Option<near_primitives::types::Balance>,
    pub receiver_id: near_primitives::types::AccountId,
    pub method_names: Vec<String>,
    pub next_action: Box<super::super::super::NextAction>,
}

impl CliFunctionCallType {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .next_action
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        if let Some(method_names) = &self.method_names {
            args.push_front(method_names.to_string());
            args.push_front("--method-names".to_owned())
        };
        if let Some(allowance) = &self.allowance {
            args.push_front(allowance.to_string());
            args.push_front("--allowance".to_owned())
        };
        if let Some(receiver_id) = &self.receiver_id {
            args.push_front(receiver_id.to_string());
            args.push_front("--receiver-id".to_owned())
        };
        args
    }
}

impl From<FunctionCallType> for CliFunctionCallType {
    fn from(function_call_type: FunctionCallType) -> Self {
        Self {
            allowance: Some(crate::common::NearBalance::from_yoctonear(
                function_call_type.allowance.unwrap_or_default(),
            )),
            receiver_id: Some(function_call_type.receiver_id),
            method_names: Some(function_call_type.method_names.join(", ")),
            next_action: Some(super::super::super::CliSkipNextAction::Skip(
                super::super::super::CliSkipAction { sign_option: None },
            )),
        }
    }
}

impl FunctionCallType {
    pub fn from(
        item: CliFunctionCallType,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        let allowance: Option<near_primitives::types::Balance> = match item.allowance {
            Some(cli_allowance) => Some(cli_allowance.to_yoctonear()),
            None => FunctionCallType::input_allowance(),
        };
        let receiver_id: near_primitives::types::AccountId = match item.receiver_id {
            Some(cli_receiver_id) => near_primitives::types::AccountId::from(cli_receiver_id),
            None => FunctionCallType::input_receiver_id(),
        };
        let method_names: Vec<String> = match item.method_names {
            Some(cli_method_names) => {
                if cli_method_names.is_empty() {
                    vec![]
                } else {
                    cli_method_names
                        .split(',')
                        .map(String::from)
                        .collect::<Vec<String>>()
                }
            }
            None => FunctionCallType::input_method_names(),
        };
        let skip_next_action: super::super::super::NextAction = match item.next_action {
            Some(cli_skip_action) => super::super::super::NextAction::from_cli_skip_next_action(
                cli_skip_action,
                connection_config,
                sender_account_id,
            )?,
            None => super::super::super::NextAction::input_next_action(
                connection_config,
                sender_account_id,
            )?,
        };
        Ok(Self {
            allowance,
            receiver_id,
            method_names,
            next_action: Box::new(skip_next_action),
        })
    }
}

impl FunctionCallType {
    pub fn input_method_names() -> Vec<String> {
        println!();
        let choose_input = vec![
            "Yes, I want to input a list of method names that can be used",
            "No, I don't to input a list of method names that can be used",
        ];
        let select_choose_input = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Do You want to input a list of method names that can be used")
            .items(&choose_input)
            .default(0)
            .interact_on_opt(&Term::stderr())
            .unwrap();
        match select_choose_input {
            Some(0) => {
                let mut input_method_names: String = Input::new()
                    .with_prompt("Enter a list of method names that can be used. The access key only allows transactions with the function call of one of the given method names. Empty list means any method name can be used.")
                    .interact_text()
                    .unwrap();
                if input_method_names.contains("\"") {
                    input_method_names.clear()
                };
                if input_method_names.is_empty() {
                    vec![]
                } else {
                    input_method_names
                        .split(',')
                        .map(String::from)
                        .collect::<Vec<String>>()
                }
            }
            Some(1) => vec![],
            _ => unreachable!("Error"),
        }
    }

    pub fn input_allowance() -> Option<near_primitives::types::Balance> {
        println!();
        let choose_input = vec![
            "Yes, I want to input allowance for receiver ID",
            "No, I don't to input allowance for receiver ID",
        ];
        let select_choose_input = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Do You want to input an allowance for receiver ID")
            .items(&choose_input)
            .default(0)
            .interact_on_opt(&Term::stderr())
            .unwrap();
        match select_choose_input {
            Some(0) => {
                let allowance_near_balance: crate::common::NearBalance = Input::new()
                    .with_prompt("Enter an allowance which is a balance limit to use by this access key to pay for function call gas and transaction fees.")
                    .interact_text()
                    .unwrap();
                Some(allowance_near_balance.to_yoctonear())
            }
            Some(1) => None,
            _ => unreachable!("Error"),
        }
    }

    pub fn input_receiver_id() -> near_primitives::types::AccountId {
        println!();
        Input::new()
            .with_prompt("Enter a receiver to use by this access key to pay for function call gas and transaction fees.")
            .interact_text()
            .unwrap()
    }

    #[async_recursion(?Send)]
    pub async fn process(
        self,
        nonce: near_primitives::types::Nonce,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
        public_key: near_crypto::PublicKey,
    ) -> crate::CliResult {
        let access_key: near_primitives::account::AccessKey = near_primitives::account::AccessKey {
            nonce,
            permission: near_primitives::account::AccessKeyPermission::FunctionCall(
                near_primitives::account::FunctionCallPermission {
                    allowance: self.allowance.clone(),
                    receiver_id: self.receiver_id.to_string().clone(),
                    method_names: self.method_names.clone(),
                },
            ),
        };
        let action = near_primitives::transaction::Action::AddKey(
            near_primitives::transaction::AddKeyAction {
                public_key,
                access_key,
            },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        match *self.next_action {
            super::super::super::NextAction::AddAction(select_action) => {
                select_action
                    .process(unsigned_transaction, network_connection_config)
                    .await
            }
            super::super::super::NextAction::Skip(skip_action) => {
                skip_action
                    .process(unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}
