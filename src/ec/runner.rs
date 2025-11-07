use std::env;
use std::fmt::Display;
use std::time::Instant;

pub const ANSI_BOLD: &str = "\x1b[1m";
pub const ANSI_RESET: &str = "\x1b[0m";
pub const ANSI_GREEN: &str = "\x1b[32m";
pub const ANSI_RED: &str = "\x1b[31m";

pub fn run_part<T: Display>(func: impl Fn(&str) -> Option<T>, input: &str, quest: u8, part: u8) {
    // Print result inline
    if input.is_empty() {
        println!("Part {}: -", part);
        return;
    }

    let timer = Instant::now();
    let result = func(input);
    let duration = timer.elapsed();

    match &result {
        Some(answer) => {
            let answer_str = answer.to_string();
            if answer_str.contains('\n') {
                println!("Part {}: (multiline) ({:?})", part, duration);
                println!("{}", answer_str);
            } else {
                print!(
                    "Part {}: {}{}{} ({:?})",
                    part, ANSI_BOLD, answer_str, ANSI_RESET, duration
                );

                // Check if we should submit and get response inline
                if let Some(submission_info) = check_and_submit(&result, quest, part) {
                    print!(" - {}", submission_info);
                }

                println!();
            }
        }
        None => {
            println!("Part {}: -", part);
        }
    }
}

fn check_and_submit<T: Display>(result: &Option<T>, quest: u8, part: u8) -> Option<String> {
    let args: Vec<String> = env::args().collect();

    // Check if we should submit AND if this is the part to submit
    let should_submit = args
        .iter()
        .position(|x| x == "--submit")
        .and_then(|idx| args.get(idx + 1))
        .and_then(|s| s.parse::<u8>().ok())
        .map(|submit_part| submit_part == part)
        .unwrap_or(false);

    if !should_submit {
        return None;
    }

    let result = result.as_ref()?;

    match super::client::Client::new() {
        Ok(client) => match client.submit_answer(quest, part, &result.to_string()) {
            Ok(response) => format_submission_response(&response),
            Err(e) => Some(format!(
                "{}✗ Submission failed: {}{}",
                ANSI_RED, e, ANSI_RESET
            )),
        },
        Err(e) => Some(format!("{}✗ Client error: {}{}", ANSI_RED, e, ANSI_RESET)),
    }
}

fn format_submission_response(response: &str) -> Option<String> {
    // Try to parse as JSON
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(response) {
        let correct = json
            .get("correct")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if correct {
            let mut parts = vec![format!("{}✓ Correct answer!{}", ANSI_GREEN, ANSI_RESET)];

            if let Some(global_place) = json.get("globalPlace").and_then(|v| v.as_i64()) {
                if global_place > 0 {
                    parts.push(format!("Global rank: #{}", global_place));
                }
            }

            Some(parts.join(" - "))
        } else {
            let mut msg = format!("{}✗ Incorrect answer{}", ANSI_RED, ANSI_RESET);

            let length_correct = json
                .get("lengthCorrect")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !length_correct {
                msg.push_str(" (wrong length)");
            }

            Some(msg)
        }
    } else {
        // Fallback to raw response
        Some(response.to_string())
    }
}
