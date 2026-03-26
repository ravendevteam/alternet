use std::time::Duration;
use std::process::Command;
use std::process::ExitStatus;
use std::process::exit;
use std::fs::write;
use std::thread::sleep;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

type Rules = Vec<Vec<&'static str>>;

fn main() -> ! {
    write("/proc/sys/net/ipv4/ip_forward", "1").unwrap();
    let mut rules: Rules = vec![];
    rules.extend(nat());
    rules.extend(firewall());
    for rule in rules {
        let exit_status: ExitStatus = Command::new("iptables").args(&rule).status().unwrap();
        if !exit_status.success() {
            eprintln!("Unable to apply rule: {:?}", rule);
            exit(1);
        }
    }
    println!("Router online");
    loop {
        sleep(Duration::from_secs(3600));
    }
}

fn nat() -> Rules {
    vec![
        vec!["-t", "nat", "-A", "POSTROUTING", "-o", "eth0", "-j", "MASQUERADE"],
        vec!["-P", "INPUT", "DROP"]
    ]
}

fn firewall() -> Rules {
    vec![
        vec!["-P", "INPUT", "DROP"]
    ]
}