use clap::{Parser, Subcommand};
use git2::Repository;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(long)]
        name: String,
        #[arg(long, default_value = "")]
        section: String,
        #[arg(long, default_value = "")]
        description: String,
        #[arg(long, default_value = "")]
        owner: String,
    },
    Rename {
        #[arg(long)]
        oldname: String,
        #[arg(long)]
        newname: String,
    },
    Remove {
        #[arg(long)]
        name: String,
    },
    Change {
        #[arg(long)]
        name: String,
        #[arg(long)]
        section: String,
        #[arg(long)]
        description: String,
        #[arg(long)]
        owner: String,
    },
    List,
}

fn main() {
    let cli = Cli::parse();
    let home = std::env::var("HOME").expect("Couldn't get $HOME variable");
    let repos = format!("{home}/repos");
    match cli.command {
        Commands::Init {
            name,
            section,
            description,
            owner,
        } => {
            let repo_path = format!("{repos}/{name}.git");
            let repo_path = Path::new(&repo_path);
            if repo_path.exists() {
                println!("Repo with that name already exists");
                return;
            }
            let owner_conf = format!("\n[gitweb]\n\towner = {}", owner);
            let post_update = format!(
                "#!/bin/sh\nchmod g+w -R {} 2> /dev/null",
                repo_path.display()
            );
            let section_conf = format!("section={}", section);
            let _repo = match Repository::init_bare(&repo_path) {
                Ok(repo) => {
                    let mut f = File::create(repo_path.join("config")).unwrap();
                    let _ = f.write(&owner_conf.into_bytes());

                    let mut f = File::create(repo_path.join("description")).unwrap();
                    let _ = f.write(&description.into_bytes());

                    let mut f = File::create(repo_path.join("cgitrc")).unwrap();
                    let _ = f.write(&section_conf.into_bytes());

                    let mut f = File::create(repo_path.join("hooks/post-update")).unwrap();
                    let _ = f.write(&post_update.into_bytes());

                    repo
                }
                Err(e) => panic!("failed to init: {}", e),
            };
            println!("Initialized repo '{}.git'", name);
        }
        _ => {}
    }
}
