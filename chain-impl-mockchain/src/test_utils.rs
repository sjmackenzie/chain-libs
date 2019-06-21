use quickcheck::{Arbitrary, Gen};
use quickcheck_macros::quickcheck;
use rand::distributions::uniform::{SampleUniform, Uniform};
use rand::Rng;
use std::iter;

pub fn arbitrary_split_value(gen: &mut impl Gen, value: u64, parts: u16) -> Vec<u64> {
    let mut in_values: Vec<_> = iter::once(0)
        .chain(iter::repeat_with(|| arbitrary_range(gen, 0..=value)))
        .take(parts as usize)
        .chain(iter::once(value))
        .collect();
    in_values.sort();
    in_values.windows(2).map(|pair| pair[1] - pair[0]).collect()
}

pub fn arbitrary_range<T: SampleUniform>(gen: &mut impl Gen, range: impl Into<Uniform<T>>) -> T {
    gen.sample(range.into())
}

mod tests {
    use super::*;

    #[quickcheck]
    fn split_value_splits_whole_value(split_value: ArbitrarySplitValue) -> () {
        assert_eq!(
            split_value.parts,
            split_value.split.len(),
            "Invalid split length"
        );
        assert_eq!(
            split_value.value,
            split_value.split.iter().sum(),
            "Invalid split sum"
        );
    }

    #[derive(Clone, Debug)]
    struct ArbitrarySplitValue {
        value: u64,
        parts: usize,
        split: Vec<u64>,
    }

    impl Arbitrary for ArbitrarySplitValue {
        fn arbitrary<G: Gen>(gen: &mut G) -> Self {
            let value = u64::arbitrary(gen);
            let parts = u16::arbitrary(gen);
            let split = arbitrary_split_value(gen, value, parts);
            let value = match parts {
                0 => 0,
                _ => value,
            };
            ArbitrarySplitValue {
                value,
                parts: parts as usize,
                split,
            }
        }
    }
}
