//! CLI entrypoint for `afri-sms-skills`.
//!
//! Scaffolds `.agent/skills/afri-sms/` into the current working directory,
//! enabling LLM coding agents to immediately use `afri-sms`.

use std::env;
use std::path::PathBuf;
use std::process;

fn print_help() {
    println!(
        r#"afri-sms-skills CLI - Autonomous agent skill scaffolder

USAGE:
    afri-sms-skills [OPTIONS] [COMMAND]

COMMANDS:
    init             Scaffold the afri-sms skill into .agent/skills/afri-sms/ (default)

OPTIONS:
    -d, --dir <PATH> Specify a custom target directory (default: .agent/skills/afri-sms)
    -s, --schema     Print the LLM function calling JSON schema to stdout and exit
        --docs       Print the full agent skill documentation (SKILL.md) to stdout
        --env        Print the .env.example template to stdout
    -f, --force      Overwrite existing skill files if already present
    -h, --help       Print help information
    -V, --version    Print version information
"#
    );
}

fn print_version() {
    println!("afri-sms-skills {}", env!("CARGO_PKG_VERSION"));
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut target_dir: Option<PathBuf> = None;
    let mut force = false;
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" | "help" => {
                print_help();
                return;
            }
            "-V" | "--version" | "version" => {
                print_version();
                return;
            }
            "-s" | "--schema" => {
                println!("{}", afri_sms_skills::schema_json().trim());
                return;
            }
            "--docs" => {
                println!("{}", afri_sms_skills::skill_markdown().trim());
                return;
            }
            "--env" => {
                println!("{}", afri_sms_skills::env_example().trim());
                return;
            }
            "-f" | "--force" => {
                force = true;
                i += 1;
            }
            "-d" | "--dir" => {
                if i + 1 < args.len() {
                    target_dir = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    eprintln!("Error: --dir requires a path argument.");
                    process::exit(1);
                }
            }
            "init" | "install" => {
                i += 1;
            }
            other if other.starts_with('-') => {
                eprintln!(
                    "Error: Unknown option '{}'. Run with --help for usage.",
                    other
                );
                process::exit(1);
            }
            positional => {
                target_dir = Some(PathBuf::from(positional));
                i += 1;
            }
        }
    }

    let destination = match target_dir {
        Some(custom) => custom,
        None => {
            let cwd = match env::current_dir() {
                Ok(dir) => dir,
                Err(err) => {
                    eprintln!(
                        "Error: Failed to determine current working directory: {}",
                        err
                    );
                    process::exit(1);
                }
            };
            cwd.join(".agent").join("skills").join("afri-sms")
        }
    };

    if let Err(err) = afri_sms_skills::scaffold(&destination, force) {
        eprintln!("Error: {}", err);
        process::exit(1);
    }

    println!(
        "\nSuccessfully scaffolded afri-sms skill into: {}",
        destination.display()
    );
}
