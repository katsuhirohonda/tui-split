use anyhow::Result;
use portable_pty::{native_pty_system, CommandBuilder, PtySize, MasterPty, Child};
use std::io::{Read, Write};

pub struct Terminal {
    master: Box<dyn MasterPty + Send>,
    child: Box<dyn Child + Send + Sync>,
}

impl Drop for Terminal {
    fn drop(&mut self) {
        // Cleanup: kill the child process gracefully
        let _ = self.child.kill();
    }
}

impl Terminal {
    pub fn new() -> Result<Self> {
        Self::new_with_size(24, 80)
    }
    
    pub fn new_with_size(rows: u16, cols: u16) -> Result<Self> {
        let pty_system = native_pty_system();
        
        let pty_pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        
        let cmd = CommandBuilder::new("zsh");
        let child = pty_pair.slave.spawn_command(cmd)?;
        
        Ok(Terminal {
            master: pty_pair.master,
            child,
        })
    }
    
    pub fn resize(&self, rows: u16, cols: u16) -> Result<()> {
        self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        Ok(())
    }

    pub fn write(&mut self, data: &[u8]) -> Result<usize> {
        let mut writer = self.master.take_writer()?;
        Ok(writer.write(data)?)
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut reader = self.master.try_clone_reader()?;
        Ok(reader.read(buf)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_terminal() {
        // Red: This test should fail first
        let terminal = Terminal::new();
        assert!(terminal.is_ok(), "Should be able to create a terminal");
    }

    #[test]
    fn test_terminal_can_execute_zsh() {
        // Red: Test that we can start zsh
        let _terminal = Terminal::new().expect("Failed to create terminal");
        // We expect this to work once implemented
    }

    #[test]
    fn test_terminal_can_write_command() {
        // Red: Test that we can send input to the terminal
        let mut terminal = Terminal::new().expect("Failed to create terminal");
        let result = terminal.write(b"echo hello\n");
        assert!(result.is_ok(), "Should be able to write to terminal");
    }

    #[test]
    fn test_terminal_can_read_output() {
        // Red: Test that we can read output from the terminal
        let mut terminal = Terminal::new().expect("Failed to create terminal");
        
        // Wait a bit for terminal to initialize
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        terminal.write(b"echo hello\n").expect("Failed to write");
        
        // Wait for command to execute
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        let mut buf = [0u8; 1024];
        let mut total_read = 0;
        
        // Try reading multiple times in case data arrives in chunks
        for _ in 0..10 {
            match terminal.read(&mut buf[total_read..]) {
                Ok(n) if n > 0 => {
                    total_read += n;
                    break;
                }
                Ok(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(e) => panic!("Read error: {}", e),
            }
        }
        
        assert!(total_read > 0, "Should have read some bytes");
    }
}