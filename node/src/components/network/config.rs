use std::str::FromStr;

use datasize::DataSize;
use libp2p::request_response::RequestResponseConfig;
use serde::{Deserialize, Serialize};

use crate::{components::small_network, types::TimeDiff};

// TODO - remove these defaults once small_network's config has been replaced by this one.
mod temp {
    pub(super) const CONNECTION_SETUP_TIMEOUT: &str = "10seconds";
    pub(super) const MAX_ONE_WAY_MESSAGE_SIZE: u32 = 1024 * 1024;
    pub(super) const REQUEST_TIMEOUT: &str = "10seconds";
    pub(super) const CONNECTION_KEEP_ALIVE: &str = "5minutes";
    pub(super) const GOSSIP_HEARTBEAT_INTERVAL: &str = "10seconds";
    // TODO - check 256kB is ok.
    pub(super) const GOSSIP_MAX_MESSAGE_SIZE: u32 = 256 * 1024;
    pub(super) const GOSSIP_DUPLICATE_CACHE_TIMEOUT: &str = "1minute";
}

const DEFAULT_BIND_ADDRESS: &str = "0.0.0.0:22777";

impl Default for Config {
    fn default() -> Self {
        Config {
            bind_address: DEFAULT_BIND_ADDRESS.to_string(),
            known_addresses: Vec::new(),
            systemd_support: false,
            connection_setup_timeout: TimeDiff::from_str(temp::CONNECTION_SETUP_TIMEOUT).unwrap(),
            max_one_way_message_size: temp::MAX_ONE_WAY_MESSAGE_SIZE,
            request_timeout: TimeDiff::from_str(temp::REQUEST_TIMEOUT).unwrap(),
            connection_keep_alive: TimeDiff::from_str(temp::CONNECTION_KEEP_ALIVE).unwrap(),
            gossip_heartbeat_interval: TimeDiff::from_str(temp::GOSSIP_HEARTBEAT_INTERVAL).unwrap(),
            gossip_max_message_size: temp::GOSSIP_MAX_MESSAGE_SIZE,
            gossip_duplicate_cache_timeout: TimeDiff::from_str(
                temp::GOSSIP_DUPLICATE_CACHE_TIMEOUT,
            )
            .unwrap(),
        }
    }
}

/// Peer-to-peer network configuration.
#[derive(DataSize, Debug, Clone, Deserialize, Serialize)]
// Disallow unknown fields to ensure config files and command-line overrides contain valid keys.
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Address to bind to.
    pub bind_address: String,
    /// Known address of a node on the network used for joining.
    pub known_addresses: Vec<String>,
    /// Enable systemd startup notification.
    pub systemd_support: bool,
    /// The timeout for connection setup (including upgrades) for all inbound and outbound
    /// connections.
    pub connection_setup_timeout: TimeDiff,
    /// The maximum serialized one-way message size in bytes.
    pub max_one_way_message_size: u32,
    /// The timeout for inbound and outbound requests.
    pub request_timeout: TimeDiff,
    /// The keep-alive timeout of idle connections.
    pub connection_keep_alive: TimeDiff,
    /// Interval used for gossip heartbeats.
    pub gossip_heartbeat_interval: TimeDiff,
    /// Maximum serialized gossip message size in bytes.
    pub gossip_max_message_size: u32,
    /// Time for which to retain a cached gossip message ID to prevent duplicates being gossiped.
    pub gossip_duplicate_cache_timeout: TimeDiff,
}

impl From<&small_network::Config> for Config {
    fn from(config: &small_network::Config) -> Self {
        Config {
            bind_address: config.bind_address.clone(),
            known_addresses: config.known_addresses.clone(),
            systemd_support: config.systemd_support,
            ..Default::default()
        }
    }
}

impl From<&Config> for RequestResponseConfig {
    fn from(config: &Config) -> Self {
        let mut request_response_config = RequestResponseConfig::default();
        request_response_config.set_request_timeout(config.request_timeout.into());
        request_response_config.set_connection_keep_alive(config.connection_keep_alive.into());
        request_response_config
    }
}
