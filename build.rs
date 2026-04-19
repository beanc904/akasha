fn main() {
    if let Some(git) = get_git_hash() {
        // Inject `GIT_HASH` into env!()
        println!("cargo:rustc-env=GIT_HASH={}", git);
    }

    // Recompile `build.rs` when dir `.git` changed.
    println!("cargo:rerun-if-changed=.git");
}

fn get_git_hash() -> Option<String> {
    use std::process::Command;

    let branch = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output();
    if let Ok(branch_output) = branch {
        let branch_string = String::from_utf8_lossy(&branch_output.stdout);
        let commit = Command::new("git")
            .arg("rev-parse")
            .arg("--verify")
            .arg("HEAD")
            .output();
        if let Ok(commit_output) = commit {
            let commit_string = String::from_utf8_lossy(&commit_output.stdout);

            Some(format!(
                "{}, {}",
                branch_string.lines().next().unwrap_or(""),
                commit_string.lines().next().unwrap_or("")
            ))
        } else {
            panic!("Cannot get git commit: {}", commit.unwrap_err());
        }
    } else {
        panic!("Cannot get git branch: {}", branch.unwrap_err());
    }
}
