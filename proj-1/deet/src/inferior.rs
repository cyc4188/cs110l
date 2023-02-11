use nix::sys::ptrace;
use nix::sys::signal;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
use std::process::Child;
use std::process::Command;
use std::os::unix::process::CommandExt;
use std::collections::HashMap;
use crate::dwarf_data:: DwarfData;
use crate::utils::align_addr_to_word;
use crate::debugger::Breakpoint;

pub enum Status {
    /// Indicates inferior stopped. Contains the signal that stopped the process, as well as the
    /// current instruction pointer that it is stopped at.
    Stopped(signal::Signal, usize),

    /// Indicates inferior exited normally. Contains the exit status code.
    Exited(i32),

    /// Indicates the inferior exited due to a signal. Contains the signal that killed the
    /// process.
    Signaled(signal::Signal),
}

/// This function calls ptrace with PTRACE_TRACEME to enable debugging on a process. You should use
/// pre_exec with Command to call this in the child process.
fn child_traceme() -> Result<(), std::io::Error> {
    ptrace::traceme().or(Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "ptrace TRACEME failed",
    )))
}

pub struct Inferior {
    child: Child,
}

impl Inferior {
    /// Attempts to start a new inferior process. Returns Some(Inferior) if successful, or None if
    /// an error is encountered.
    pub fn new(target: &str, args: &Vec<String>, breakpoints: &mut HashMap<usize, Option<Breakpoint>>) -> Option<Inferior> {
        let mut cmd = Command::new(target);
        cmd.args(args);
        unsafe {
            cmd.pre_exec(child_traceme);
        };
        
        match cmd.spawn() {
            Ok(child) => {
                let mut inferior = Inferior { child };
                breakpoints.iter_mut().for_each(|(addr, breakpoint)| {
                    match inferior.set_breakpoint(*addr) {
                        Ok(orig_byte) => {
                            breakpoint.replace(Breakpoint { addr: *addr, orig_byte });
                        }
                        Err(e) => println!("Error setting breakpoint: {:?}", e),
                    }
                });
                Some(inferior)
            }
            Err(_) => None,
        }
    }

    /// Returns the pid of this inferior.
    pub fn pid(&self) -> Pid {
        nix::unistd::Pid::from_raw(self.child.id() as i32)
    }

    /// Calls waitpid on this inferior and returns a Status to indicate the state of the process
    /// after the waitpid call.
    pub fn wait(&self, options: Option<WaitPidFlag>) -> Result<Status, nix::Error> {
        Ok(match waitpid(self.pid(), options)? {
            WaitStatus::Exited(_pid, exit_code) => Status::Exited(exit_code),
            WaitStatus::Signaled(_pid, signal, _core_dumped) => Status::Signaled(signal),
            WaitStatus::Stopped(_pid, signal) => {
                let regs = ptrace::getregs(self.pid())?;
                Status::Stopped(signal, regs.rip as usize)
            }
            other => panic!("waitpid returned unexpected status: {:?}", other),
        })
    }

    /// continue to run the inferior
    pub fn cont(&self) -> Result<Status, nix::Error> {


        ptrace::cont(self.pid(), None)?;
        self.wait(None)
    }

    /// kill the inferior
    pub fn kill(&mut self) {
        println!("Killing inferior {}", self.pid());
        self.child.kill().unwrap();
        self.wait(None).unwrap();
    }
    
    /// print stack trace of the program
    pub fn backtrace(&self, debug_data: &DwarfData) -> Result<(), nix::Error>  {
        let regs = ptrace::getregs(self.pid())?;
        let mut instr_ptr = regs.rip as usize;
        let mut ebp = regs.rbp as usize; 
        loop {
            println!("instr_ptr: 0x{:#x}:", instr_ptr);
            println!("ebp: 0x{:#x}:", ebp);
            let line = debug_data.get_line_from_addr(instr_ptr).unwrap();
            let func = debug_data.get_function_from_addr(instr_ptr).unwrap();
            println!("{} ({})", func, line);
            if func == "main" {
                break;
            }
            instr_ptr = ptrace::read(self.pid(), (ebp + 8) as ptrace::AddressType)? as usize;
            ebp = ptrace::read(self.pid(), ebp as ptrace::AddressType)? as usize;
        }
        Ok(())
    }

    pub fn set_breakpoint(&mut self, addr: usize) -> Result<u8, nix::Error> {
        self.write_byte(addr, 0xcc)
    }

    /// write byte to process memory
    /// used for setting breakpoints
    fn write_byte(&mut self, addr: usize, val: u8) -> Result<u8, nix::Error> {
        let aligned_addr = align_addr_to_word(addr);
        let byte_offset = addr - aligned_addr;
        let word = ptrace::read(self.pid(), aligned_addr as ptrace::AddressType)? as u64;
        let orig_byte = (word >> 8 * byte_offset) & 0xff;
        let masked_word = word & !(0xff << 8 * byte_offset);
        let updated_word = masked_word | ((val as u64) << 8 * byte_offset);
        unsafe {
            ptrace::write(
                self.pid(),
                aligned_addr as ptrace::AddressType,
                updated_word as *mut std::ffi::c_void,
            )?;
        }
        Ok(orig_byte as u8)
    }

}
