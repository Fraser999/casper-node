use assert_cmd::{crate_name, Command};
use predicates::{boolean::PredicateBooleanExt, str};

fn command(subcommand: &str) -> Command {
    let mut command = Command::cargo_bin(crate_name!()).unwrap();
    command.arg(subcommand);
    command
}

#[test]
fn should_print_help() {
    let matches_help = || str::starts_with("Casper client").and(str::contains("USAGE:"));
    // If run with `--help`, `-h` or `help`, should exit with success and print help.
    command("-h").assert().success().stdout(matches_help());
    command("--help").assert().success().stdout(matches_help());
    command("help").assert().success().stdout(matches_help());

    // If run with no args, should exit with failure and print help.
    command("").assert().failure().stdout(matches_help());
}

#[test]
fn should_put_deploy() {
    let inputs = vec![
        vec!["-h"],
        vec!["--node-address", "http://localhost:40101"],
        vec!["--node-address", "http://localhost:40101"],
    ];

    command("put-deploy")
        .arg("-h")
        .assert()
        .success()
        .stdout(str::starts_with("casper-client-put-deploy").and(str::contains("USAGE:")));

    command("put-deploy")
        .arg("-h")
        .assert()
        .success()
        .stdout(str::starts_with("casper-client-put-deploy").and(str::contains("USAGE:")));
}
