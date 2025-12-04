use crate::config::RepoConfig;
use std::path::Path;
use std::process::Stdio;
use tokio::{fs, process::Command};
use tracing::{error, info};

pub async fn perform_update(name: String, config: RepoConfig) {
    if let Err(e) = update_logic(&name, &config).await {
        error!("[{}] Deployment failed: {}", name, e);
    }
}

async fn update_logic(name: &str, config: &RepoConfig) -> std::io::Result<()> {
    let path = Path::new(&config.path);

    // 1. Git Operations
    if path.exists() && path.join(".git").exists() {
        info!("[{}] Pulling changes...", name);
        run_command("git", &["pull"], Some(path)).await?;
    } else {
        info!("[{}] Cloning repo...", name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        let url = format!("git@github.com:{}.git", name);
        run_command("git", &["clone", &url, config.path.as_str()], None).await?;
    }

    // 2. Deploy Script
    if let Some(cmd_str) = &config.deploy_command {
        info!("[{}] Executing deploy command...", name);
        let parts: Vec<&str> = cmd_str.split_whitespace().collect();
        if let Some((prog, args)) = parts.split_first() {
            run_command(prog, args, Some(path)).await?;
        }
    }

    info!("[{}] Successfully updated.", name);
    Ok(())
}

async fn run_command(program: &str, args: &[&str], cwd: Option<&Path>) -> std::io::Result<()> {
    let mut cmd = Command::new(program);
    cmd.args(args);
    
    if let Some(path) = cwd {
        cmd.current_dir(path);
    }

    // Inherit stdio so logs show up in system journal
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    let status = cmd.status().await?;
    if !status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other, 
            format!("{} exited with error", program)
        ));
    }
    Ok(())
}