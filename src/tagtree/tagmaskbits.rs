#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) struct TagMaskBits(u32);

impl TagMaskBits {
    pub const BITS: usize = u32::BITS as usize;
    pub const CLEAR: Self = Self(0);
    pub const ALL: Self = Self(u32::MAX);
    pub const UPPER_HALF: Self = Self(Self::ALL.0 << ((Self::BITS / 2) as u32));
    pub const LOWER_HALF: Self = Self(Self::ALL.0 >> ((Self::BITS / 2) as u32));
    pub const FIRST: Self = Self(0x1);

    // idx zero is the smallest value bit.
    pub fn unset_bit(&mut self, idx: usize) {
        assert!(idx < Self::BITS);
        self.0 &= !(1u32 << (idx));
    }

    // idx zero is the smallest value bit.
    pub fn set_bit(&mut self, idx: usize) {
        assert!(idx < Self::BITS);
        self.0 |= 1u32 << (idx);
    }

    pub fn is_set(&self, idx: usize) -> bool {
        (self.0 & (1u32 << (idx))) != 0
    }

    pub fn set_union(&mut self, other: &Self) {
        self.0 |= other.0;
    }

    pub fn set_intersect(&mut self, other: &Self) {
        self.0 &= other.0;
    }

    pub fn intersect(&self, other: &Self) -> Self {
        TagMaskBits(self.0 & other.0)
    }

    pub fn last_idx(&self) -> Option<usize> {
        if self.0 == 0 {
            None
        } else {
            // Use leading instead of trailing to get last.
            Some(Self::BITS - (self.0.leading_zeros() as usize))
        }
    }
}

impl Iterator for &mut TagMaskBits {
    type Item = usize;

    // Iterate from lowest value bits upwards
    fn next(&mut self) -> Option<usize> {
        if self.0 != 0 {
            let idx = (self.0.trailing_zeros()) as usize;
            self.unset_bit(idx);
            Some(idx)
        } else {
            None
        }
    }
}

impl ExactSizeIterator for &mut TagMaskBits {
    fn len(&self) -> usize {
        self.0.count_ones() as usize
    }
}

#[cfg(test)]
mod tests {
    use super::TagMaskBits;

    #[test]
    fn test_iterate_all_bits() {
        let mut bits = TagMaskBits::ALL;

        assert_eq!(
            (0..TagMaskBits::BITS).collect::<Vec<usize>>(),
            bits.map(|x| x).collect::<Vec<usize>>()
        );
    }

    #[test]
    fn test_iterate_first_bit() {
        let mut bits = TagMaskBits(0x80000000);

        assert_eq!(
            (31..32).collect::<Vec<usize>>(),
            bits.map(|x| x).collect::<Vec<usize>>()
        );
    }

    #[test]
    fn test_iterate_last_bit() {
        let mut bits = TagMaskBits(1);

        assert_eq!(
            (0..1).collect::<Vec<usize>>(),
            bits.map(|x| x).collect::<Vec<usize>>()
        );
    }

    #[test]
    fn test_iterate_no_bits() {
        let mut bits = TagMaskBits(0);

        assert_eq!(Vec::<usize>::new(), bits.map(|x| x).collect::<Vec<usize>>());
    }

    #[test]
    fn test_unset_first_bit() {
        let mut bits = TagMaskBits(1);
        bits.unset_bit(0);

        assert_eq!(TagMaskBits(0), bits);
    }

    #[test]
    fn test_unset_last_bit() {
        let mut bits = TagMaskBits(0x80000000);
        bits.unset_bit(31);

        assert_eq!(TagMaskBits(0), bits);
    }

    #[test]
    fn test_unset_last_bit_affects_no_other_bits() {
        let mut bits = TagMaskBits(0x7FFFFFFF);
        bits.unset_bit(31);

        assert_eq!(TagMaskBits(0x7FFFFFFF), bits);
    }

    #[test]
    fn test_set_first_bit() {
        let mut bits = TagMaskBits(0);
        bits.set_bit(0);

        assert_eq!(TagMaskBits(1), bits);
    }

    #[test]
    fn test_set_last_bit() {
        let mut bits = TagMaskBits(0);
        bits.set_bit(31);

        assert_eq!(TagMaskBits(0x80000000), bits);
    }

    #[test]
    fn test_set_last_bit_affects_no_other_bits() {
        let mut bits = TagMaskBits(0x40000000);
        bits.set_bit(30);

        assert_eq!(TagMaskBits(0x40000000), bits);
    }
}
