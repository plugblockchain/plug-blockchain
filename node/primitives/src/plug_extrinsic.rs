#[cfg(feature = "std")]
use std::fmt;

use rstd::prelude::*;
use runtime_io::blake2_256;
use sr_primitives::codec::{Compact, Decode, Encode, Input};
use sr_primitives::generic::Era;
use sr_primitives::traits::{
	self, BlockNumberToHash, Checkable, CurrentHeight, Doughnuted, Extrinsic, Lookup, MaybeDisplay,
	Member, SimpleArithmetic, DoughnutApi, SaturatedConversion
};
use sr_primitives::weights::{Weighable, Weight};

const TRANSACTION_VERSION: u8 = 0b0000_00001;
const MASK_VERSION: u8 = 0b0000_1111;
const BIT_SIGNED: u8 = 0b1000_0000;
const BIT_DOUGHNUT: u8 = 0b0100_0000;

fn encode_with_vec_prefix<T: Encode, F: Fn(&mut Vec<u8>)>(encoder: F) -> Vec<u8> {
	let size = ::rstd::mem::size_of::<T>();
	let reserve = match size {
		x if x <= 0b0011_1111 => 1,
		x if x <= 0b0011_1111_1111_1111 => 2,
		_ => 4,
	};
	let mut v = Vec::with_capacity(reserve + size);
	v.resize(reserve, 0);
	encoder(&mut v);

	// need to prefix with the total length to ensure it's binary compatible with
	// Vec<u8>.
	let mut length: Vec<()> = Vec::new();
	length.resize(v.len() - reserve, ());
	length.using_encoded(|s| {
		v.splice(0..reserve, s.iter().cloned());
	});

	v
}

/// A extrinsic right from the external world. This is unchecked and so
/// can contain a signature.
#[derive(PartialEq, Eq, Clone)]
pub struct PlugExtrinsic<AccountId, Address, Index, Call, Signature, Doughnut> {
	/// The signature, address, number of extrinsics have come before from
	/// the same signer and an era describing the longevity of this transaction,
	/// if this is a signed extrinsic.
	pub signature: Option<(Address, Signature, Compact<Index>, Era)>,
	/// The function that should be called.
	pub function: Call,
	/// Doughnut attached, if any
	pub doughnut: Option<Doughnut>,
	/// phantom for AccountId
	pub _phantom: rstd::marker::PhantomData<AccountId>,
}

/// Definition of something that the external world might want to say; its
/// existence implies that it has been checked and is good, particularly with
/// regards to the signature.
#[derive(PartialEq, Eq, Clone)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct CheckedPlugExtrinsic<AccountId, Index, Call, Doughnut> {
	/// Who this purports to be from and the number of extrinsics that have come before
	/// from the same signer, if anyone (note this is not a signature).
	pub signed: Option<(AccountId, Index)>,
	/// The function that should be called.
	pub function: Call,
	/// Doughnut attached, if any
	pub doughnut: Option<Doughnut>,
}

impl<AccountId: Encode + Clone, Index, Call, Doughnut> Doughnuted
	for CheckedPlugExtrinsic<AccountId, Index, Call, Doughnut>
where
	Doughnut: Encode + Clone + DoughnutApi,
{
	type Doughnut = Doughnut;
	fn doughnut(&self) -> Option<&Self::Doughnut> {
		self.doughnut.as_ref()
	}
}

impl<AccountId, Index, Call, Doughnut> traits::Applyable for CheckedPlugExtrinsic<AccountId, Index, Call, Doughnut>
where
	AccountId: Member + MaybeDisplay,
	Index: Member + MaybeDisplay + SimpleArithmetic,
	Call: Member,
	Doughnut: Member,
{
	type Index = Index;
	type AccountId = AccountId;
	type Call = Call;

	fn index(&self) -> Option<&Self::Index> {
		self.signed.as_ref().map(|x| &x.1)
	}

	fn sender(&self) -> Option<&Self::AccountId> {
		self.signed.as_ref().map(|x| &x.0)
	}

	fn call(&self) -> &Self::Call {
		&self.function
	}

	fn deconstruct(self) -> (Self::Call, Option<Self::AccountId>) {
		(self.function, self.signed.map(|x| x.0))
	}
}

impl<AccountId, Address, Index, Call, Signature, Doughnut>
	PlugExtrinsic<AccountId, Address, Index, Call, Signature, Doughnut>
{
	/// New instance of a signed extrinsic aka "transaction".
	pub fn new_signed(
		index: Index,
		function: Call,
		signed: Address,
		signature: Signature,
		era: Era,
		doughnut: Option<Doughnut>,
	) -> Self {
		Self {
			signature: Some((signed, signature, index.into(), era)),
			function,
			doughnut,
			_phantom: rstd::marker::PhantomData,
		}
	}

	/// New instance of an unsigned extrinsic aka "inherent".
	pub fn new_unsigned(function: Call) -> Self {
		Self {
			signature: None,
			function,
			doughnut: None,
			_phantom: rstd::marker::PhantomData,
		}
	}
}

impl<AccountId: Encode, Address: Encode, Index: Encode, Call: Encode, Signature: Encode, Doughnut: Encode> Extrinsic
	for PlugExtrinsic<AccountId, Address, Index, Call, Signature, Doughnut>
{
	fn is_signed(&self) -> Option<bool> {
		Some(self.signature.is_some())
	}
}

impl<AccountId: Encode + Clone, Address, Index, Call, Signature: Encode + Clone, Doughnut> Doughnuted
	for PlugExtrinsic<AccountId, Address, Index, Call, Signature, Doughnut>
where
	Doughnut: Encode + Clone + DoughnutApi,
{
	type Doughnut = Doughnut;
	fn doughnut(&self) -> Option<&Self::Doughnut> {
		self.doughnut.as_ref()
	}
}

impl<AccountId, Address, Index, Call, Signature, Context, Hash, BlockNumber, Doughnut> Checkable<Context>
	for PlugExtrinsic<AccountId, Address, Index, Call, Signature, Doughnut>
where
	Address: Member + MaybeDisplay,
	Index: Member + MaybeDisplay + SimpleArithmetic,
	Compact<Index>: Encode,
	Call: Encode + Member,
	Signature: Member + traits::Verify<Signer = AccountId> + Encode + Decode,
	AccountId: Member + MaybeDisplay + Encode + Decode,
	BlockNumber: SimpleArithmetic,
	Hash: Encode,
	Context: Lookup<Source = Address, Target = AccountId>
		+ CurrentHeight<BlockNumber = BlockNumber>
		+ BlockNumberToHash<BlockNumber = BlockNumber, Hash = Hash>,
	Doughnut: Encode + DoughnutApi,
	<Doughnut as DoughnutApi>::AccountId: AsRef<[u8]>,
	<Doughnut as DoughnutApi>::Signature: AsRef<[u8]>,
{
	type Checked = CheckedPlugExtrinsic<AccountId, Index, Call, Doughnut>;

	fn check(self, context: &Context) -> Result<Self::Checked, &'static str> {
		// There's no signature so we're done
		if self.signature.is_none() {
			return Ok(Self::Checked {
				signed: None,
				function: self.function,
				doughnut: self.doughnut,
			});
		};

		let (signed, signature, index, era) = self.signature.unwrap();
		let current_u64 = context.current_height().saturated_into::<u64>();
		let h = context.block_number_to_hash(era.birth(current_u64).saturated_into())
			.ok_or("transaction birth block ancient")?;
		let signed = context.lookup(signed)?;
		let verify_signature = |payload: &[u8]| {
			if payload.len() > 256 {
				signature.verify(&blake2_256(payload)[..], &signed)
			} else {
				signature.verify(payload, &signed)
			}
		};

		let verified = if let Some(ref doughnut) = self.doughnut {
			let doughnut_signature = Signature::decode(&mut doughnut.signature().as_ref()).ok_or("doughnut has incompatible signature for runtime")?;
			let issuer = AccountId::decode(&mut doughnut.issuer().as_ref()).ok_or("doughnut has incompatible issuer for runtime")?;
			if !doughnut_signature.verify(doughnut.payload().as_ref(), &issuer) {
				return Err("bad signature in doughnut");
			}
			(&index, &self.function, era, h, &doughnut).using_encoded(verify_signature)
		} else {
			(&index, &self.function, era, h).using_encoded(verify_signature)
		};

		if !verified {
			return Err("bad signature in extrinsic");
		}

		return Ok(Self::Checked {
			signed: Some((signed, index.0)),
			function: self.function,
			doughnut: self.doughnut,
		});
	}
}

impl<AccountId, Address, Index, Call, Signature, Doughnut> Decode
	for PlugExtrinsic<AccountId, Address, Index, Call, Signature, Doughnut>
where
	AccountId: Decode,
	Address: Decode,
	Signature: Decode,
	Compact<Index>: Decode,
	Call: Decode,
	Doughnut: Decode,
{
	fn decode<I: Input>(input: &mut I) -> Option<Self> {
		// This is a little more complicated than usual since the binary format must be compatible
		// with substrate's generic `Vec<u8>` type. Basically this just means accepting that there
		// will be a prefix of vector length (we don't need to use this).
		let _length_do_not_remove_me_see_above: Vec<()> = Decode::decode(input)?;

		let version = input.read_byte()?;

		let is_signed = version & BIT_SIGNED != 0;
		let has_doughnut = version & BIT_DOUGHNUT != 0;
		let version = version & MASK_VERSION;

		if version != TRANSACTION_VERSION {
			return None;
		}

		let signature = if is_signed {
			Some(Decode::decode(input)?)
		} else {
			None
		};
		let function = Decode::decode(input)?;

		let doughnut = if has_doughnut {
			Some(Decode::decode(input)?)
		} else {
			None
		};

		Some(PlugExtrinsic {
			signature,
			function,
			doughnut,
			_phantom: rstd::marker::PhantomData,
		})
	}
}

impl<AccountId, Address, Index, Call, Signature, Doughnut> Encode
	for PlugExtrinsic<AccountId, Address, Index, Call, Signature, Doughnut>
where
	AccountId: Encode,
	Address: Encode,
	Signature: Encode,
	Compact<Index>: Encode,
	Call: Encode,
	Doughnut: Encode,
{
	fn encode(&self) -> Vec<u8> {
		encode_with_vec_prefix::<Self, _>(|v| {
			// 1 byte version id.
			let mut version = TRANSACTION_VERSION;
			if self.signature.is_some() {
				version |= BIT_SIGNED;
			}
			if self.doughnut.is_some() {
				version |= BIT_DOUGHNUT;
			}
			v.push(version);

			if let Some(s) = self.signature.as_ref() {
				s.encode_to(v);
			}
			self.function.encode_to(v);
			if let Some(d) = self.doughnut.as_ref() {
				d.encode_to(v);
			}
		})
	}
}

#[cfg(feature = "std")]
impl<AccountId: Encode, Address: Encode, Index, Signature: Encode, Call: Encode, Doughnut: Encode> serde::Serialize
	for PlugExtrinsic<AccountId, Address, Index, Call, Signature, Doughnut>
where
	Compact<Index>: Encode,
{
	fn serialize<S>(&self, seq: S) -> Result<S::Ok, S::Error>
	where
		S: ::serde::Serializer,
	{
		self.using_encoded(|bytes| seq.serialize_bytes(bytes))
	}
}

#[cfg(feature = "std")]
impl<AccountId, Address, Index, Call, Signature, Doughnut> fmt::Debug
	for PlugExtrinsic<AccountId, Address, Index, Call, Signature, Doughnut>
where
	AccountId: fmt::Debug,
	Address: fmt::Debug,
	Index: fmt::Debug,
	Call: fmt::Debug,
	Signature: fmt::Debug,
	Doughnut: fmt::Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"PlugExtrinsic({:?}, {:?}, {:?})",
			self.signature.as_ref().map(|x| (&x.0, &x.2)),
			self.function,
			self.doughnut
		)
	}
}

impl<AccountId, Index, Call, Doughnut> Weighable for CheckedPlugExtrinsic<AccountId, Index, Call, Doughnut>
where
	Call: Weighable,
{
	fn weight(&self, len: usize) -> Weight {
		self.function.weight(len)
	}
}
