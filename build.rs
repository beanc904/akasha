use std::process::Command;

fn main() {
    if let Some(git) = get_git_hash() {
        // Inject `GIT_HASH` into env!()
        println!("cargo:rustc-env=GIT_HASH={}", git);
    }

    if let Some(time) = get_build_time() {
        println!("cargo:rustc-env=BUILD_TIME={}", time);
    }

    // Recompile `build.rs` when dir `.git` changed.
    println!("cargo:rerun-if-changed=.git");
}

fn get_git_hash() -> Option<String> {
    let branch = Command::new("git").arg("rev-parse").arg("--abbrev-ref").arg("HEAD").output();
    if let Ok(branch_output) = branch {
        let branch_string = String::from_utf8_lossy(&branch_output.stdout);
        let commit = Command::new("git").arg("rev-parse").arg("--verify").arg("HEAD").output();
        if let Ok(commit_output) = commit {
            let commit_string = String::from_utf8_lossy(&commit_output.stdout);

            Some(format!(
                "{}, {}",
                branch_string.lines().next().unwrap_or(""),
                commit_string.lines().next().unwrap_or("").get(..7).unwrap_or("")
            ))
        } else {
            panic!("Cannot get git commit: {}", commit.unwrap_err());
        }
    } else {
        panic!("Cannot get git branch: {}", branch.unwrap_err());
    }
}

fn get_build_time() -> Option<String> {
    let output = Command::new("date").arg("+%Y-%m-%d %H:%M:%S").output();
    if let Ok(time_string) = output {
        let build_time = String::from_utf8_lossy(&time_string.stdout)
            .lines()
            .next()
            .unwrap_or("")
            .to_string();
        Some(build_time)
    } else {
        None
    }
}
