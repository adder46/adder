use adder::RippleCarryAdder;

#[test]
fn add() {
    let mut adder: RippleCarryAdder = Default::default();
    let result = adder.add(4, 4);
    assert_eq!(result, Ok(8));
}

#[test]
fn add_max() {
    let mut adder: RippleCarryAdder = Default::default();
    let result = adder.add(128, 127);
    assert_eq!(result, Ok(255));
}

#[test]
fn add_overflow() {
    let mut adder: RippleCarryAdder = Default::default();
    let result = adder.add(128, 128);
    assert_eq!(result, Err("Overflow."));
}