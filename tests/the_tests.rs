use bracer::read_spsr;

#[test]
fn test_read_spsr() {
  assert_eq!(read_spsr!("r0"), "mrs r0, SPSR");
  assert_eq!(read_spsr!("R0"), "mrs R0, SPSR");
  assert_eq!(read_spsr!("lr"), "mrs lr, SPSR");
}
