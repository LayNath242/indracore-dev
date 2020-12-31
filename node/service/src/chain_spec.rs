// Copyright 2017-2020 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! Indracore chain configurations.

use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use babe_primitives::AuthorityId as BabeId;
use grandpa::AuthorityId as GrandpaId;
use hex_literal::hex;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_staking::Forcing;
use indracore::constants::currency::SELS;
use indracore_primitives::v1::{AccountId, AccountPublic, ValidatorId, AssignmentId, Balance};
use indracore_runtime as indracore;
use xelendra_runtime as xelendra;
use xelendra_runtime::constants::currency::SELS as XEL;
use sc_chain_spec::{ChainSpecExtension, ChainType};
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, Pair, Public};
use sp_runtime::{traits::IdentifyAccount, Perbill};
use telemetry::TelemetryEndpoints;

const INDRACORE_STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
const XELENDRA_STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
const DEFAULT_PROTOCOL_ID: &str = "sel";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: sc_client_api::ForkBlocks<indracore_primitives::v1::Block>,
	/// Known bad block hashes.
	pub bad_blocks: sc_client_api::BadBlocks<indracore_primitives::v1::Block>,
}

/// The `ChainSpec` parametrised for the indracore runtime.
pub type IndracoreChainSpec = service::GenericChainSpec<indracore::GenesisConfig, Extensions>;

/// The `ChainSpec` parametrized for the xelendra runtime.
pub type XelendraChainSpec = service::GenericChainSpec<XelendraGenesisExt, Extensions>;

/// Extension for the Xelendra genesis config to support a custom changes to the genesis state.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct XelendraGenesisExt {
	/// The runtime genesis config.
	runtime_genesis_config: xelendra::GenesisConfig,
	/// The session length in blocks.
	///
	/// If `None` is supplied, the default value is used.
	session_length_in_blocks: Option<u32>,
}

impl sp_runtime::BuildStorage for XelendraGenesisExt {
	fn assimilate_storage(
		&self,
		storage: &mut sp_core::storage::Storage,
	) -> Result<(), String> {
		sp_state_machine::BasicExternalities::execute_with_storage(storage, || {
			if let Some(length) = self.session_length_in_blocks.as_ref() {
				xelendra::constants::time::EpochDurationInBlocks::set(length);
			}
		});
		self.runtime_genesis_config.assimilate_storage(storage)
	}
}

pub fn indracore_config() -> Result<IndracoreChainSpec, String> {
	IndracoreChainSpec::from_json_bytes(&include_bytes!("../res/indracore.json")[..])
}

pub fn xelendra_config() -> Result<IndracoreChainSpec, String> {
	IndracoreChainSpec::from_json_bytes(&include_bytes!("../res/xelendra.json")[..])
}

fn indracore_session_keys(
	babe: BabeId,
	grandpa: GrandpaId,
	im_online: ImOnlineId,
	para_validator: ValidatorId,
	para_assignment: AssignmentId,
	authority_discovery: AuthorityDiscoveryId,
) -> indracore::SessionKeys {
	indracore::SessionKeys {
		babe,
		grandpa,
		im_online,
		para_validator,
		para_assignment,
		authority_discovery,
	}
}

fn xelendra_session_keys(
	babe: BabeId,
	grandpa: GrandpaId,
	im_online: ImOnlineId,
	para_validator: ValidatorId,
	para_assignment: AssignmentId,
	authority_discovery: AuthorityDiscoveryId
) -> xelendra_runtime::SessionKeys {
	xelendra_runtime::SessionKeys {
		babe,
		grandpa,
		im_online,
		para_validator,
		para_assignment,
		authority_discovery,
	}
}

fn indracore_staging_testnet_config_genesis(wasm_binary: &[u8]) -> indracore::GenesisConfig {
	// subkey inspect "$SECRET"
	let endowed_accounts = vec![];

	let initial_authorities: Vec<(
		AccountId,
		AccountId,
		BabeId,
		GrandpaId,
		ImOnlineId,
		ValidatorId,
		AssignmentId,
		AuthorityDiscoveryId,
	)> = vec![];

	const ENDOWMENT: Balance = 2u128.pow(32) * SELS;
	const STASH: Balance = 100 * SELS;

	indracore::GenesisConfig {
		frame_system: Some(indracore::SystemConfig {
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_balances: Some(indracore::BalancesConfig {
			balances: endowed_accounts
				.iter()
				.map(|k: &AccountId| (k.clone(), ENDOWMENT))
				.chain(initial_authorities.iter().map(|x| (x.0.clone(), STASH)))
				.collect(),
		}),
		pallet_indices: Some(indracore::IndicesConfig { indices: vec![] }),
		pallet_session: Some(indracore::SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						indracore_session_keys(
							x.2.clone(),
							x.3.clone(),
							x.4.clone(),
							x.5.clone(),
							x.6.clone(),
							x.7.clone(),
						),
					)
				})
				.collect::<Vec<_>>(),
		}),
		pallet_staking: Some(indracore::StakingConfig {
			validator_count: 50,
			minimum_validator_count: 4,
			stakers: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.1.clone(),
						STASH,
						indracore::StakerStatus::Validator,
					)
				})
				.collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			force_era: Forcing::ForceNone,
			slash_reward_fraction: Perbill::from_percent(10),
			..Default::default()
		}),
		pallet_elections_phragmen: Some(Default::default()),
		pallet_democracy: Some(Default::default()),
		pallet_collective_Instance1: Some(indracore::CouncilConfig {
			members: vec![],
			phantom: Default::default(),
		}),
		pallet_collective_Instance2: Some(indracore::TechnicalCommitteeConfig {
			members: vec![],
			phantom: Default::default(),
		}),
		pallet_membership_Instance1: Some(Default::default()),
		pallet_babe: Some(Default::default()),
		pallet_grandpa: Some(Default::default()),
		pallet_im_online: Some(Default::default()),
		pallet_authority_discovery: Some(indracore::AuthorityDiscoveryConfig { keys: vec![] }),
		pallet_vesting: Some(indracore::VestingConfig { vesting: vec![] }),
		pallet_treasury: Some(Default::default()),
		pallet_contracts: Some(indracore::ContractsConfig {
			current_schedule: pallet_contracts::Schedule {
				..Default::default()
			},
		}),
	}
}

fn xelendra_staging_testnet_config_genesis(wasm_binary: &[u8]) -> xelendra_runtime::GenesisConfig {
	// subkey inspect "$SECRET"
	let endowed_accounts = vec![
		// 5FeyRQmjtdHoPH56ASFW76AJEP1yaQC1K9aEMvJTF9nzt9S9
		hex!["9ed7705e3c7da027ba0583a22a3212042f7e715d3c168ba14f1424e2bc111d00"].into(),
	];

	// ./scripts/prepare-test-net.sh 8
	let initial_authorities: Vec<(
		AccountId,
		AccountId,
		BabeId,
		GrandpaId,
		ImOnlineId,
		ValidatorId,
		AssignmentId,
		AuthorityDiscoveryId
	)> = vec![];

	const ENDOWMENT: Balance = 1_000_000 * XEL;
	const STASH: Balance = 100 * XEL;

	xelendra_runtime::GenesisConfig {
		frame_system: Some(xelendra_runtime::SystemConfig {
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_balances: Some(xelendra_runtime::BalancesConfig {
			balances: endowed_accounts.iter()
				.map(|k: &AccountId| (k.clone(), ENDOWMENT))
				.chain(initial_authorities.iter().map(|x| (x.0.clone(), STASH)))
				.collect(),
		}),
		pallet_indices: Some(xelendra_runtime::IndicesConfig {
			indices: vec![],
		}),
		pallet_session: Some(xelendra_runtime::SessionConfig {
			keys: initial_authorities.iter().map(|x| (
				x.0.clone(),
				x.0.clone(),
				xelendra_session_keys(
					x.2.clone(),
					x.3.clone(),
					x.4.clone(),
					x.5.clone(),
					x.6.clone(),
					x.7.clone(),
				),
			)).collect::<Vec<_>>(),
		}),
		pallet_babe: Some(Default::default()),
		pallet_grandpa: Some(Default::default()),
		pallet_im_online: Some(Default::default()),
		pallet_authority_discovery: Some(xelendra_runtime::AuthorityDiscoveryConfig {
			keys: vec![],
		}),
		pallet_staking: Some(Default::default()),
		pallet_sudo: Some(xelendra_runtime::SudoConfig {
			key: endowed_accounts[0].clone(),
		}),
		pallet_contracts: Some(xelendra_runtime::ContractsConfig {
			current_schedule: pallet_contracts::Schedule {
				..Default::default()
			},
		}),
		parachains_configuration: Some(xelendra_runtime::ParachainsConfigurationConfig {
			config: indracore_runtime_parachains::configuration::HostConfiguration {
				validation_upgrade_frequency: 600u32,
				validation_upgrade_delay: 300,
				acceptance_period: 1200,
				max_code_size: 5 * 1024 * 1024,
				max_pov_size: 50 * 1024 * 1024,
				max_head_data_size: 32 * 1024,
				group_rotation_frequency: 20,
				chain_availability_period: 4,
				thread_availability_period: 4,
				no_show_slots: 10,
				..Default::default()
			},
		}),
	}
}

/// Indracore staging testnet config.
pub fn indracore_staging_testnet_config() -> Result<IndracoreChainSpec, String> {
	let wasm_binary = indracore::WASM_BINARY.ok_or("Indracore development wasm not available")?;
	let boot_nodes = vec![];

	Ok(IndracoreChainSpec::from_genesis(
		"Indranet Testnet",
		"indranet_staging_testnet",
		ChainType::Live,
		move || indracore_staging_testnet_config_genesis(wasm_binary),
		boot_nodes,
		Some(
			TelemetryEndpoints::new(vec![(INDRACORE_STAGING_TELEMETRY_URL.to_string(), 0)])
				.expect("Indracore Staging telemetry url is valid; qed"),
		),
		Some(DEFAULT_PROTOCOL_ID),
		None,
		Default::default(),
	))
}

/// Xelendra staging testnet config.
pub fn xelendra_staging_testnet_config() -> Result<XelendraChainSpec, String> {
	let wasm_binary = xelendra::WASM_BINARY.ok_or("Xelendra development wasm not available")?;
	let boot_nodes = vec![];

	Ok(XelendraChainSpec::from_genesis(
		"Xelendra Testnet",
		"xelendra_testnet",
		ChainType::Live,
		move || XelendraGenesisExt {
			runtime_genesis_config: xelendra_staging_testnet_config_genesis(wasm_binary),
			session_length_in_blocks: None,
		},
		boot_nodes,
		Some(
			TelemetryEndpoints::new(vec![(XELENDRA_STAGING_TELEMETRY_URL.to_string(), 0)])
				.expect("Xelendra Staging telemetry url is valid; qed"),
		),
		Some(DEFAULT_PROTOCOL_ID),
		None,
		Default::default(),
	))
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn get_authority_keys_from_seed(
	seed: &str,
) -> (
	AccountId,
	AccountId,
	BabeId,
	GrandpaId,
	ImOnlineId,
	ValidatorId,
	AssignmentId,
	AuthorityDiscoveryId,
) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<ValidatorId>(seed),
		get_from_seed::<AssignmentId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),
	)
}

fn testnet_accounts() -> Vec<AccountId> {
	vec![
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		get_account_id_from_seed::<sr25519::Public>("Bob"),
		get_account_id_from_seed::<sr25519::Public>("Charlie"),
		get_account_id_from_seed::<sr25519::Public>("Dave"),
		get_account_id_from_seed::<sr25519::Public>("Eve"),
		get_account_id_from_seed::<sr25519::Public>("Ferdie"),
		get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
		get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
		get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
		get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
		get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
		get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
	]
}

/// Helper function to create indracore GenesisConfig for testing
pub fn indracore_testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		BabeId,
		GrandpaId,
		ImOnlineId,
		ValidatorId,
		AssignmentId,
		AuthorityDiscoveryId,
	)>,
	_root_key: AccountId,
	endowed_accounts: Option<Vec<AccountId>>,
) -> indracore::GenesisConfig {
	let endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(testnet_accounts);

	const ENDOWMENT: Balance = 2u128.pow(32) * SELS;
	const STASH: Balance = 100 * SELS;

	indracore::GenesisConfig {
		frame_system: Some(indracore::SystemConfig {
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_indices: Some(indracore::IndicesConfig { indices: vec![] }),
		pallet_balances: Some(indracore::BalancesConfig {
			balances: endowed_accounts
				.iter()
				.map(|k| (k.clone(), ENDOWMENT))
				.collect(),
		}),
		pallet_session: Some(indracore::SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						indracore_session_keys(
							x.2.clone(),
							x.3.clone(),
							x.4.clone(),
							x.5.clone(),
							x.6.clone(),
							x.7.clone(),
						),
					)
				})
				.collect::<Vec<_>>(),
		}),
		pallet_staking: Some(indracore::StakingConfig {
			minimum_validator_count: 1,
			validator_count: 2,
			stakers: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.1.clone(),
						STASH,
						indracore::StakerStatus::Validator,
					)
				})
				.collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			force_era: Forcing::NotForcing,
			slash_reward_fraction: Perbill::from_percent(10),
			..Default::default()
		}),
		pallet_elections_phragmen: Some(Default::default()),
		pallet_democracy: Some(indracore::DemocracyConfig::default()),
		pallet_collective_Instance1: Some(indracore::CouncilConfig {
			members: vec![],
			phantom: Default::default(),
		}),
		pallet_collective_Instance2: Some(indracore::TechnicalCommitteeConfig {
			members: vec![],
			phantom: Default::default(),
		}),
		pallet_membership_Instance1: Some(Default::default()),
		pallet_babe: Some(Default::default()),
		pallet_grandpa: Some(Default::default()),
		pallet_im_online: Some(Default::default()),
		pallet_authority_discovery: Some(indracore::AuthorityDiscoveryConfig { keys: vec![] }),
		pallet_vesting: Some(indracore::VestingConfig { vesting: vec![] }),
		pallet_treasury: Some(Default::default()),
		pallet_contracts: Some(indracore::ContractsConfig {
			current_schedule: pallet_contracts::Schedule {
				..Default::default()
			},
		}),
	}
}

/// Helper function to create xelendra GenesisConfig for testing
pub fn xelendra_testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		BabeId,
		GrandpaId,
		ImOnlineId,
		ValidatorId,
		AssignmentId,
		AuthorityDiscoveryId,
	)>,
	root_key: AccountId,
	endowed_accounts: Option<Vec<AccountId>>,
) -> xelendra_runtime::GenesisConfig {
	let endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(testnet_accounts);

	const ENDOWMENT: Balance = 1_000_000 * XEL;

	xelendra_runtime::GenesisConfig {
		frame_system: Some(xelendra_runtime::SystemConfig {
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_indices: Some(xelendra_runtime::IndicesConfig {
			indices: vec![],
		}),
		pallet_balances: Some(xelendra_runtime::BalancesConfig {
			balances: endowed_accounts.iter().map(|k| (k.clone(), ENDOWMENT)).collect(),
		}),
		pallet_session: Some(xelendra_runtime::SessionConfig {
			keys: initial_authorities.iter().map(|x| (
				x.0.clone(),
				x.0.clone(),
				xelendra_session_keys(
					x.2.clone(),
					x.3.clone(),
					x.4.clone(),
					x.5.clone(),
					x.6.clone(),
					x.7.clone(),
				),
			)).collect::<Vec<_>>(),
		}),
		pallet_babe: Some(Default::default()),
		pallet_grandpa: Some(Default::default()),
		pallet_im_online: Some(Default::default()),
		pallet_authority_discovery: Some(xelendra_runtime::AuthorityDiscoveryConfig {
			keys: vec![],
		}),
		pallet_staking: Some(Default::default()),
		pallet_sudo: Some(xelendra_runtime::SudoConfig { key: root_key }),
		pallet_contracts: Some(xelendra_runtime::ContractsConfig {
			current_schedule: pallet_contracts::Schedule {
				..Default::default()
			},
		}),
		parachains_configuration: Some(xelendra_runtime::ParachainsConfigurationConfig {
			config: indracore_runtime_parachains::configuration::HostConfiguration {
				validation_upgrade_frequency: 600u32,
				validation_upgrade_delay: 300,
				acceptance_period: 1200,
				max_code_size: 5 * 1024 * 1024,
				max_pov_size: 50 * 1024 * 1024,
				max_head_data_size: 32 * 1024,
				group_rotation_frequency: 20,
				chain_availability_period: 4,
				thread_availability_period: 4,
				no_show_slots: 10,
				max_upward_queue_count: 8,
				max_upward_queue_size: 8 * 1024,
				max_downward_message_size: 1024,
				// this is approximatelly 4ms.
				//
				// Same as `4 * frame_support::weights::WEIGHT_PER_MILLIS`. We don't bother with
				// an import since that's a made up number and should be replaced with a constant
				// obtained by benchmarking anyway.
				preferred_dispatchable_upward_messages_step_weight: 4 * 1_000_000_000,
				max_upward_message_size: 1024,
				max_upward_message_num_per_candidate: 5,
				hrmp_open_request_ttl: 5,
				hrmp_sender_deposit: 0,
				hrmp_recipient_deposit: 0,
				hrmp_channel_max_capacity: 8,
				hrmp_channel_max_total_size: 8 * 1024,
				hrmp_max_parachain_inbound_channels: 4,
				hrmp_max_parathread_inbound_channels: 4,
				hrmp_channel_max_message_size: 1024,
				hrmp_max_parachain_outbound_channels: 4,
				hrmp_max_parathread_outbound_channels: 4,
				hrmp_max_message_num_per_candidate: 5,
				..Default::default()
			},
		}),
	}
}

fn indracore_development_config_genesis(wasm_binary: &[u8]) -> indracore::GenesisConfig {
	indracore_testnet_genesis(
		wasm_binary,
		vec![get_authority_keys_from_seed("Alice")],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
	)
}

/// Indracore development config (single validator Alice)
pub fn indracore_development_config() -> Result<IndracoreChainSpec, String> {
	let wasm_binary = indracore::WASM_BINARY.ok_or("Indracore development wasm not available")?;

	Ok(IndracoreChainSpec::from_genesis(
		"Development",
		"dev",
		ChainType::Development,
		move || indracore_development_config_genesis(wasm_binary),
		vec![],
		None,
		Some(DEFAULT_PROTOCOL_ID),
		None,
		Default::default(),
	))
}

fn indracore_local_testnet_genesis(wasm_binary: &[u8]) -> indracore::GenesisConfig {
	indracore_testnet_genesis(
		wasm_binary,
		vec![
			get_authority_keys_from_seed("Alice"),
			get_authority_keys_from_seed("Bob"),
		],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
	)
}

/// Indracore local testnet config (multivalidator Alice + Bob)
pub fn indracore_local_testnet_config() -> Result<IndracoreChainSpec, String> {
	let wasm_binary = indracore::WASM_BINARY.ok_or("Indracore development wasm not available")?;

	Ok(IndracoreChainSpec::from_genesis(
		"Local Testnet",
		"local_testnet",
		ChainType::Local,
		move || indracore_local_testnet_genesis(wasm_binary),
		vec![],
		None,
		Some(DEFAULT_PROTOCOL_ID),
		None,
		Default::default(),
	))
}

fn xelendra_local_testnet_genesis(wasm_binary: &[u8]) -> xelendra_runtime::GenesisConfig {
	xelendra_testnet_genesis(
		wasm_binary,
		vec![
			get_authority_keys_from_seed("Alice"),
			get_authority_keys_from_seed("Bob"),
		],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
	)
}

/// Xelendra local testnet config (multivalidator Alice + Bob)
pub fn xelendra_local_testnet_config() -> Result<XelendraChainSpec, String> {
	let wasm_binary = xelendra::WASM_BINARY.ok_or("Xelendra development wasm not available")?;

	Ok(XelendraChainSpec::from_genesis(
		"Xelendra Local Testnet",
		"xelendra_local_testnet",
		ChainType::Local,
		move || XelendraGenesisExt {
			runtime_genesis_config: xelendra_local_testnet_genesis(wasm_binary),
			// Use 1 minute session length.
			session_length_in_blocks: Some(10),
		},
		vec![],
		None,
		Some(DEFAULT_PROTOCOL_ID),
		None,
		Default::default(),
	))
}
