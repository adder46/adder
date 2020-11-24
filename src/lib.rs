#[derive(Clone, Copy, Debug, Default)]
pub struct RippleCarryAdder {
    adders: [FullAdder; 8]
}

impl RippleCarryAdder {

    pub fn add(&mut self, a: u8, b: u8) -> Result<u8, &'static str>  {
        for i in 0..self.adders.len() {
            let bit1 = Bit((a >> i) & 1);
            let bit2 = Bit((b >> i) & 1);
            let (left, right) = self.adders.split_at_mut(i+1);
            let current_adder = &mut left[i];
            current_adder.add(bit1, bit2);
            match right.get_mut(0) {
                Some(next_adder) => {
                    next_adder.carry_in = current_adder.carry_out;
                }
                None => {
                    if current_adder.carry_out == Bit(1) {
                        return Err("Overflow.");
                    }
                } 
            }
        }
        Ok(self.get_result())
    }

    fn get_result(&self) -> u8 {
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
    halfadder2: HalfAdder
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
