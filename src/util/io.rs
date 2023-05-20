pub trait ConsoleOutput<OUT, ERR>
where
    OUT: std::io::Write,
    ERR: std::io::Write,
{
    fn stdout<'a>(&'a mut self) -> &'a mut OUT;
    fn stderr<'a>(&'a mut self) -> &'a mut ERR;
}

pub struct StdOutput {
    stdout: std::io::Stdout,
    stderr: std::io::Stderr,
}

impl StdOutput {
    pub fn new() -> Self {
        Self {
            stdout: std::io::stdout(),
            stderr: std::io::stderr(),
        }
    }
}

impl ConsoleOutput<std::io::Stdout, std::io::Stderr> for StdOutput {
    fn stdout<'a>(&'a mut self) -> &'a mut std::io::Stdout {
        &mut self.stdout
    }

    fn stderr<'a>(&'a mut self) -> &'a mut std::io::Stderr {
        &mut self.stderr
    }
}

pub struct BufferedOutput {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl BufferedOutput {
    pub fn new() -> Self {
        Self {
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }

    pub fn stdout_to_string(&self) -> String {
        String::from_utf8(self.stdout.clone()).unwrap()
    }

    pub fn stderr_to_string(&self) -> String {
        String::from_utf8(self.stderr.clone()).unwrap()
    }
}

impl ConsoleOutput<Vec<u8>, Vec<u8>> for BufferedOutput {
    fn stdout<'a>(&'a mut self) -> &'a mut Vec<u8> {
        &mut self.stdout
    }

    fn stderr<'a>(&'a mut self) -> &'a mut Vec<u8> {
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
