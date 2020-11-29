#[derive(Clone, Copy, Debug, Default)]
pub struct RippleCarryAdder {
    adders: [FullAdder; 8],
}

impl RippleCarryAdder {
    pub fn add(&mut self, a: u8, b: u8) -> Result<u8, &'static str> {
        for i in 0..self.adders.len() {
            // We are extracting the bits in reverse, by shifting right `i` positions,
            // where 0 <= i < 8. In the beginning, i=0, so we are not shifting anything.
            // However, each time through the loop, the number of positions (`i`) we are
            // shifting right increases by 1 until it reaches 7. This means that the
            // previously extracted bit is discarded, and the bit we want to extract next
            // comes to its place, i.e., to the least significant position. By Boolean ANDing
            // the resulting number with 1 (mask), the individual bit is effectively extracted
            // from the number - masking out the rest.
            let bit1 = Bit((a >> i) & 1);
            let bit2 = Bit((b >> i) & 1);
            // The adders are split in half at the position following the current adder,
            // which means that the current adder becomes the last one in the left half,
            //and the next adder is the first one in the right half.
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
                        // having no adders that come after the current one,
                        // means that current adder is the last one, and if
                        // the carry out for that adder is 1, 8-bit ripple carry
                        // adder overflowed.
                        return Err("Overflow.");
                    }
                }
            }
        }
        Ok(self.get_result())
    }

    fn get_result(&self) -> u8 {
        // Since the adding process was in reverse,
        // when building the result, the adders are
        // reversed so that the adder holding the MSB
        // comes first. We start from 0, and pack the bits
        // back into the result, by shifting left by 1
        // each time through the loop and ORing with the sum.
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
        self.carry_out = Bit(self.halfadder1.carry_out.0 | self.halfadder2.carry_out.0);
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct HalfAdder {
    carry_out: Bit,
    sum: Bit,
}

impl HalfAdder {
    fn add(&mut self, a: Bit, b: Bit) {
        self.carry_out = Bit(a.0 & b.0);
        self.sum = Bit(a.0 ^ b.0);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Bit(u8);

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;
    #[test]
    fn ripple_add() {
        let mut adder: RippleCarryAdder = Default::default();
        let result = adder.add(4, 4);
        assert_eq!(result, Ok(8));
    }

    #[test]
    fn ripple_add_max() {
        let mut adder: RippleCarryAdder = Default::default();
        let result = adder.add(128, 127);
        assert_eq!(result, Ok(255));
    }

    #[test]
    fn ripple_add_overflow() {
        let mut adder: RippleCarryAdder = Default::default();
        let result = adder.add(128, 128);
        assert_eq!(result, Err("Overflow."));
    }

    #[test_case(Bit(0), Bit(0), Bit(0), Bit(0); "0 and 0")]
    #[test_case(Bit(0), Bit(1), Bit(0), Bit(1); "0 and 1")]
    #[test_case(Bit(1), Bit(0), Bit(0), Bit(1); "1 and 0")]
    #[test_case(Bit(1), Bit(1), Bit(1), Bit(0); "1 and 1")]
    fn full_add(a: Bit, b: Bit, expected_carry_out: Bit, expected_sum: Bit) {
        let mut fulladder: FullAdder = Default::default();
        fulladder.add(a, b);
        assert_eq!(fulladder.carry_out, expected_carry_out);
        assert_eq!(fulladder.sum, expected_sum);
    }

    #[test_case(Bit(0), Bit(0), Bit(0), Bit(0); "0 and 0")]
    #[test_case(Bit(0), Bit(1), Bit(0), Bit(1); "0 and 1")]
    #[test_case(Bit(1), Bit(0), Bit(0), Bit(1); "1 and 0")]
    #[test_case(Bit(1), Bit(1), Bit(1), Bit(0); "1 and 1")]
    fn half_add(a: Bit, b: Bit, expected_carry_out: Bit, expected_sum: Bit) {
        let mut halfadder: HalfAdder = Default::default();
        halfadder.add(a, b);
        assert_eq!(halfadder.carry_out, expected_carry_out);
        assert_eq!(halfadder.sum, expected_sum);
    }
}
