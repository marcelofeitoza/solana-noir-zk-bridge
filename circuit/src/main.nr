fn main(x: u32, y: pub u32) -> pub bool {
    let z = x + y;
    assert(z == 42);
    true
}

#[test]
fn test_main() {
    let result = main(23, 19);
    assert(result == true);
}
