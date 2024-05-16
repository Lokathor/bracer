use bracer::{a32_fake_blx, read_spsr, write_spsr};

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

#[test]
fn test_a32_fake_blx() {
  let asm_lines = a32_fake_blx!("r0");
  let mut lines = asm_lines.lines();
  let mut line = lines.next().unwrap();
  let mut line_iter = line.split(' ');

  assert_eq!(line_iter.next().unwrap(), "adr");
  assert_eq!(line_iter.next().unwrap(), "lr,");
  let local_label = line_iter.next().unwrap();
  assert!(line_iter.next().is_none());

  line = lines.next().unwrap();
  assert_eq!(line.trim(), "bx r0");

  line = lines.next().unwrap();
  assert_eq!(line.trim(), format!("{local_label}:"));

  assert!(lines.next().is_none());
}
