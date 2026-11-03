//! CLI entrypoint for `afri-sms-skills`.
//!
//! Scaffolds `.agent/skills/afri-sms/` into the current working directory,
//! enabling LLM coding agents to immediately use `afri-sms`.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

const SKILL_DOC: &str = include_str!("../assets/SKILL.md");
const SCHEMA_DOC: &str = include_str!("../assets/schema.json");

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
                println!("{}", SCHEMA_DOC.trim());
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

    if let Err(err) = scaffold_skill(&destination, force) {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}

fn scaffold_skill(dest: &Path, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(dest)?;

    let skill_path = dest.join("SKILL.md");
    let schema_path = dest.join("schema.json");

    if skill_path.exists() && !force {
        println!(
            "Note: {} already exists. Use --force to overwrite.",
            skill_path.display()
        );
    } else {
        fs::write(&skill_path, SKILL_DOC)?;
        println!("✓ Created {}", skill_path.display());
    }

    if schema_path.exists() && !force {
        println!(
            "Note: {} already exists. Use --force to overwrite.",
            schema_path.display()
        );
    } else {
        fs::write(&schema_path, SCHEMA_DOC)?;
        println!("✓ Created {}", schema_path.display());
    }

    println!(
        "\nSuccessfully scaffolded afri-sms skill into: {}",
        dest.display()
    );
    Ok(())
}
