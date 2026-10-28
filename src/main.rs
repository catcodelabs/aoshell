use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

fn print_help() {
    println!("aoshell - A lightweight script dispatcher utility");
    println!("\nUsage:");
    println!("  cs <script_name> [args...]   Run a custom script");
    println!("  aoshell init                 Initialize directory and shell alias");
    println!("  aoshell list                 List all available scripts");
    println!("  aoshell new <name>           Create a new script template");
    println!("  aoshell edit <name>          Edit a script using your editor");
    println!("  aoshell remove <name>        Delete an existing script");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_help();
        std::process::exit(1);
    }

    let command = &args[1];

    let home = env::var("HOME").expect("HOME environment variable not set");
    let scripts_dir = PathBuf::from(&home).join(".config/aoshell/scripts");

    match command.as_str() {
        "init" => handle_init(&home, &scripts_dir),
        "list" => handle_list(&scripts_dir),
        "new" => {
            if args.len() < 3 {
                eprintln!("Error: Missing script name for 'new'");
                eprintln!("Usage: aoshell new <script_name>");
                std::process::exit(1);
            }
            handle_new(&scripts_dir, &args[2]);
        }
        "edit" => {
            if args.len() < 3 {
                eprintln!("Error: Missing script name for 'edit'");
                eprintln!("Usage: aoshell edit <script_name>");
                std::process::exit(1);
            }
            handle_edit(&scripts_dir, &args[2]);
        }
        "remove" | "rm" => {
            if args.len() < 3 {
                eprintln!("Error: Missing script name for 'remove'");
                eprintln!("Usage: aoshell remove <script_name>");
                std::process::exit(1);
            }
            handle_remove(&scripts_dir, &args[2]);
        }
        script_name => {
            let script_args = &args[2..];
            execute_script(script_name, script_args, &scripts_dir);
        }
    }
}

fn handle_init(home: &str, scripts_dir: &PathBuf) {
    if let Err(e) = fs::create_dir_all(scripts_dir) {
        eprintln!("Failed to create scripts directory: {}", e);
        std::process::exit(1);
    }
    println!("[✔] Created directory: {}", scripts_dir.display());

    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    
    let (rc_file, alias_line) = if shell.contains("fish") {
        let fish_config_dir = PathBuf::from(home).join(".config/fish");
        let _ = fs::create_dir_all(&fish_config_dir);
        (
            fish_config_dir.join("config.fish"),
            "\n# Aoshell alias\nalias cs=\"aoshell\"\n",
        )
    } else if shell.contains("zsh") {
        (
            PathBuf::from(home).join(".zshrc"),
            "\n# Aoshell alias\nalias cs=\"aoshell\"\n",
        )
    } else {
        (
            PathBuf::from(home).join(".bashrc"),
            "\n# Aoshell alias\nalias cs=\"aoshell\"\n",
        )
    };

    let content = fs::read_to_string(&rc_file).unwrap_or_default();
    if !content.contains("alias cs=") {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&rc_file)
            .expect("Failed to open shell config file");
        
        file.write_all(alias_line.as_bytes())
            .expect("Failed to write alias to shell config");
        println!("[✔] Added 'cs' alias to {}", rc_file.display());
    } else {
        println!("[ℹ] 'cs' alias already exists in {}", rc_file.display());
    }

    println!("\nInitialization complete! Restart your shell or run: source {}", rc_file.display());
}

fn handle_list(scripts_dir: &PathBuf) {
    println!("Available aoshell scripts in {}:", scripts_dir.display());
    if !scripts_dir.exists() {
        println!("  (Directory does not exist yet. Run 'aoshell init')");
        return;
    }

    let entries = match fs::read_dir(scripts_dir) {
        Ok(e) => e,
        Err(_) => {
            eprintln!("Failed to read scripts directory.");
            return;
        }
    };

    for entry in entries.flatten() {
        if let Ok(file_type) = entry.file_type() {
            if file_type.is_file() {
                if let Some(name) = entry.file_name().to_str() {
                    println!("  - {}", name);
                }
            }
        }
    }
}

fn handle_new(scripts_dir: &PathBuf, name: &str) {
    if !scripts_dir.exists() {
        fs::create_dir_all(scripts_dir).expect("Failed to create scripts directory");
    }

    let script_path = scripts_dir.join(name);

    if script_path.exists() {
        eprintln!("Error: Script '{}' already exists.", script_path.display());
        std::process::exit(1);
    }

    let template = "#!/usr/bin/env bash\n\n# Description: Auto-generated script\n\necho \"Running script: {}\"\n";
    fs::write(&script_path, template.replace("{}", name)).expect("Failed to write script template");

    let metadata = fs::metadata(&script_path).expect("Failed to read metadata");
    let mut perms = metadata.permissions();
    perms.set_mode(perms.mode() | 0o755);
    fs::set_permissions(&script_path, perms).expect("Failed to set permissions");

    println!("[✔] Created new script at: {}", script_path.display());
}

fn handle_edit(scripts_dir: &PathBuf, name: &str) {
    let script_path = scripts_dir.join(name);

    if !script_path.is_file() {
        eprintln!("Error: Script '{}' not found in {:?}", name, scripts_dir);
        std::process::exit(1);
    }

    let editor = env::var("VISUAL")
        .or_else(|_| env::var("EDITOR"))
        .unwrap_or_else(|_| "nano".to_string());

    let status = Command::new(&editor)
        .arg(&script_path)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("Failed to open editor '{}': {}", editor, e);
            std::process::exit(1);
        });

    std::process::exit(status.code().unwrap_or(1));
}

fn handle_remove(scripts_dir: &PathBuf, name: &str) {
    let script_path = scripts_dir.join(name);

    if !script_path.is_file() {
        eprintln!("Error: Script '{}' not found in {:?}", name, scripts_dir);
        std::process::exit(1);
    }

    fs::remove_file(&script_path).expect("Failed to delete script file");
    println!("[✔] Removed script: {}", script_path.display());
}

fn execute_script(target_script: &str, script_args: &[String], scripts_dir: &PathBuf) {
    let script_path = scripts_dir.join(target_script);

    if !script_path.is_file() {
        eprintln!("Error: Script '{}' not found in {:?}", target_script, scripts_dir);
        std::process::exit(127);
    }

    let metadata = fs::metadata(&script_path).expect("Failed to read script metadata");
    let mut permissions = metadata.permissions();
    if permissions.mode() & 0o100 == 0 {
        permissions.set_mode(permissions.mode() | 0o111);
        let _ = fs::set_permissions(&script_path, permissions);
    }

    let status = Command::new(&script_path)
        .args(script_args)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("Failed to execute script: {}", e);
            std::process::exit(1);
        });

    std::process::exit(status.code().unwrap_or(1));
}