use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Loads environment variables from a .env file
///
/// This function tries to load environment variables from a .env file in the current directory.
/// It mimics the basic behavior of the dotenvy crate.
///
/// # Returns
///
/// * `Ok(())` if the .env file was successfully loaded or doesn't exist
/// * `Err` if there was an error reading or parsing the .env file
pub fn dotenv() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(".env");

    // If .env file doesn't exist, that's fine - just return Ok
    if !path.exists() {
        return Ok(());
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse KEY=VALUE pairs
        if let Some(pos) = line.find('=') {
            let key = &line[..pos];
            let value = &line[pos + 1..];

            // Remove quotes from value if present
            let value = if (value.starts_with('"') || value.starts_with('\''))
                && (value.ends_with('"') || value.ends_with('\''))
                && value.len() >= 2
            {
                &value[1..value.len() - 1]
            } else {
                value
            };

            unsafe { env::set_var(key, value) };
        }
    }

    Ok(())
}
