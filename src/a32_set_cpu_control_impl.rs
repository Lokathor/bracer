use super::*;

pub fn a32_set_cpu_control_impl(token_stream: TokenStream) -> TokenStream {
  let mut stream_iter = token_stream.into_iter();
  // CPSR low bits are: `I F T MMMMM`, and T must always be left as 0.

  // processor modes bits reference:
  // https://developer.arm.com/documentation/ddi0406/c/System-Level-Architecture/The-System-Level-Programmers--Model/ARM-processor-modes-and-ARM-core-registers/ARM-processor-modes?lang=en
  let mode =
    match stream_iter.next().expect("too few tokens").to_string().as_str() {
      "User" | "usr" => "10000",
      "FIQ" | "fiq" => "10001",
      "IRQ" | "irq" => "10010",
      "Supervisor" | "svc" => "10011",
      "System" | "sys" => "11111",
      other => {
        panic!("First argument must be a valid cpu mode name, got `{other}`")
      }
    };
  assert_eq!(
    stream_iter.next().expect("too few tokens").to_string(),
    ",",
    "must have comma after the first arg"
  );

  assert_eq!(
    stream_iter.next().expect("too few tokens").to_string(),
    "irq_masked",
    "second setting must be `irq_masked`"
  );
  assert_eq!(
    stream_iter.next().expect("too few tokens").to_string(),
    "=",
    "after `irq_masked` must be a `=`"
  );
  let i = get_bool(&stream_iter.next().expect("too few tokens"))
    .expect("`irq_masked` must be set as `true` or `false`") as u8;
  assert_eq!(
    stream_iter.next().expect("too few tokens").to_string(),
    ",",
    "must have comma after the second arg"
  );

  assert_eq!(
    stream_iter.next().expect("too few tokens").to_string(),
    "fiq_masked",
    "third setting must be `fiq_masked`"
  );
  assert_eq!(
    stream_iter.next().expect("too few tokens").to_string(),
    "=",
    "after `fiq_masked` must be a `=`"
  );
  let f = get_bool(&stream_iter.next().expect("too few tokens"))
    .expect("`fiq_masked` must be set as `true` or `false`") as u8;
  assert!(stream_iter.next().is_none(), "too many tokens");
  TokenStream::from_iter(Some(TokenTree::Literal(Literal::string(&format!(
    "msr CPSR_c, #0b{i}{f}0{mode}"
  )))))
}
