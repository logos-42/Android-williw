use ethers::signers::{LocalWallet, Signer};
use ethers::types::Address;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Invalid wallet address")]
    InvalidAddress,
    #[error("Signature verification failed")]
    VerificationFailed,
    #[error("Signature error: {0}")]
    SignatureError(String),
}

pub fn verify_wallet_signature(
    wallet_address: &str,
    signature: &str,
    message: &str,
) -> Result<bool, WalletError> {
    let address: Address = wallet_address
        .parse()
        .map_err(|_| WalletError::InvalidAddress)?;

    let wallet = LocalWallet::from_address(address);

    let recovered = wallet
        .sign_message(message.as_bytes())
        .map_err(|e| WalletError::SignatureError(e.to_string()))?;

    let sig_bytes = hex::decode(signature.trim_start_matches("0x"))
        .map_err(|_| WalletError::SignatureError("Invalid hex".to_string()))?;

    if sig_bytes.len() != 65 {
        return Err(WalletError::SignatureError("Invalid signature length".to_string()));
    }

    Ok(recovered.to_bytes().to_vec() == sig_bytes)
}

pub fn is_valid_eth_address(address: &str) -> bool {
    address.starts_with("0x") && address.len() == 42
}
