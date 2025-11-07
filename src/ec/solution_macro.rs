use std::{env, fs};

/// Helper function that reads an input file to a string.
/// Returns an empty string if the file doesn't exist.
#[must_use]
pub fn read_input_file(quest: u8, part: u8) -> String {
    let cwd = env::current_dir().unwrap();
    let filepath = cwd
        .join("inputs")
        .join("notes")
        .join(format!("{:02}-{}.txt", quest, part));

    match fs::read_to_string(&filepath) {
        Ok(content) => content,
        Err(_) => String::new(),
    }
}

/// Helper function that reads an example file to a string.
#[must_use]
pub fn read_example_file(quest: u8, part: u8) -> String {
    let cwd = env::current_dir().unwrap();
    let filepath = cwd
        .join("inputs")
        .join("examples")
        .join(format!("{:02}-{}.txt", quest, part));
    fs::read_to_string(filepath).expect("could not open example file")
}

/// Creates the solution macro for quest binaries
#[macro_export]
macro_rules! solution {
    ($quest:expr) => {
        $crate::solution!(@impl $quest, [part_one, 1] [part_two, 2] [part_three, 3]);
    };
    ($quest:expr, 1) => {
        $crate::solution!(@impl $quest, [part_one, 1]);
    };
    ($quest:expr, 2) => {
        $crate::solution!(@impl $quest, [part_two, 2]);
    };
    ($quest:expr, 3) => {
        $crate::solution!(@impl $quest, [part_three, 3]);
    };

    (@impl $quest:expr, $( [$func:expr, $part:expr] )*) => {
        const QUEST: u8 = $quest;

        fn main() {
            use ec::{run_part, read_input_file};
            $(
                let input = read_input_file(QUEST, $part);
                run_part($func, &input, QUEST, $part);
            )*

            println!("");
        }
    };
}
