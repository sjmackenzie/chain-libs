use crate::config::ConfigParam;
use chain_core::mempack::{ReadBuf, ReadError, Readable};
use chain_core::property;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "generic-serialization",
    derive(serde_derive::Serialize, serde_derive::Deserialize),
    serde(transparent)
)]
pub struct ConfigParams(pub(crate) Vec<ConfigParam>);

impl ConfigParams {
    pub fn new() -> Self {
        ConfigParams(Vec::new())
    }

    pub fn push(&mut self, config: ConfigParam) {
        self.0.push(config)
    }

    pub fn iter(&self) -> std::slice::Iter<ConfigParam> {
        self.0.iter()
    }
}

impl property::Serialize for ConfigParams {
    type Error = std::io::Error;
    fn serialize<W: std::io::Write>(&self, mut writer: W) -> Result<(), Self::Error> {
        // FIXME: put params in canonical order (e.g. sorted by tag)?
        use chain_core::packer::*;
        Codec::new(&mut writer).put_u16(self.0.len() as u16)?;
        for config in &self.0 {
            config.serialize(&mut writer)?
        }
        Ok(())
    }
}

impl Readable for ConfigParams {
    fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        // FIXME: check canonical order?
        let len = buf.get_u16()?;
        let mut configs = vec![];
        for _ in 0..len {
            configs.push(ConfigParam::read(buf)?);
        }
        Ok(ConfigParams(configs))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chain_addr::Discrimination;
    use quickcheck::{Arbitrary, Gen, TestResult};

    quickcheck! {
        fn initial_ents_serialization_bijection(b: ConfigParams) -> TestResult {
            property::testing::serialization_bijection_r(b)
        }
    }

    impl Arbitrary for ConfigParams {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let size = u8::arbitrary(g) as usize;
            ConfigParams(
                std::iter::repeat_with(|| ConfigParam::arbitrary(g))
                    .take(size)
                    .collect(),
            )
        }
    }

    impl ConfigParams {
        pub fn arbitrary_all_params(g: &mut impl Gen, discrimination: Discrimination) -> Self {
            ConfigParams(vec![
                ConfigParam::Discrimination(discrimination),
                ConfigParam::Block0Date(Arbitrary::arbitrary(g)),
                ConfigParam::ConsensusVersion(Arbitrary::arbitrary(g)),
                ConfigParam::SlotsPerEpoch(Arbitrary::arbitrary(g)),
                ConfigParam::SlotDuration(Arbitrary::arbitrary(g)),
                ConfigParam::EpochStabilityDepth(Arbitrary::arbitrary(g)),
                ConfigParam::ConsensusGenesisPraosActiveSlotsCoeff(Arbitrary::arbitrary(g)),
                ConfigParam::MaxNumberOfTransactionsPerBlock(Arbitrary::arbitrary(g)),
                ConfigParam::BftSlotsRatio(Arbitrary::arbitrary(g)),
                ConfigParam::AddBftLeader(Arbitrary::arbitrary(g)),
                ConfigParam::LinearFee(Arbitrary::arbitrary(g)),
                ConfigParam::ProposalExpiration(Arbitrary::arbitrary(g)),
                ConfigParam::KESUpdateSpeed(Arbitrary::arbitrary(g)),
            ])
        }
    }
}
