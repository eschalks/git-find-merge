use std::process::Command;

use anyhow::Result;
use clap::{App, Arg, ArgMatches};
use std::ffi::OsStr;
use std::collections::HashSet;

fn main() -> Result<()> {
    let args = parse_args();
    let range = create_commit_range(&args)?;

    let ancestry = execute_git("rev-list", [range.clone(), "--ancestry-path".to_string()])?;
    let first_parent = execute_git("rev-list", [range, "--first-parent".to_string()])?;
    let common_line = find_last_line_in_common(ancestry, first_parent);

    let output = get_output(&args, common_line)?;

    if let Some(output) = output {
        println!("{}", output);
    }

    Ok(())
}

fn get_output(args: &ArgMatches, hash: Option<String>) -> Result<Option<String>> {
    let output = match hash {
        Some(hash) => {
            let show_log = args.is_present("log");

            if show_log {
                Some(execute_git("show", [hash])?)
            }  else {
                Some(hash)
            }
        }

        None => None
    };

    Ok(output)
}

fn find_last_line_in_common(output1: String, output2: String) -> Option<String> {
    let line_set: HashSet<&str> = output2.lines().collect();

    output1.lines().rfind(|line| {
        line_set.contains(line)
    }).map(|line| line.to_string())
}

fn create_commit_range(args: &ArgMatches) -> Result<String> {
    let commit = args.value_of("commit").unwrap();

    let range_end = match args.value_of("branch") {
        Some(b) => {
            String::from(b)
        },
        None => (
            execute_git("show-ref", ["-s", "HEAD"])?
        )
    };

    Ok(format!("{}..{}", commit, range_end))
}

fn parse_args() -> ArgMatches<'static> {
    App::new("Find Merge")
        .version("1.0")
        .author("Erwin Schalks (erwin@schalks.org)")
        .about("Finds commit in which the specified hash was merged into the current branch.")
        .arg(
            Arg::with_name("commit")
                .required(true)
                .help("The hash of the commit to look for")
                .takes_value(true)

        )
        .arg(
            Arg::with_name("branch")
                .required(false)
                .help("The branch that was merged to")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("log")
                .long("log")
                .required(false)
                .help("Whether to show the entire log entry instead of just the hash.")
                .takes_value(false)
        )
        .get_matches()
}

fn execute_git<Args, S>(command: &str, args: Args) -> Result<String>
    where Args: IntoIterator<Item=S>,
          S: AsRef<OsStr>
{
    let output = Command::new("git")
        .arg(command)
        .args(args)
        .output()?;

    if !output.status.success() {
        let err = String::from_utf8(output.stderr)?;
        return Err(anyhow::Error::msg(format!("Git command failed with code {}: {}", output.status.code().unwrap(), err)));
    }

    Ok(String::from_utf8(output.stdout)?)
}

