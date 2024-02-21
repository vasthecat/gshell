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
        section: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        owner: Option<String>,
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
                    f.write(&owner_conf.into_bytes()).unwrap();

                    let mut f = File::create(repo_path.join("description")).unwrap();
                    f.write(&description.into_bytes()).unwrap();

                    let mut f = File::create(repo_path.join("cgitrc")).unwrap();
                    f.write(&section_conf.into_bytes()).unwrap();

                    let mut f = File::create(repo_path.join("hooks/post-update")).unwrap();
                    f.write(&post_update.into_bytes()).unwrap();

                    repo
                }
                Err(e) => panic!("failed to init: {}", e),
            };
            println!("Initialized repo '{}.git'", name);
        }
        Commands::Rename { oldname, newname } => {
            let old_path = format!("{repos}/{oldname}.git");
            let old_path = Path::new(&old_path);
            let new_path = format!("{repos}/{newname}.git");
            let new_path = Path::new(&new_path);
            if new_path.exists() {
                println!("Repo with new name already exists");
                return;
            }
            if !old_path.exists() {
                println!("Repo with name {oldname} doesn't exist");
                return;
            }
            let post_update = format!(
                "#!/bin/sh\nchmod g+w -R {} 2> /dev/null",
                new_path.display()
            );
            std::fs::rename(old_path, new_path).unwrap();
            let mut f = File::open(new_path.join("hooks/post-update")).unwrap();
            f.write(&post_update.into_bytes()).unwrap();
        }
        Commands::Remove { name } => {
            let repo_path = format!("{repos}/{name}.git");
            let repo_path = Path::new(&repo_path);
            if !repo_path.exists() {
                println!("Repo with that name doesn't exist");
                return;
            }
            std::fs::remove_dir_all(repo_path).unwrap();
        }
        Commands::Change {
            name,
            section,
            description,
            owner,
        } => {
            let repo_path = format!("{repos}/{name}.git");
            let repo_path = Path::new(&repo_path);
            if !repo_path.exists() {
                println!("Repo with that name doesn't exist");
                return;
            }

            if let Some(section) = section {
                let section_conf = format!("section={}", section);
                let mut f = File::create(repo_path.join("cgitrc")).unwrap();
                f.write(&section_conf.into_bytes()).unwrap();
            }

            if let Some(description) = description {
                let mut f = File::create(repo_path.join("description")).unwrap();
                f.write(&description.into_bytes()).unwrap();
            }

            if let Some(owner) = owner {
                let owner_conf = format!("\n[gitweb]\n\towner = {}", owner);
                let mut f = File::create(repo_path.join("config")).unwrap();
                f.write(&owner_conf.into_bytes()).unwrap();
            }
        }
        Commands::List => unimplemented!(),
    }
}
