pub trait ConsoleOutput<'a, OUT, ERR>
where
    OUT: std::io::Write,
    ERR: std::io::Write,
{
    fn stdout(&'a mut self) -> &'a mut OUT;
    fn stderr(&'a mut self) -> &'a mut ERR;
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

impl<'a> ConsoleOutput<'a, std::io::Stdout, std::io::Stderr> for StdOutput {
    fn stdout(&'a mut self) -> &'a mut std::io::Stdout {
        &mut self.stdout
    }

    fn stderr(&'a mut self) -> &'a mut std::io::Stderr {
        &mut self.stderr
    }
}

pub struct MockOutput {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl MockOutput {
    pub fn new() -> Self {
        Self {
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }
}

impl<'a> ConsoleOutput<'a, Vec<u8>, Vec<u8>> for MockOutput {
    fn stdout(&'a mut self) -> &'a mut Vec<u8> {
        &mut self.stdout
    }

    fn stderr(&'a mut self) -> &'a mut Vec<u8> {
        &mut self.stderr
    }
}
