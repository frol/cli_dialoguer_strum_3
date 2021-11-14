use dialoguer::{theme::ColorfulTheme, Select};
use std::io::Write;
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod block_id_hash;
mod block_id_height;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliBlockId {
    /// Specify a block ID final to view this contract
    AtFinalBlock,
    /// Specify a block ID height to view this contract
    AtBlockHeight(self::block_id_height::CliBlockIdHeight),
    /// Specify a block ID hash to view this contract
    AtBlockHash(self::block_id_hash::CliBlockIdHash),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum BlockId {
    #[strum_discriminants(strum(message = "View this contract at final block"))]
    AtFinalBlock,
    #[strum_discriminants(strum(message = "View this contract at block heigt"))]
    AtBlockHeight(self::block_id_height::BlockIdHeight),
    #[strum_discriminants(strum(message = "View this contract at block hash"))]
    AtBlockHash(self::block_id_hash::BlockIdHash),
}

impl CliBlockId {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::AtFinalBlock => {
                let mut args = std::collections::VecDeque::new();
                args.push_front("at-final-block".to_owned());
                args
            }
            Self::AtBlockHeight(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("at-block-height".to_owned());
                args
            }
            Self::AtBlockHash(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("at-block-hash".to_owned());
                args
            }
        }
    }
}

impl From<BlockId> for CliBlockId {
    fn from(block_id: BlockId) -> Self {
        match block_id {
            BlockId::AtFinalBlock => Self::AtFinalBlock,
            BlockId::AtBlockHeight(block_id_height) => Self::AtBlockHeight(block_id_height.into()),
            BlockId::AtBlockHash(block_id_hash) => Self::AtBlockHash(block_id_hash.into()),
        }
    }
}

impl From<CliBlockId> for BlockId {
    fn from(item: CliBlockId) -> Self {
        match item {
            CliBlockId::AtFinalBlock => Self::AtFinalBlock,
            CliBlockId::AtBlockHeight(cli_block_id_height) => {
                Self::AtBlockHeight(cli_block_id_height.into())
            }
            CliBlockId::AtBlockHash(cli_block_id_hash) => {
                Self::AtBlockHash(cli_block_id_hash.into())
            }
        }
    }
}

impl BlockId {
    pub fn choose_block_id() -> Self {
        println!();
        let variants = BlockIdDiscriminants::iter().collect::<Vec<_>>();
        let blocks = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose your action")
            .items(&blocks)
            .default(0)
            .interact()
            .unwrap();
        let cli_block_id = match variants[selection] {
            BlockIdDiscriminants::AtFinalBlock => CliBlockId::AtFinalBlock,
            BlockIdDiscriminants::AtBlockHeight => CliBlockId::AtBlockHeight(Default::default()),
            BlockIdDiscriminants::AtBlockHash => CliBlockId::AtBlockHash(Default::default()),
        };
        Self::from(cli_block_id)
    }

    pub async fn process(
        self,
        contract_id: near_primitives::types::AccountId,
        network_connection_config: crate::common::ConnectionConfig,
        file_path: Option<std::path::PathBuf>,
    ) -> crate::CliResult {
        println!();
        match self {
            Self::AtBlockHeight(block_id_height) => {
                block_id_height
                    .process(contract_id, network_connection_config, file_path)
                    .await
            }
            Self::AtBlockHash(block_id_hash) => {
                block_id_hash
                    .process(contract_id, network_connection_config, file_path)
                    .await
            }
            Self::AtFinalBlock => {
                self.at_final_block(contract_id, network_connection_config, file_path)
                    .await
            }
        }
    }

    fn rpc_client(&self, selected_server_url: &str) -> near_jsonrpc_client::JsonRpcClient {
        near_jsonrpc_client::new_client(&selected_server_url)
    }

    async fn at_final_block(
        self,
        contract_id: near_primitives::types::AccountId,
        network_connection_config: crate::common::ConnectionConfig,
        file_path: Option<std::path::PathBuf>,
    ) -> crate::CliResult {
        let query_view_method_response = self
            .rpc_client(network_connection_config.rpc_url().as_str())
            .query(near_jsonrpc_primitives::types::query::RpcQueryRequest {
                block_reference: near_primitives::types::Finality::Final.into(),
                request: near_primitives::views::QueryRequest::ViewCode {
                    account_id: contract_id,
                },
            })
            .await
            .map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Failed to fetch query for view contract: {:?}",
                    err
                ))
            })?;
        let call_access_view =
            if let near_jsonrpc_primitives::types::query::QueryResponseKind::ViewCode(result) =
                query_view_method_response.kind
            {
                result
            } else {
                return Err(color_eyre::Report::msg(format!("Error call result")));
            };
        match &file_path {
            Some(file_path) => {
                let dir_name = &file_path.parent().unwrap();
                std::fs::create_dir_all(&dir_name)?;
                std::fs::File::create(file_path)
                    .map_err(|err| {
                        color_eyre::Report::msg(format!("Failed to create file: {:?}", err))
                    })?
                    .write(&call_access_view.code)
                    .map_err(|err| {
                        color_eyre::Report::msg(format!("Failed to write to file: {:?}", err))
                    })?;
                println!("\nThe file {:?} was downloaded successfully", file_path);
            }
            None => {
                println!("\nHash of the contract: {}", &call_access_view.hash)
            }
        }
        Ok(())
    }
}
