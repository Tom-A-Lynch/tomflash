use ethers::{
    prelude::*,
    providers::{Http, Provider},
    signers::LocalWallet,
    types::{Address, TransactionRequest, U256},
};
use std::str::FromStr;
use tracing::{debug, info, warn};

use crate::{
    config::Config,
    utils::{Result, UtilError},
    utils::traits::Scorable,
};

pub struct Client {
    provider: Provider<Http>,
    wallet: LocalWallet,
    chain_id: u64,
}

impl Client {
    pub fn new(config: &Config) -> Result<Self> {
        let provider = Provider::<Http>::try_from(&config.eth_rpc_url)
            .map_err(|e| UtilError::ConversionError(e.to_string()))?;

        let wallet = LocalWallet::from_str(&config.eth_private_key)
            .map_err(|e| UtilError::ConversionError(e.to_string()))?
            .with_chain_id(config.eth_chain_id);

        Ok(Self {
            provider,
            wallet,
            chain_id: config.eth_chain_id,
        })
    }

    pub async fn get_balance(&self) -> Result<U256> {
        let address = self.wallet.address();
        let balance = self.provider.get_balance(address, None)
            .await
            .map_err(|e| UtilError::ConversionError(e.to_string()))?;
        
        debug!("Wallet balance: {} ETH", ethers::utils::format_ether(balance));
        Ok(balance)
    }

    pub async fn transfer_eth(&self, to_address: &str, amount_eth: f64) -> Result<H256> {
        let to_address = Address::from_str(to_address)
            .map_err(|e| UtilError::ConversionError(e.to_string()))?;

        let amount = ethers::utils::parse_ether(amount_eth)
            .map_err(|e| UtilError::ConversionError(e.to_string()))?;

        let tx = TransactionRequest::new()
            .to(to_address)
            .value(amount)
            .from(self.wallet.address());

        info!("Initiating transfer of {} ETH to {}", amount_eth, to_address);

        let tx_hash = self.provider
            .send_transaction(tx, None)
            .await
            .map_err(|e| UtilError::ConversionError(e.to_string()))?
            .tx_hash();

        debug!("Transaction hash: {}", tx_hash);
        Ok(tx_hash)
    }

    pub fn wallet_address(&self) -> String {
        self.wallet.address().to_string()
    }
}

pub async fn wallet_address_in_post(content: &str) -> Option<String> {
    // Basic ETH address regex
    let re = regex::Regex::new(r"0x[a-fA-F0-9]{40}").ok()?;
    re.find(content).map(|m| m.as_str().to_string())
}

#[async_trait::async_trait]
impl Scorable for TransactionRequest {
    async fn calculate_significance(&self) -> Result<f32> {
        let value = self.value.unwrap_or_default();
        let eth_value = ethers::utils::format_ether(value);
        let eth_float: f64 = eth_value.parse()
            .map_err(|e| UtilError::ConversionError(e.to_string()))?;

        // Score based on transaction value
        // 0.1 ETH or less: 0.1-0.3
        // 0.1-1 ETH: 0.3-0.6
        // 1+ ETH: 0.6-1.0
        let score = match eth_float {
            x if x <= 0.1 => 0.1 + (x * 2.0),
            x if x <= 1.0 => 0.3 + (x * 0.3),
            x => (0.6 + (x * 0.1)).min(1.0),
        };

        Ok(score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::types::U256;

    #[tokio::test]
    async fn test_wallet_address_detection() {
        let content = "Send ETH to 0x742d35Cc6634C0532925a3b844Bc454e4438f44e please";
        let address = wallet_address_in_post(content).await;
        assert_eq!(address, Some("0x742d35Cc6634C0532925a3b844Bc454e4438f44e".to_string()));
    }

    #[tokio::test]
    async fn test_transaction_significance() {
        let mut tx = TransactionRequest::new();
        
        // Test small amount
        tx.value = Some(ethers::utils::parse_ether(0.05f64).unwrap());
        let score = tx.calculate_significance().await.unwrap();
        assert!(score < 0.3);

        // Test medium amount
        tx.value = Some(ethers::utils::parse_ether(0.5f64).unwrap());
        let score = tx.calculate_significance().await.unwrap();
        assert!(score > 0.3 && score < 0.6);

        // Test large amount
        tx.value = Some(ethers::utils::parse_ether(2.0f64).unwrap());
        let score = tx.calculate_significance().await.unwrap();
        assert!(score > 0.6);
    }
}