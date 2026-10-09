//! # 进程与管道
//!
//! 在本练习中，你将学习如何创建子进程并通过管道进行通信。
//!
//! ## 概念
//! - `std::process::Command` 创建子进程（对应 `fork()` + `execve()` 系统调用）
//! - `Stdio::piped()` 设置管道（对应 `pipe()` + `dup2()` 系统调用）
//! - 通过 stdin/stdout 与子进程通信
//! - 获取子进程退出状态（对应 `waitpid()` 系统调用）
//!
//! ## 操作系统概念映射
//! 本练习展示了用户空间对底层操作系统原语的抽象：
//! - **进程创建**：Rust 的 `Command::new()` 内部调用 `fork()` 创建子进程，
//!   然后调用 `execve()`（或等价调用）用目标程序替换子进程的内存映像。
//! - **进程间通信（IPC）**：管道是由内核管理的缓冲区，允许相关进程之间单向传输数据。
//!   `pipe()` 系统调用创建一个管道，返回两个文件描述符（读端、写端）。`dup2()` 复制文件描述符，
//!   从而能够重定向标准输入/输出。
//! - **资源管理**：当 Rust 的 `Stdio` 对象被丢弃时，文件描述符（包括管道端）会自动关闭，防止资源泄漏。
//!
//! ## 练习结构
//! 1. **基本命令执行**（`run_command`）——启动子进程并捕获其 stdout。
//! 2. **双向管道通信**（`pipe_through_cat`）——向子进程（`cat`）发送数据并读取其输出。
//! 3. **退出码获取**（`get_exit_code`）——获取子进程的终止状态。
//! 4. **进阶：错误处理版本**（`run_command_with_result`）——学习正确的错误传播。
//! 5. **进阶：复杂双向通信**（`pipe_through_grep`）——与一个读取多行并产生过滤输出的过滤程序交互。
//!
//! 每个函数都包含一个 `TODO` 注释，指示你需要编写代码的位置。
//! 运行 `cargo test` 来检查你的实现。

use std::io::{self, Read, Write};
use std::os::unix::process;
use std::process::{Command, CommandEnvs, Output, Stdio};

/// Execute the given shell command and return its stdout output.
///
/// For example: `run_command("echo", &["hello"])` should return `"hello\n"`
///
/// # Underlying System Calls
/// - `Command::new(program)` → `fork()` + `execve()` family
/// - `Stdio::piped()` → `pipe()` + `dup2()` (sets up a pipe for stdout)
/// - `.output()` → `waitpid()` (waits for child process termination)
///
/// # Implementation Steps
/// 1. Create a `Command` with the given program and arguments.
/// 2. Set `.stdout(Stdio::piped())` to capture the child's stdout.
/// 3. Call `.output()` to execute the child and obtain its `Output`.
/// 4. Convert the `stdout` field (a `Vec<u8>`) into a `String`.

pub fn run_command(program: &str, args: &[&str]) -> String {
    let output = Command::new(program)
        .args(args)                 
        .stdout(Stdio::piped())
        .output()
        .expect("failed to execute command");   

    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Write data to child process (cat) stdin via pipe and read its stdout output.
///
/// This demonstrates bidirectional pipe communication between parent and child processes.
///
/// # Underlying System Calls
/// - `Command::new("cat")` → `fork()` + `execve("cat")`
/// - `Stdio::piped()` (twice) → `pipe()` creates two pipes (stdin & stdout) + `dup2()` redirects them
/// - `ChildStdin::write_all()` → `write()` to the pipe's write end
/// - `drop(stdin)` → `close()` on the write end, sending EOF to child
/// - `ChildStdout::read_to_string()` → `read()` from the pipe's read end
///
/// # Ownership and Resource Management
/// Rust's ownership system ensures pipes are closed at the right time:
/// 1. The `ChildStdin` handle is owned by the parent; writing to it transfers data to the child.
/// 2. After writing, we explicitly `drop(stdin)` (or let it go out of scope) to close the write end.
/// 3. Closing the write end signals EOF to `cat`, causing it to exit after processing all input.
/// 4. The `ChildStdout` handle is then read to completion; dropping it closes the read end.
///
/// Without dropping `stdin`, the child would wait forever for more input (pipe never closes).
///
/// # Implementation Steps
/// 1. Create a `Command` for `"cat"` with `.stdin(Stdio::piped())` and `.stdout(Stdio::piped())`.
/// 2. `.spawn()` the command to obtain a `Child` with `stdin` and `stdout` handles.
/// 3. Write `input` bytes to the child's stdin (`child.stdin.take().unwrap().write_all(...)`).
/// 4. Drop the stdin handle (explicit `drop` or let it go out of scope) to close the pipe.
/// 5. Read the child's stdout (`child.stdout.take().unwrap().read_to_string(...)`).
/// 6. Wait for the child to exit with `.wait()` (or rely on drop‑wait).
pub fn pipe_through_cat(input: &str) -> String {
    // TODO: Create "cat" command, set stdin and stdout to piped
    // TODO: Spawn process
    // TODO: Write input to child process stdin
    // TODO: Drop stdin to close pipe (otherwise cat won't exit)
    // TODO: Read output from child process stdout
    todo!()
}

/// Get child process exit code.
/// Execute command `sh -c {command}` and return the exit code.
///
/// # Underlying System Calls
/// - `Command::new("sh")` → `fork()` + `execve("/bin/sh")`
/// - `.args(["-c", command])` passes the shell command line
/// - `.status()` → `waitpid()` (waits for child and retrieves exit status)
/// - `ExitStatus::code()` extracts the low‑byte exit code (0‑255)
///
/// # Implementation Steps
/// 1. Create a `Command` for `"sh"` with arguments `["-c", command]`.
/// 2. Call `.status()` to execute the shell and obtain an `ExitStatus`.
/// 3. Use `.code()` to get the exit code as `Option<i32>`.
/// 4. If the child terminated normally, return the exit code; otherwise return a default.
pub fn get_exit_code(command: &str) -> i32 {
    // TODO: Use Command::new("sh").args(["-c", command])
    // TODO: Execute and get status
    // TODO: Return exit code
    todo!()
}

/// Execute the given shell command and return its stdout output as a `Result`.
///
/// This version properly propagates errors that may occur during process creation,
/// execution, or I/O (e.g., command not found, permission denied, broken pipe).
///
/// # Underlying System Calls
/// Same as `run_command`, but errors are captured from the OS and returned as `Err`.
///
/// # Error Handling
/// - `Command::new()` only constructs the builder; errors occur at `.output()`.
/// - `.output()` returns `Result<Output, std::io::Error>`.
/// - `String::from_utf8()` may fail if the child's output is not valid UTF‑8.
///   In that case we return an `io::Error` with kind `InvalidData`.
///
/// # Implementation Steps
/// 1. Create a `Command` with the given program and arguments.
/// 2. Set `.stdout(Stdio::piped())`.
/// 3. Call `.output()` and propagate any `io::Error`.
/// 4. Convert `stdout` to `String` with `String::from_utf8`; if that fails, map to an `io::Error`.
pub fn run_command_with_result(program: &str, args: &[&str]) -> io::Result<String> {
    // TODO: Use Command::new to create process
    // TODO: Set stdout to Stdio::piped()
    // TODO: Execute with .output() and handle Result
    // TODO: Convert stdout to String with from_utf8, mapping errors to io::Error
    todo!()
}

/// Interact with `grep` via bidirectional pipes, filtering lines that contain a pattern.
///
/// This demonstrates complex parent‑child communication: the parent sends multiple
/// lines of input, the child (`grep`) filters them according to a pattern, and the
/// parent reads back only the matching lines.
///
/// # Underlying System Calls
/// - `Command::new("grep")` → `fork()` + `execve("grep")`
/// - Two pipes (stdin & stdout) as in `pipe_through_cat`
/// - Line‑by‑line writing and reading to simulate interactive filtering
///
/// # Implementation Steps
/// 1. Create a `Command` for `"grep"` with argument `pattern`, and both ends piped.
/// 2. `.spawn()` the command, obtaining `Child` with `stdin` and `stdout` handles.
/// 3. Write each line of `input` (separated by `'\n'`) to the child's stdin.
/// 4. Close the write end (drop stdin) to signal EOF.
/// 5. Read the child's stdout line by line, collecting matching lines.
/// 6. Wait for the child to exit (optional; `grep` exits after EOF).
/// 7. Return the concatenated matching lines as a single `String`.
///
pub fn pipe_through_grep(pattern: &str, input: &str) -> String {
    // TODO: Create "grep" command with pattern, set stdin and stdout to piped
    // TODO: Spawn process
    // TODO: Write input lines to child stdin
    // TODO: Drop stdin to close pipe
    // TODO: Read output from child stdout line by line
    // TODO: Collect and return matching lines
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_echo() {
        let output = run_command("echo", &["hello"]);
        assert_eq!(output.trim(), "hello");
    }

    #[test]
    fn test_run_with_args() {
        let output = run_command("echo", &["-n", "no newline"]);
        assert_eq!(output, "no newline");
    }

    #[test]
    fn test_pipe_cat() {
        let output = pipe_through_cat("hello pipe!");
        assert_eq!(output, "hello pipe!");
    }

    #[test]
    fn test_pipe_multiline() {
        let input = "line1\nline2\nline3";
        assert_eq!(pipe_through_cat(input), input);
    }

    #[test]
    fn test_exit_code_success() {
        assert_eq!(get_exit_code("true"), 0);
    }

    #[test]
    fn test_exit_code_failure() {
        assert_eq!(get_exit_code("false"), 1);
    }

    #[test]
    fn test_run_command_with_result_success() {
        let result = run_command_with_result("echo", &["hello"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "hello");
    }

    #[test]
    fn test_run_command_with_result_nonexistent() {
        let result = run_command_with_result("nonexistent_command_xyz", &[]);
        // Should be an error because command not found
        assert!(result.is_err());
    }

    #[test]
    fn test_pipe_through_grep_basic() {
        let input = "apple\nbanana\ncherry\n";
        let output = pipe_through_grep("a", input);
        // grep outputs matching lines with newline
        assert_eq!(output, "apple\nbanana\n");
    }

    #[test]
    fn test_pipe_through_grep_no_match() {
        let input = "apple\nbanana\ncherry\n";
        let output = pipe_through_grep("z", input);
        // No lines match -> empty string
        assert_eq!(output, "");
    }

    #[test]
    fn test_pipe_through_grep_multiline() {
        let input = "first line\nsecond line\nthird line\n";
        let output = pipe_through_grep("second", input);
        assert_eq!(output, "second line\n");
    }
}
