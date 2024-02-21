use clap::{Parser, Subcommand};
use git2::Repository;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Parser)]
struct ShellCli {
    #[arg(short)]
    command: String,
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize repo with optional section, description or owner
    Init {
        /// Name of the new repo (this should be unique)
        #[arg(long)]
        name: String,
        /// Section of the repo (used in cgit)
        #[arg(long, default_value = "", num_args = 1..)]
        section: Vec<String>,
        /// Detailed description of the repo
        #[arg(long, default_value = "", num_args = 1..)]
        description: Vec<String>,
        /// Owner of the repo (used in gitweb or cgit)
        #[arg(long, default_value = "", num_args = 1..)]
        owner: Vec<String>,
    },
    /// Rename repo
    Rename {
        /// Name of the repo to be renamed
        #[arg(long)]
        oldname: String,
        /// New name of the repo (should be unique)
        #[arg(long)]
        newname: String,
    },
    /// Remove repo
    Remove {
        /// Name of the repo to be removed
        #[arg(long)]
        name: String,
    },
    /// Change section, description or owner of a given repo
    Change {
        /// Name of the repo (mandatory field)
        #[arg(long)]
        name: String,
        /// Change section of the repo to given value
        #[arg(long, num_args = 1..)]
        section: Option<Vec<String>>,
        /// Change description of the repo to given value
        #[arg(long, num_args = 1..)]
        description: Option<Vec<String>>,
        /// Change owner of the repo to given value
        #[arg(long, num_args = 1..)]
        owner: Option<Vec<String>>,
    },
    /// List all repos
    List,
}

fn main() {
    let program_name = std::env::args().nth(0).unwrap();
    let cmdline = match ShellCli::try_parse() {
        Ok(shellcli) => {
            let mut line = shellcli
                .command
                .split(" ")
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
            let mut newline = vec![program_name];
            newline.append(&mut line);
            newline
        }
        Err(_) => {
            Cli::parse_from([program_name, "--help".to_string()]);
            return;
        }
    };

    let cli = Cli::parse_from(cmdline);
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
            let owner_conf = format!("\n[gitweb]\n\towner = {}", owner.join(" "));
            let post_update = format!(
                "#!/bin/sh\nchmod g+w -R {} 2> /dev/null",
                repo_path.display()
            );
            let section_conf = format!("section={}", section.join(" "));
            let _repo = match Repository::init_bare(&repo_path) {
                Ok(repo) => {
                    let mut f = File::create(repo_path.join("config")).unwrap();
                    f.write(&owner_conf.into_bytes()).unwrap();

                    let mut f = File::create(repo_path.join("description")).unwrap();
                    f.write(&description.join(" ").into_bytes()).unwrap();

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
            if !old_path.exists() {
                println!("Repo with name {oldname} doesn't exist");
                return;
            }
            if new_path.exists() {
                println!("Repo with new name already exists");
                return;
            }
            let post_update = format!(
                "#!/bin/sh\nchmod g+w -R {} 2> /dev/null",
                new_path.display()
            );
            std::fs::rename(old_path, new_path).unwrap();
            let mut f = File::create(new_path.join("hooks/post-update")).unwrap();
            f.write(&post_update.into_bytes()).unwrap();
            println!("Repo '{oldname}.git' renamed to '{newname}.git'");
        }
        Commands::Remove { name } => {
            let repo_path = format!("{repos}/{name}.git");
            let repo_path = Path::new(&repo_path);
            if !repo_path.exists() {
                println!("Repo with that name doesn't exist");
                return;
            }
            std::fs::remove_dir_all(repo_path).unwrap();
            println!("Repo '{name}.git' removed");
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
                let section_conf = format!("section={}", section.join(" "));
                let mut f = File::create(repo_path.join("cgitrc")).unwrap();
                f.write(&section_conf.into_bytes()).unwrap();
            }

            if let Some(description) = description {
                let mut f = File::create(repo_path.join("description")).unwrap();
                f.write(&description.join(" ").into_bytes()).unwrap();
            }

            if let Some(owner) = owner {
                let owner_conf = format!("\n[gitweb]\n\towner = {}", owner.join(" "));
                let mut f = File::create(repo_path.join("config")).unwrap();
                f.write(&owner_conf.into_bytes()).unwrap();
            }

            println!("Changed config of '{name}.git' repo");
        }
        Commands::List => unimplemented!(),
    }
}
