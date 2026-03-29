use std::process::Command;

pub fn speak_word(text: &str) -> Result<(), String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("say")
            .arg(trimmed)
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("Failed to start macOS speech synthesis: {}", error))
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err("Text-to-speech is only supported on macOS.".to_string())
    }
}
