// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::errors::Result;
use crate::{types::NamedMessage, Actor};
use communication_refactored::firewall::FirewallConfiguration;
use communication_refactored::InitKeypair;
use communication_refactored::{ReceiveRequest, ShCommunicationBuilder};
use dashmap::DashMap;
use futures::{channel::mpsc, AsyncRead, AsyncWrite};
use libp2p::{core::Transport, Multiaddr};

pub struct ActorBuilder {
  receiver: mpsc::Receiver<ReceiveRequest<NamedMessage, NamedMessage>>,
  comm_builder: ShCommunicationBuilder<NamedMessage, NamedMessage, NamedMessage>,
  listening_addresses: Vec<Multiaddr>,
}

impl ActorBuilder {
  pub fn new() -> Self {
    let (sender, receiver) = mpsc::channel(512);
    let (firewall_sender, _) = mpsc::channel(512);
    let comm_builder = ShCommunicationBuilder::new(firewall_sender, sender, None)
      .with_firewall_config(FirewallConfiguration::allow_all());
    Self {
      receiver,
      comm_builder,
      listening_addresses: vec![],
    }
  }

  #[cfg(feature = "tcp")]
  pub async fn build(self) -> Result<Actor> {
    let comm = self.comm_builder.build().await?;
    let handlers = DashMap::new();
    let objects = DashMap::new();
    Actor::from_builder(self.receiver, comm, handlers, objects, self.listening_addresses).await
  }

  pub async fn build_with_transport<TRA>(self, transport: TRA) -> Result<Actor>
  where
    TRA: Transport + Sized + Clone + Send + Sync + 'static,
    TRA::Output: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    TRA::Dial: Send + 'static,
    TRA::Listener: Send + 'static,
    TRA::ListenerUpgrade: Send + 'static,
    TRA::Error: Send + Sync,
  {
    let comm = self.comm_builder.build_with_transport(transport).await;
    let handlers = DashMap::new();
    let objects = DashMap::new();
    Actor::from_builder(self.receiver, comm, handlers, objects, self.listening_addresses).await
  }

  pub fn keys(mut self, keys: InitKeypair) -> Self {
    self.comm_builder = self.comm_builder.with_keys(keys);
    self
  }

  pub fn listen_on(mut self, address: Multiaddr) -> Self {
    self.listening_addresses.push(address);
    self
  }
}
