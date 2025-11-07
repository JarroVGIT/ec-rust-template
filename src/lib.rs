pub mod ec;

use std::fs;
use std::path::PathBuf;

pub use ec::{Client, Quest, read_example_file, read_input_file, run_part};

/// Determines which part to scaffold based on existing input files
pub fn determine_next_part(quest: u8) -> u8 {
    let inputs_dir = PathBuf::from("inputs/notes");

    for part in 1..=3 {
        let file_path = inputs_dir.join(format!("{:02}-{}.txt", quest, part));
        if !file_path.exists() {
            return part;
        }
    }

    // If all exist, default to 1
    1
}

/// Creates the directory structure and files for a quest
pub fn scaffold_quest(quest: u8, part: Option<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let part = part.unwrap_or_else(|| determine_next_part(quest));

    // Try to download input first - if this fails, don't create any files
    let input_content = download_input(quest, part)?;

    // Create directories
    fs::create_dir_all("src/bin")?;
    fs::create_dir_all("inputs/notes")?;
    fs::create_dir_all("inputs/examples")?;

    // Create quest file from template if it doesn't exist
    let quest_file = PathBuf::from(format!("src/bin/quest_{:02}.rs", quest));
    if !quest_file.exists() {
        let template = include_str!("./template.txt");
        let content = template.replace("%QUEST_NUMBER%", &quest.to_string());
        fs::write(&quest_file, content)?;
        println!("Created [\x1b[0;32m {} \x1b[0m]", quest_file.display());
    } else {
        println!("Quest file already exists: {}", quest_file.display());
    }

    // Create input file
    let input_file = PathBuf::from(format!("inputs/notes/{:02}-{}.txt", quest, part));
    if !input_file.exists() {
        fs::write(&input_file, input_content)?;
        println!("Created [\x1b[0;32m {} \x1b[0m]", input_file.display());
    } else {
        println!("Input file already exists: {}", input_file.display());
    }

    // Create example file
    let example_file = PathBuf::from(format!("inputs/examples/{:02}-{}.txt", quest, part));
    if !example_file.exists() {
        fs::write(&example_file, "")?;
        println!("Created [\x1b[0;32m {} \x1b[0m]", example_file.display());
    } else {
        println!("Example file already exists: {}", example_file.display());
    }

    println!("\nScaffolding complete for Quest {} Part {}", quest, part);
    Ok(())
}

/// Downloads and decrypts input for a quest part
fn download_input(quest: u8, part: u8) -> Result<String, Box<dyn std::error::Error>> {
    let client = ec::Client::new()?;
    let input = client.fetch_and_decrypt_input(quest, part)?;
    Ok(input)
}

/// Runs a quest solution
pub fn solve_quest(
    quest: u8,
    part: Option<u8>,
    submit: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Stdio;

    let quest_str = format!("{:02}", quest);

    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("run")
        .arg("--release")
        .arg("--bin")
        .arg(format!("quest_{}", quest_str))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    // Add -- separator before custom args
    if submit {
        if let Some(p) = part {
            cmd.arg("--");
            cmd.arg("--submit");
            cmd.arg(p.to_string());
        } else {
            return Err("Must specify a part number to submit".into());
        }
    }

    let status = cmd.status()?;

    if !status.success() {
        return Err("Quest execution failed".into());
    }

    Ok(())
}
