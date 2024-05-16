use bracer::{read_spsr, write_spsr};

#[test]
fn test_read_spsr() {
  assert_eq!(read_spsr!("r0"), "mrs r0, SPSR");
  assert_eq!(read_spsr!("R0"), "mrs R0, SPSR");
  assert_eq!(read_spsr!("lr"), "mrs lr, SPSR");
}

#[test]
fn test_write_spsr() {
  assert_eq!(write_spsr!("r0"), "msr r0, SPSR");
  assert_eq!(write_spsr!("R0"), "msr R0, SPSR");
  assert_eq!(write_spsr!("lr"), "msr lr, SPSR");
}
