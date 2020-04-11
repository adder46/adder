use adder::RippleCarryAdder;

#[test]
fn test_add() { 
    let mut adder: RippleCarryAdder = Default::default();
    adder.add(4, 4);
    assert_eq!(adder.get_result(), 8)
}
