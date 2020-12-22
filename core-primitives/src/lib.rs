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

//! Polkadot types shared between the runtime and the Node-side code.
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use sp_runtime::{
	generic, traits::{Verify, BlakeTwo256, IdentifyAccount}, MultiSignature
};

/// Opaque, encoded, unchecked extrinsic.
pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Alias to the public key used for this chain, actually a `MultiSigner`. Like the signature, this
/// also isn't a fixed size when encoded, as different cryptos have different size public keys.
pub type AccountPublic = <Signature as Verify>::Signer;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them.
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = u128;

/// Type used for expressing timestamp.
pub type Moment = u64;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// A timestamp: milliseconds since the unix epoch.
/// `u64` is enough to represent a duration of half a billion years, when the
/// time scale is milliseconds.
pub type Timestamp = u64;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;
/// Header type.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// Block ID.
pub type BlockId = generic::BlockId<Block>;

/// Identifier for a chain. 32-bit should be plenty.
pub type ChainId = u32;

/// The information that goes alongside a transfer_into_parachain operation. Entirely opaque, it
/// will generally be used for identifying the reason for the transfer. Typically it will hold the
/// destination account to which the transfer should be credited. If still more information is
/// needed, then this should be a hash with the pre-image presented via an off-chain mechanism on
/// the parachain.
pub type Remark = [u8; 32];

/// Unit type wrapper around [`Hash`] that represents a candidate hash.
///
/// This type is produced by [`CandidateReceipt::hash`].
///
/// This type makes it easy to enforce that a hash is a candidate hash on the type level.
#[derive(Clone, Copy, Encode, Decode, Hash, Eq, PartialEq, Debug, Default)]
pub struct CandidateHash(pub Hash);

/// A message sent from the relay-chain down to a parachain.
///
/// The size of the message is limited by the `config.max_downward_message_size` parameter.
pub type DownwardMessage = sp_std::vec::Vec<u8>;

#[derive(Encode, Decode, Clone, sp_runtime::RuntimeDebug, PartialEq, Eq, Hash)]
pub struct OutboundHrmpMessage<Id> {
	/// The para that will get this message in its downward message queue.
	pub recipient: Id,
	/// The message payload.
	pub data: sp_std::vec::Vec<u8>,
}

/// An HRMP message seen from the perspective of a recipient.
#[derive(Encode, Decode, Clone, sp_runtime::RuntimeDebug, PartialEq)]
pub struct InboundHrmpMessage<BlockNumber = crate::BlockNumber> {
	/// The block number at which this message was sent.
	/// Specifically, it is the block number at which the candidate that sends this message was
	/// enacted.
	pub sent_at: BlockNumber,
	/// The message payload.
	pub data: sp_std::vec::Vec<u8>,
}

/// A wrapped version of `DownwardMessage`. The difference is that it has attached the block number when
/// the message was sent.
#[derive(Encode, Decode, Clone, sp_runtime::RuntimeDebug, PartialEq)]
pub struct InboundDownwardMessage<BlockNumber = crate::BlockNumber> {
	/// The block number at which this messages was put into the downward message queue.
	pub sent_at: BlockNumber,
	/// The actual downward message to processes.
	pub msg: DownwardMessage,
}