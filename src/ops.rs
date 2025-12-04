use crate::config::RepoConfig;
use std::path::Path;
use std::process::Stdio;
use std::time::Instant;
use tokio::{fs, process::Command};
use tracing::{error, info, debug, warn, instrument};

#[instrument(skip(config), fields(path = %config.path))]
pub async fn perform_update(name: String, config: RepoConfig) {
    let start = Instant::now();
    info!(repo = %name, "Starting deployment");
    
    match update_logic(&name, &config).await {
        Ok(()) => {
            let duration = start.elapsed();
            info!(
                repo = %name,
                duration_ms = duration.as_millis(),
                "Deployment completed successfully"
            );
        }
        Err(e) => {
            let duration = start.elapsed();
            error!(
                repo = %name,
                duration_ms = duration.as_millis(),
                error = %e,
                "Deployment failed"
            );
        }
    }
}

async fn update_logic(name: &str, config: &RepoConfig) -> std::io::Result<()> {
    let path = Path::new(&config.path);

    // 1. Git Operations
    if path.exists() && path.join(".git").exists() {
        debug!(repo = name, path = %config.path, "Repository exists, pulling changes");
        run_command(name, "git", &["pull"], Some(path)).await?;
    } else {
        debug!(repo = name, path = %config.path, "Repository not found, cloning");
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
            debug!(repo = name, parent = ?parent, "Created parent directories");
        }
        let url = format!("git@github.com:{}.git", name);
        run_command(name, "git", &["clone", &url, config.path.as_str()], None).await?;
    }

    // 2. Deploy Script
    if let Some(cmd_str) = &config.deploy_command {
        info!(repo = name, command = cmd_str, "Executing deploy command");
        let parts: Vec<&str> = cmd_str.split_whitespace().collect();
        if let Some((prog, args)) = parts.split_first() {
            run_command(name, prog, args, Some(path)).await?;
        }
    } else {
        debug!(repo = name, "No deploy command configured");
    }

    Ok(())
}

async fn run_command(repo: &str, program: &str, args: &[&str], cwd: Option<&Path>) -> std::io::Result<()> {
    let cmd_str = format!("{} {}", program, args.join(" "));
    debug!(repo = repo, command = %cmd_str, cwd = ?cwd, "Executing command");
    
    let start = Instant::now();
    let mut cmd = Command::new(program);
    cmd.args(args);
    
    if let Some(path) = cwd {
        cmd.current_dir(path);
    }

    // Capture output for logging
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let output = cmd.output().await?;
    let duration = start.elapsed();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    if !stdout.is_empty() {
        debug!(repo = repo, command = %cmd_str, "stdout: {}", stdout.trim());
    }
    if !stderr.is_empty() {
        warn!(repo = repo, command = %cmd_str, "stderr: {}", stderr.trim());
    }
    
    if !output.status.success() {
        error!(
            repo = repo,
            command = %cmd_str,
            exit_code = output.status.code(),
            duration_ms = duration.as_millis(),
            "Command failed"
        );
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other, 
            format!("{} exited with code {:?}", program, output.status.code())
        ));
    }
    
    debug!(
        repo = repo,
        command = %cmd_str,
        duration_ms = duration.as_millis(),
        "Command completed successfully"
    );
    Ok(())
}