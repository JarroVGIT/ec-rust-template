# Everybody Codes Rust Template

A Rust template for solving [Everybody Codes](https://everybody.codes) challenges.

## Setup

1. Get your session cookie from your browser (look for `everybody-codes` cookie)
2. Save it to `~/.ec-session`:
   ```bash
   echo "your_session_cookie_value" > ~/.ec-session
   ```

3. Configure your event in `.cargo/config.toml`:
   ```toml
   [env]
   EC_SEED = " "     # Leave empty - will be auto-fetched on first run
   EC_EVENT = "2025"
   ```

The CLI will automatically fetch your seed from the API on first run if `EC_SEED` is empty or just whitespace. You can then add the printed seed value to the config to avoid fetching it on subsequent runs.

## Usage

### Scaffold a Quest

Creates the quest file, downloads and decrypts the input, and creates an empty example file:

```bash
cargo scaffold 1        # Auto-detects next part to scaffold
cargo scaffold 1 2      # Scaffold quest 1, part 2 specifically
```

This creates:
- `src/bin/quest_01.rs` - Your solution file
- `inputs/notes/01-2.txt` - Downloaded and decrypted input
- `inputs/examples/01-2.txt` - Empty example file (fill this yourself)

### Solve a Quest

Runs your solution in release mode:

```bash
cargo solve 1           # Run all parts of quest 1
cargo solve 1 2         # Run all parts of quest 1 and submit part 2
```

When you specify a part number, it automatically submits your answer to the API.

## Quest File Structure

Each quest file uses a macro that handles reading inputs and running your solution:

```rust
ec::solution!(1);

pub fn part_one(input: &str) -> Option<String> {
    // Your solution here
    Some("answer".to_string())
}

pub fn part_two(input: &str) -> Option<String> {
    None  // Return None if not implemented yet
}

pub fn part_three(input: &str) -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use ec::read_example_file;

    #[test]
    fn test_part_one() {
        let result = part_one(&read_example_file(1, 1));
        assert_eq!(result, Some("expected".to_string()));
    }
}
```

## How It Works

- **Auto-seed fetching**: Automatically fetches your seed from the API if not configured
- **Smart scaffolding**: Automatically detects which part to scaffold based on existing input files
- **Auto-download**: Fetches and decrypts inputs using your session cookie and seed
- **AES decryption**: Handles the encrypted input notes from the CDN
- **Simple timing**: Shows execution time for each part
- **Auto-submit**: When you specify a part number in `solve`, it submits your answer


## Notice
This template was heavily inspired by the work of [fspoettel](https://github.com/fspoettel/advent-of-code-rust), credits to him for his template! 