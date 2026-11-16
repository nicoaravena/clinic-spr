use std::process::Command;

/// The exact output of a run. Update it when a step changes what the clinic does:
/// `cargo run -p clinic > crates/clinic/tests/expected-output.txt`
const EXPECTED: &str = include_str!("expected-output.txt");

#[test]
fn a_run_prints_the_expected_day_and_succeeds() {
    let output = Command::new(env!("CARGO_BIN_EXE_clinic"))
        .output()
        .expect("the clinic binary should start");

    assert!(
        output.status.success(),
        "exit status {}, stderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr),
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), EXPECTED);
}
