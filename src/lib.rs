use std::ops::{BitAnd, BitOr, BitXor};

#[cfg(test)]
use rstest_reuse::{self, *};

#[derive(Clone, Copy, Debug, Default)]
pub struct RippleCarryAdder {
    adders: [FullAdder; 8],
}

impl RippleCarryAdder {
    pub fn add(&mut self, a: u8, b: u8) -> Result<u8, &'static str> {
        for i in 0..self.adders.len() {
            /* We are extracting the bits in reverse, by shifting right `i` positions,
             * where 0 <= i < 8. In the beginning, i=0, so we are not shifting anything.
             * However, each time through the loop, the number of positions (`i`) we are
             * shifting right increases by 1 until it reaches 7. This means that the
             * previously extracted bit is discarded, and the bit we want to extract next
             * comes to its place, i.e., to the least significant position. By Boolean ANDing
             * the resulting number with 1 (mask), the individual bit is effectively extracted
             * from the number - masking out the rest.
             */

            let bit1 = Bit((a >> i) & 1);
            let bit2 = Bit((b >> i) & 1);

            /* The adders are split in half at the position following the current adder,
             * which means that the current adder becomes the last one in the left half,
             * and the next adder is the first one in the right half.
             */

            let (left, right) = self.adders.split_at_mut(i + 1);
            let current_adder = &mut left[i];
            current_adder.add(bit1, bit2);
            match right.get_mut(0) {
                Some(next_adder) => {
                    // pass the carry bit if there is an adder following the current one
                    next_adder.carry_in = current_adder.carry_out;
                }
                None => {
                    if current_adder.carry_out == Bit(1) {
                        /* having no adders that come after the current one,
                         * means that current adder is the last one, and if
                         * the carry out for that adder is 1, 8-bit ripple carry
                         * adder overflowed.
                         */
                        return Err("Overflow.");
                    }
                }
            }
        }
        Ok(self.get_result())
    }

    fn get_result(&self) -> u8 {
        /* Since the adding process was in reverse,
         * when building the result, the adders are
         * reversed so that the adder holding the MSB
         * comes first. We start from 0, and pack the bits
         * back into the result, by shifting left by 1
         * each time through the loop and ORing with the sum.
         */
        let mut result = 0;
        for adder in self.adders.iter().rev() {
            result <<= 1;
            result |= adder.sum.0;
        }
        result
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct FullAdder {
    carry_in: Bit,
    carry_out: Bit,
    sum: Bit,
    halfadder1: HalfAdder,
    halfadder2: HalfAdder,
}

impl FullAdder {
    fn add(&mut self, a: Bit, b: Bit) {
        self.halfadder1.add(a, b);
        self.halfadder2.add(self.halfadder1.sum, self.carry_in);
        self.sum = self.halfadder2.sum;
        self.carry_out = self.halfadder1.carry_out | self.halfadder2.carry_out;
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct HalfAdder {
    carry_out: Bit,
    sum: Bit,
}

impl HalfAdder {
    fn add(&mut self, a: Bit, b: Bit) {
        self.carry_out = a & b;
        self.sum = a ^ b;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Bit(u8);

impl BitAnd for Bit {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Bit(self.0 & rhs.0)
    }
}

impl BitOr for Bit {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Bit(self.0 | rhs.0)
    }
}

impl BitXor for Bit {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        Bit(self.0 ^ rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest(
        a,
        b,
        expected,
        case(4, 4, Ok(8)),
        case(128, 127, Ok(255)),
        case(128, 128, Err("Overflow."))
    )]
    fn ripple_add(a: u8, b: u8, expected: Result<u8, &'static str>) {
        let mut adder: RippleCarryAdder = Default::default();
        let result = adder.add(a, b);
        assert_eq!(result, expected);
    }

    #[template]
    #[rstest(
        a,
        b,
        expected_carry_out,
        expected_sum,
        case(Bit(0), Bit(0), Bit(0), Bit(0)),
        case(Bit(0), Bit(1), Bit(0), Bit(1)),
        case(Bit(1), Bit(0), Bit(0), Bit(1)),
        case(Bit(1), Bit(1), Bit(1), Bit(0))
    )]
    fn possible_combinations(a: Bit, b: Bit, expected_carry_out: Bit, expected_sum: Bit) {}

    #[apply(possible_combinations)]
    fn full_add(a: Bit, b: Bit, expected_carry_out: Bit, expected_sum: Bit) {
        let mut full_adder: FullAdder = Default::default();
        full_adder.add(a, b);
        assert_eq!(full_adder.carry_out, expected_carry_out);
        assert_eq!(full_adder.sum, expected_sum);
    }

    #[apply(possible_combinations)]
    fn half_add(a: Bit, b: Bit, expected_carry_out: Bit, expected_sum: Bit) {
        let mut half_adder: HalfAdder = Default::default();
        half_adder.add(a, b);
        assert_eq!(half_adder.carry_out, expected_carry_out);
        assert_eq!(half_adder.sum, expected_sum);
    }
}
