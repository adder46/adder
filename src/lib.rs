use std::ops::{BitAnd, BitOr, BitXor};

#[cfg(test)]
use rstest_reuse::{self, *};

#[derive(Clone, Copy, Debug, Default)]
pub struct CarrySelectAdder {
    upper1: RippleCarryAdder,
    upper2: RippleCarryAdder,
    lower: RippleCarryAdder,
}

impl CarrySelectAdder {
    pub fn add(mut self, a: u16, b: u16) -> Result<u16, &'static str> {
        let (a_high, a_low) = self.bisect(a);
        let (b_high, b_low) = self.bisect(b);

        let (tx1, rx1) = std::sync::mpsc::channel();
        let (tx2, rx2) = std::sync::mpsc::channel();

        let u1 = std::thread::spawn(move || {
            self.upper1.adders[0].carry_in = Bit(1);
            self.upper1.add(a_high, b_high).unwrap();
            tx1.send(self.upper1.get_result()).unwrap();
        });

        let u2 = std::thread::spawn(move || {
            self.upper2.adders[0].carry_in = Bit(0);
            self.upper2.add(a_high, b_high).unwrap();
            tx2.send(self.upper2.get_result()).unwrap();
        });

        let _ = u1.join().unwrap();
        let _ = u2.join().unwrap();

        match self.lower.add(a_low, b_low) {
            Ok(_) => {
                let upper = rx2.recv().unwrap();
                let lower = self.lower.get_result();
                return Ok(self.combine(upper, lower));
            },
            Err(_) => {
                let upper = rx1.recv().unwrap();
                let lower = self.lower.get_result();
                return Ok(self.combine(upper, lower));
            },
        }        
    }

    fn bisect(&self, n: u16) -> (u8, u8) {
        (((n & 0xFF00) >> 8) as u8, (n & 0x00FF) as u8)
    }

    fn combine(&self, a: u8, b: u8) -> u16 {
        let mut result: u16 = 0;
        result |= (a as u16) << 8 | b as u16;
        result
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RippleCarryAdder {
    adders: [FullAdder; 8],
}

impl RippleCarryAdder {
    pub fn add(&mut self, a: u8, b: u8) -> Result<u8, &'static str> {
        for i in 0..self.adders.len() {
            /* The bits are extracted from the narrow end by shifting right `i` positions,
             * where 0 <= i < 8. On first iteration, i == 0, meaning that nothing is shifted.
             * However, each time through the loop, the number of shifted positions (`i`)
             * increases by 1 until it reaches 7. As a result, previously extracted bit is
             * discarded and the bit we want to extract next comes to its place, i.e., becomes LSB.
             * By bitwise ANDing the resulting number with 1 (mask), the individual bit at the end
             * is effectively extracted from the number - masking out the rest.
             */
            let bit1 = Bit((a >> i) & 1);
            let bit2 = Bit((b >> i) & 1);
            /* The adders are split into two halves at the position following the current adder (`i`),
             * which places the current adder in the last position in the left half, while the next
             * adder becomes the first one in the right half.
             */
            let (left, right) = self.adders.split_at_mut(i + 1);
            let current_adder = &mut left[i];
            current_adder.add(bit1, bit2);
            match right.get_mut(0) {
                Some(next_adder) => {
                    // Pass the carry bit to the next adder if the current adder is not the last one.
                    next_adder.carry_in = current_adder.carry_out;
                }
                None => {
                    if current_adder.carry_out == Bit(1) {
                        /* Having no adders that come after the current one,
                         * means that current adder is the last one. Furthermore,
                         * if the carry out for that adder is 1, we have an overflow.
                         */
                        return Err("Overflow.");
                    }
                }
            }
        }
        Ok(self.get_result())
    }

    fn get_result(&self) -> u8 {
        /* Since the addition was done starting from the narrow end,
         * when building the result, the adders are reversed so that
         * the adder holding the MSB comes first. The bits are packed
         * back into the result by shifting left by 1 each time through
         * the loop and ORing the result with the sum of an individual adder.
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
        case(255, 255, Ok(510)),
    )]
    fn carry_select_add(a: u16, b: u16, expected: Result<u16, &'static str>) {
        let adder = CarrySelectAdder::default();
        let result = adder.add(a, b);
        assert_eq!(result, expected);
    }

    #[rstest(
        a,
        b,
        expected,
        case(4, 4, Ok(8)),
        case(128, 127, Ok(255)),
        case(128, 128, Err("Overflow.")),
        case(187, 246, Err("Overflow.")),
        case(255, 255, Err("Overflow.")),
    )]
    fn ripple_add(a: u8, b: u8, expected: Result<u8, &'static str>) {
        let mut adder = RippleCarryAdder::default();
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
        let mut full_adder = FullAdder::default();
        full_adder.add(a, b);
        assert_eq!(full_adder.carry_out, expected_carry_out);
        assert_eq!(full_adder.sum, expected_sum);
    }

    #[apply(possible_combinations)]
    fn half_add(a: Bit, b: Bit, expected_carry_out: Bit, expected_sum: Bit) {
        let mut half_adder = HalfAdder::default();
        half_adder.add(a, b);
        assert_eq!(half_adder.carry_out, expected_carry_out);
        assert_eq!(half_adder.sum, expected_sum);
    }
}
