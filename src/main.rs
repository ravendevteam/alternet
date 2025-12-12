#![deny(clippy::unwrap_used)]

mod node;

#[::tokio::main]
async fn main() {
    node::Node::default().bootstrap().await;
}



// rough test build before running
#[cfg(test)]
mod int_test {
    use ::std::process;
    use ::std::io;

    use io::Write as _;

    #[test]
    fn launch() {
        let mut child = process::Command::new("./target/debug/alternet")
            .stdin(process::Stdio::piped())
            .stdout(process::Stdio::piped())
            .stderr(process::Stdio::null())
            .spawn()
            .expect("Spawn failed"); 
        {{
            let stdin = child.stdin.as_mut().expect("Failed to open stdin");
            stdin.write_all(b"hello world\n").expect("...");
        }}
        let output = child.wait_with_output().expect("");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("hello world"));
    }
}