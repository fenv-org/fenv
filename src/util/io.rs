use anyhow::Result;

/// Trait for reading user input from console
pub trait ConsoleInput {
    /// Reads a single character from input.
    /// Returns Ok(Some(char)) for a character, Ok(None) for Enter key,
    /// or Err for I/O errors.
    fn read_char(&mut self) -> Result<Option<char>>;
}

/// Real stdin implementation using crossterm raw mode
/// Reads a single key press without requiring Enter
pub struct StdInput;

impl StdInput {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StdInput {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsoleInput for StdInput {
    fn read_char(&mut self) -> Result<Option<char>> {
        use crossterm::{
            event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
            terminal,
        };

        // Enable raw mode to read single key press
        terminal::enable_raw_mode()?;

        let result = loop {
            if let Event::Key(KeyEvent {
                code, modifiers, ..
            }) = event::read()?
            {
                // Handle Ctrl+C for graceful exit
                if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                    break Err(anyhow::anyhow!("Interrupted by user"));
                }

                match code {
                    KeyCode::Enter => break Ok(None),
                    KeyCode::Char(c) => break Ok(Some(c)),
                    _ => continue, // Ignore other keys
                }
            }
        };

        // Restore terminal mode
        terminal::disable_raw_mode()?;

        // Print newline after input (since raw mode doesn't echo)
        println!();

        result
    }
}

/// Test implementation with predefined responses
#[derive(Default)]
pub struct BufferedInput {
    responses: Vec<Option<char>>,
    index: usize,
}

impl BufferedInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_responses(responses: Vec<Option<char>>) -> Self {
        Self { responses, index: 0 }
    }
}

impl ConsoleInput for BufferedInput {
    fn read_char(&mut self) -> Result<Option<char>> {
        if self.index < self.responses.len() {
            let response = self.responses[self.index];
            self.index += 1;
            Ok(response)
        } else {
            Ok(None) // Default to Enter (None) if no more responses
        }
    }
}

pub trait ConsoleOutput<OUT, ERR>
where
    OUT: std::io::Write,
    ERR: std::io::Write,
{
    fn stdout(&mut self) -> &mut OUT;
    fn stderr(&mut self) -> &mut ERR;
}

pub struct StdOutput {
    stdout: std::io::Stdout,
    stderr: std::io::Stderr,
}

impl Default for StdOutput {
    fn default() -> Self {
        Self {
            stdout: std::io::stdout(),
            stderr: std::io::stderr(),
        }
    }
}

impl StdOutput {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ConsoleOutput<std::io::Stdout, std::io::Stderr> for StdOutput {
    fn stdout(&mut self) -> &mut std::io::Stdout {
        &mut self.stdout
    }

    fn stderr(&mut self) -> &mut std::io::Stderr {
        &mut self.stderr
    }
}

#[derive(Default)]
pub struct BufferedOutput {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl BufferedOutput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stdout_to_string(&self) -> String {
        String::from_utf8(self.stdout.clone()).unwrap()
    }

    pub fn stderr_to_string(&self) -> String {
        String::from_utf8(self.stderr.clone()).unwrap()
    }
}

impl ConsoleOutput<Vec<u8>, Vec<u8>> for BufferedOutput {
    fn stdout(&mut self) -> &mut Vec<u8> {
        &mut self.stdout
    }

    fn stderr(&mut self) -> &mut Vec<u8> {
        &mut self.stderr
    }
}

#[cfg(test)]
mod tests {
    use crate::util::io::ConsoleOutput;
    use std::io::Write;

    #[test]
    fn create_std_output() {
        let mut output = super::StdOutput::new();
        writeln!(output.stdout(), "Hello world!").unwrap();
        writeln!(output.stderr(), "Hello world!").unwrap();
    }
}
