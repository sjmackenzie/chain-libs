use crate::account;
use crate::key::{deserialize_public_key, serialize_public_key, Hash};
use crate::leadership::genesis::GenesisPraosLeader;

use chain_core::mempack::{ReadBuf, ReadError, Readable};
use chain_core::property;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StakePoolId(Hash);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StakePoolInfo {
    pub serial: u128,
    pub owners: Vec<account::Identifier>,
    pub initial_key: GenesisPraosLeader,
}

impl StakePoolInfo {
    pub fn to_id(&self) -> StakePoolId {
        let mut v = Vec::new();
        v.extend_from_slice(&self.serial.to_be_bytes());
        for o in &self.owners {
            v.extend_from_slice(o.as_ref().as_ref())
        }
        v.extend_from_slice(self.initial_key.kes_public_key.as_ref());
        v.extend_from_slice(self.initial_key.vrf_public_key.as_ref());
        StakePoolId(Hash::hash_bytes(&v))
    }
}

impl property::Serialize for StakePoolId {
    type Error = std::io::Error;
    fn serialize<W: std::io::Write>(&self, mut writer: W) -> Result<(), Self::Error> {
        writer.write_all(self.0.as_ref())
    }
}

impl Readable for StakePoolId {
    fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        Hash::read(buf).map(StakePoolId)
    }
}

impl property::Serialize for GenesisPraosLeader {
    type Error = std::io::Error;
    fn serialize<W: std::io::Write>(&self, mut writer: W) -> Result<(), Self::Error> {
        serialize_public_key(&self.kes_public_key, &mut writer)?;
        serialize_public_key(&self.vrf_public_key, &mut writer)?;
        Ok(())
    }
}

impl Readable for GenesisPraosLeader {
    fn read<'a>(reader: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        let kes_public_key = deserialize_public_key(reader)?;
        let vrf_public_key = deserialize_public_key(reader)?;
        Ok(GenesisPraosLeader {
            vrf_public_key,
            kes_public_key,
        })
    }
}

impl property::Serialize for StakePoolInfo {
    type Error = std::io::Error;
    fn serialize<W: std::io::Write>(&self, writer: W) -> Result<(), Self::Error> {
        assert!(self.owners.len() < 256);

        use chain_core::packer::Codec;

        let mut codec = Codec::new(writer);
        codec.put_u128(self.serial)?;
        codec.put_u8(self.owners.len() as u8)?;
        for o in &self.owners {
            serialize_public_key(o.as_ref(), &mut codec)?;
        }
        self.initial_key.serialize(&mut codec)?;
        Ok(())
    }
}

impl Readable for StakePoolInfo {
    fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        let serial = buf.get_u128()?;
        let owner_nb = buf.get_u8()? as usize;
        let mut owners = Vec::with_capacity(owner_nb);
        for _ in 0..owner_nb {
            let pub_key = account::Identifier::read(buf)?;
            owners.push(pub_key)
        }
        let initial_key = GenesisPraosLeader::read(buf)?;

        Ok(StakePoolInfo {
            serial,
            owners,
            initial_key,
        })
    }
}

impl From<StakePoolId> for [u8; 32] {
    fn from(h: StakePoolId) -> [u8; 32] {
        h.0.into()
    }
}

impl From<Hash> for StakePoolId {
    fn from(hash: Hash) -> Self {
        StakePoolId(hash)
    }
}
impl From<chain_crypto::Blake2b256> for StakePoolId {
    fn from(hash: chain_crypto::Blake2b256) -> Self {
        StakePoolId(hash.into())
    }
}
impl std::fmt::Display for StakePoolId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for StakePoolId {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            StakePoolId(Arbitrary::arbitrary(g))
        }
    }
}
