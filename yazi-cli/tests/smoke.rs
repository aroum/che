use std::process::Command;

#[test]
fn test_cli_versions_and_help() {
	let target_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
		.parent()
		.unwrap()
		.join("target/debug");

	let che_bin = target_dir.join("che");
	let ch_bin = target_dir.join("ch");

	if !che_bin.exists() || !ch_bin.exists() {
		return;
	}

	// 1. Check che --version
	let out = Command::new(&che_bin).arg("--version").output().expect("run che --version");
	assert!(out.status.success(), "che --version exited with non-zero");
	let stdout = String::from_utf8_lossy(&out.stdout);
	assert!(
		stdout.starts_with("Che"),
		"Expected che --version to start with 'Che', got:\n{stdout}"
	);

	// 2. Check ch --version
	let out = Command::new(&ch_bin).arg("--version").output().expect("run ch --version");
	assert!(out.status.success(), "ch --version exited with non-zero");
	let stdout = String::from_utf8_lossy(&out.stdout);
	assert!(
		stdout.starts_with("Ch"),
		"Expected ch --version to start with 'Ch', got:\n{stdout}"
	);

	// 3. Check ch --help
	let out = Command::new(&ch_bin).arg("--help").output().expect("run ch --help");
	assert!(out.status.success());
	let stdout = String::from_utf8_lossy(&out.stdout);
	assert!(stdout.contains("archive"), "Expected ch --help to contain 'archive'");
	assert!(stdout.contains("multirename"), "Expected ch --help to contain 'multirename'");

	// 4. Check ch archive --help
	let out = Command::new(&ch_bin).args(["archive", "--help"]).output().expect("run ch archive --help");
	assert!(out.status.success());

	// 5. Check ch multirename --help
	let out =
		Command::new(&ch_bin).args(["multirename", "--help"]).output().expect("run ch multirename --help");
	assert!(out.status.success());

	// 6. Check ch env
	let out = Command::new(&ch_bin).arg("env").output().expect("run ch env");
	assert!(out.status.success());
	let stdout = String::from_utf8_lossy(&out.stdout);
	assert!(
		stdout.contains("Config") || stdout.contains("Dependencies"),
		"Expected ch env diagnostic output"
	);
}
