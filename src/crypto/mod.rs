use anyhow::Result;
use async_trait::async_trait;
use downcast_rs::{impl_downcast, Downcast};
use dyn_clone::DynClone;
use ed25519_dalek::PublicKey;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub use derived_key::*;
pub use encrypted_key::*;
pub use ledger_key::*;
pub use mnemonic::*;

use crate::utils::*;

mod derived_key;
mod encrypted_key;
mod ledger_key;
mod mnemonic;
mod symmetric;

pub type Signature = [u8; ed25519_dalek::SIGNATURE_LENGTH];

pub trait UnsignedMessage: DynClone + Send {
    /// Adjust expiration timestamp from now
    fn refresh_timeout(&mut self);

    /// Current expiration timestamp
    fn expire_at(&self) -> u32;

    /// Message body hash
    fn hash(&self) -> &[u8];

    /// Create signed message from prepared inputs
    /// # Arguments
    /// `signature` - signature, received from [`hash`]
    fn sign(&self, signature: &Signature) -> Result<SignedMessage>;
}

dyn_clone::clone_trait_object!(UnsignedMessage);

#[derive(Clone, Debug)]
pub struct SignedMessage {
    pub message: ton_block::Message,
    pub expire_at: u32,
}

#[async_trait]
pub trait Signer: SignerStorage {
    type CreateKeyInput: Serialize + DeserializeOwned;
    type ExportKeyInput: Serialize + DeserializeOwned;
    type ExportKeyOutput: Serialize + DeserializeOwned;
    type GetPublicKeys: Serialize + DeserializeOwned;
    type UpdateKeyInput: Serialize + DeserializeOwned;
    type SignInput: Serialize + DeserializeOwned;

    async fn add_key(&mut self, input: Self::CreateKeyInput) -> Result<SignerEntry>;
    async fn update_key(&mut self, input: Self::UpdateKeyInput) -> Result<SignerEntry>;
    async fn export_key(&self, input: Self::ExportKeyInput) -> Result<Self::ExportKeyOutput>;
    async fn get_public_keys(&self, input: Self::GetPublicKeys) -> Result<Vec<PublicKey>>;

    async fn sign(&self, data: &[u8], input: Self::SignInput) -> Result<Signature>;
}

#[async_trait]
pub trait SignerStorage: Downcast + Send + Sync {
    fn load_state(&mut self, data: &str) -> Result<()>;
    fn store_state(&self) -> String;

    fn get_entries(&self) -> Vec<SignerEntry>;
    async fn remove_key(&mut self, public_key: &PublicKey) -> Option<SignerEntry>;
    async fn clear(&mut self);
}

impl_downcast!(SignerStorage);

pub trait WithPublicKey {
    fn public_key(&self) -> &PublicKey;
}

#[derive(Clone, Serialize, Deserialize, Debug, Copy)]
pub struct SignerEntry {
    #[serde(with = "serde_public_key")]
    pub public_key: PublicKey,
    #[serde(with = "serde_public_key")]
    pub master_key: PublicKey,
    pub account_id: u16,
}
