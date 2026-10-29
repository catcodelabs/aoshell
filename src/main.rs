use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

fn print_help() {
    println!("{} v{} - {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_DESCRIPTION"));
    println!("\nUsage:");
    println!("  aoshell <script_name> [args...] Run a custom script with timing");
    println!("  aoshell init                    Initialize scripts directory");
    println!("  aoshell list                    List all available scripts and descriptions");
    println!("  aoshell new <name> [lang]       Create a new script template (default: bash)");
    println!("  aoshell edit <name>             Edit a script using your editor");
    println!("  aoshell rename <old> <new>      Rename an existing script");
    println!("  aoshell remove <name>           Delete an existing script");
    println!("  aoshell which <name>            Print absolute path of a script");
    println!("  aoshell cat <name>              Print script source code");
    println!("  aoshell args <name>             Print documented arguments for a script");
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
        "init" => handle_init(&scripts_dir),
        "list" => handle_list(&scripts_dir),
        "new" => {
            if args.len() < 3 {
                eprintln!("Error: Missing script name for 'new'");
                eprintln!("Usage: aoshell new <script_name> [lang]");
                std::process::exit(1);
            }
            let script_name = &args[2];
            let lang = args.get(3).map(|s| s.as_str()).unwrap_or("bash");
            handle_new(&scripts_dir, script_name, lang);
        }
        "edit" => {
            if args.len() < 3 {
                eprintln!("Error: Missing script name for 'edit'");
                eprintln!("Usage: aoshell edit <script_name>");
                std::process::exit(1);
            }
            handle_edit(&scripts_dir, &args[2]);
        }
        "rename" => {
            if args.len() < 4 {
                eprintln!("Error: Missing arguments for 'rename'");
                eprintln!("Usage: aoshell rename <old_name> <new_name>");
                std::process::exit(1);
            }
            handle_rename(&scripts_dir, &args[2], &args[3]);
        }
        "remove" | "rm" => {
            if args.len() < 3 {
                eprintln!("Error: Missing script name for 'remove'");
                eprintln!("Usage: aoshell remove <script_name>");
                std::process::exit(1);
            }
            handle_remove(&scripts_dir, &args[2]);
        }
        "which" => {
            if args.len() < 3 {
                eprintln!("Error: Missing script name for 'which'");
                eprintln!("Usage: aoshell which <script_name>");
                std::process::exit(1);
            }
            handle_which(&scripts_dir, &args[2]);
        }
        "cat" => {
            if args.len() < 3 {
                eprintln!("Error: Missing script name for 'cat'");
                eprintln!("Usage: aoshell cat <script_name>");
                std::process::exit(1);
            }
            handle_cat(&scripts_dir, &args[2]);
        }
        "args" => {
            if args.len() < 3 {
                eprintln!("Error: Missing script name for 'args'");
                eprintln!("Usage: aoshell args <script_name>");
                std::process::exit(1);
            }
            handle_args(&scripts_dir, &args[2]);
        }
        script_name => {
            let script_args = &args[2..];
            execute_script(script_name, script_args, &scripts_dir);
        }
    }
}

fn extract_interpreter(content: &str) -> String {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#!") {
            let parts: Vec<&str> = trimmed[2..].split_whitespace().collect();
            if let Some(first) = parts.first() {
                if *first == "/usr/bin/env" || *first == "/bin/env" {
                    if let Some(interp) = parts.get(1) {
                        return interp.to_string();
                    }
                } else if let Some(file_name) = PathBuf::from(first).file_name() {
                    if let Some(name_str) = file_name.to_str() {
                        return name_str.to_string();
                    }
                }
            }
        }
        break;
    }
    "unknown".to_string()
}

fn extract_metadata(content: &str) -> (String, String) {
    let mut description = String::new();
    let mut args_doc = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        let comment_stripped = if let Some(stripped) = trimmed.strip_prefix('#') {
            Some(stripped.trim())
        } else if let Some(stripped) = trimmed.strip_prefix("--") {
            Some(stripped.trim())
        } else if let Some(stripped) = trimmed.strip_prefix("//") {
            Some(stripped.trim())
        } else {
            None
        };

        if let Some(inner) = comment_stripped {
            if let Some(desc) = inner.strip_prefix("Description:") {
                if description.is_empty() {
                    description = desc.trim().to_string();
                }
            } else if let Some(args) = inner.strip_prefix("Args:") {
                if args_doc.is_empty() {
                    args_doc = args.trim().to_string();
                }
            }
        }
    }
    (description, args_doc)
}

fn handle_init(scripts_dir: &PathBuf) {
    if let Err(e) = fs::create_dir_all(scripts_dir) {
        eprintln!("Failed to create scripts directory: {}", e);
        std::process::exit(1);
    }
    println!("[✔] Created directory: {}", scripts_dir.display());
    println!("\nInitialization complete!");
}

fn handle_list(scripts_dir: &PathBuf) {
    println!("Available aoshell scripts in {}:\n", scripts_dir.display());
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
                    let path = entry.path();
                    let mut description = String::new();
                    let mut interpreter = "unknown".to_string();
                    
                    if let Ok(content) = fs::read_to_string(&path) {
                        let (desc, _) = extract_metadata(&content);
                        description = desc;
                        interpreter = extract_interpreter(&content);
                    }

                    if description.is_empty() {
                        println!("  - {} : {} : (No description)", name, interpreter);
                    } else {
                        println!("  - {} : {} : {}", name, interpreter, description);
                    }
                }
            }
        }
    }
}

fn handle_new(scripts_dir: &PathBuf, name: &str, lang: &str) {
    if !scripts_dir.exists() {
        fs::create_dir_all(scripts_dir).expect("Failed to create scripts directory");
    }

    let script_path = scripts_dir.join(name);

    if script_path.exists() {
        eprintln!("Error: Script '{}' already exists.", script_path.display());
        std::process::exit(1);
    }

    let (shebang, comment_prefix, body_example) = match lang.to_lowercase().as_str() {
        "lua" => (
            "#!/usr/bin/env lua".to_string(),
            "--",
            format!("print(\"Running script: {}\")", name),
        ),
        "python" | "py" => (
            "#!/usr/bin/env python3".to_string(),
            "#",
            format!("print(\"Running script: {}\")", name),
        ),
        "awk" => (
            "#!/usr/bin/awk -f".to_string(),
            "#",
            "BEGIN {\n    print \"Running script: ".to_string() + name + "\"\n}",
        ),
        "sh" => (
            "#!/bin/sh".to_string(),
            "#",
            format!("echo \"Running script: {}\"", name),
        ),
        "bash" => (
            "#!/usr/bin/env bash".to_string(),
            "#",
            format!("echo \"Running script: {}\"", name),
        ),
        other => (
            format!("#!/usr/bin/env {}", other),
            "#",
            "".to_string(),
        ),
    };

    let template = if body_example.is_empty() {
        format!(
            "{}\n{} Script: {}\n{} Description: \n{} Args: \n",
            shebang, comment_prefix, name, comment_prefix, comment_prefix
        )
    } else {
        format!(
            "{}\n{} Script: {}\n{} Description: \n{} Args: \n\n{}\n",
            shebang, comment_prefix, name, comment_prefix, comment_prefix, body_example
        )
    };

    fs::write(&script_path, template).expect("Failed to write script template");

    let metadata = fs::metadata(&script_path).expect("Failed to read metadata");
    let mut perms = metadata.permissions();
    perms.set_mode(perms.mode() | 0o755);
    fs::set_permissions(&script_path, perms).expect("Failed to set permissions");

    println!("[✔] Created new script with interpreter '{}' at: {}", lang, script_path.display());
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

fn handle_rename(scripts_dir: &PathBuf, old_name: &str, new_name: &str) {
    let old_path = scripts_dir.join(old_name);
    let new_path = scripts_dir.join(new_name);

    if !old_path.is_file() {
        eprintln!("Error: Script '{}' not found in {:?}", old_name, scripts_dir);
        std::process::exit(1);
    }

    if new_path.exists() {
        eprintln!("Error: Target script name '{}' already exists.", new_path.display());
        std::process::exit(1);
    }

    fs::rename(&old_path, &new_path).expect("Failed to rename script file");
    println!("[✔] Renamed script '{}' to '{}'", old_name, new_name);
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

fn handle_which(scripts_dir: &PathBuf, name: &str) {
    let script_path = scripts_dir.join(name);
    if !script_path.is_file() {
        eprintln!("Error: Script '{}' not found in {:?}", name, scripts_dir);
        std::process::exit(1);
    }
    println!("{}", script_path.display());
}

fn handle_cat(scripts_dir: &PathBuf, name: &str) {
    let script_path = scripts_dir.join(name);
    if !script_path.is_file() {
        eprintln!("Error: Script '{}' not found in {:?}", name, scripts_dir);
        std::process::exit(1);
    }
    match fs::read_to_string(&script_path) {
        Ok(contents) => print!("{}", contents),
        Err(e) => {
            eprintln!("Failed to read script: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_args(scripts_dir: &PathBuf, name: &str) {
    let script_path = scripts_dir.join(name);
    if !script_path.is_file() {
        eprintln!("Error: Script '{}' not found in {:?}", name, scripts_dir);
        std::process::exit(1);
    }

    let mut args_doc = String::new();
    if let Ok(content) = fs::read_to_string(&script_path) {
        let (_, args) = extract_metadata(&content);
        args_doc = args;
    }

    if args_doc.is_empty() {
        println!("No arguments documented for script '{}'.", name);
    } else {
        println!("Arguments for '{}': {}", name, args_doc);
    }
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

    let start_time = Instant::now();

    let status = Command::new(&script_path)
        .args(script_args)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("Failed to execute script: {}", e);
            std::process::exit(1);
        });

    let duration = start_time.elapsed();
    println!("\n[ℹ] Execution finished in {:.2?}", duration);

    std::process::exit(status.code().unwrap_or(1));
}