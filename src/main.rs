use ansi_brush::Style;
use clap::Parser;
use nix::unistd;
use nom::multi::many0;
use std::{fs, io};

mod diff;
mod memory_map;
mod parse;

fn get_rss(pid: usize) -> io::Result<usize> {
    let path = format!("/proc/{}/statm", pid);
    let contents = fs::read_to_string(path)?;

    let parts: Vec<&str> = contents.split_whitespace().collect();
    if parts.len() > 1 {
        // The second value in /proc/[pid]/statm is the RSS in pages
        let rss_pages = parts[1].parse::<usize>().unwrap();
        // Convert pages to bytes
        let page_size = unistd::sysconf(unistd::SysconfVar::PAGE_SIZE)
            .unwrap()
            .unwrap_or(4096) as i64;
        Ok(rss_pages * page_size as usize)
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Unexpected /proc/[pid]/statm format",
        ))
    }
}

#[derive(Parser, Debug)]
struct Args {
    /// PID of the process
    #[arg(short, long)]
    pid: usize,

    /// Show just the files
    #[arg(long)]
    files: bool,

    /// Show differences every period seconds
    #[arg(long)]
    period: Option<usize>,
}

fn main() {
    let args = Args::parse();
    let smaps_path = format!("/proc/{}/smaps", args.pid);

    if let Some(period) = args.period {
        let mut last_memory_map = Vec::new();
        loop {
            let content = fs::read_to_string(&smaps_path).expect("Failed to read smaps file");

            let memory_map = many0(parse::parse_memory_map)(&content)
                .expect("Failed to parse memory map")
                .1;

            let diffs = diff::diff_sorted(&last_memory_map, &memory_map);
            println!("");
            println!(
                "{} - {} mb",
                chrono::Local::now(),
                get_rss(args.pid).unwrap() / (1024 * 1024)
            );
            println!("ADDED");
            for m in diffs.added {
                println!("{}{}{}", "".green(), m, "".reset());
            }

            println!("REMOVED");
            for m in diffs.removed {
                println!("{}{}{}", "".red(), m, "".reset());
            }

            println!("CHANGED");
            for (a, b) in diffs.changed {
                println!("{}{}{}", "".cyan(), a, "".reset());
                println!("{}{}{}", "".yellow(), b, "".reset());
                println!("--------");
            }

            last_memory_map = memory_map;

            std::thread::sleep(std::time::Duration::from_secs(period as u64));
        }
    } else {
        let content = fs::read_to_string(smaps_path).expect("Failed to read smaps file");

        let map = many0(parse::parse_memory_map)(&content);
        match map {
            Ok((_, memory_map)) => {
                for m in memory_map {
                    if args.files {
                        if let Some(path) = &m.path {
                            println!("{} {}", path, m.size().unwrap());
                        }
                    } else {
                        println!("{:?}", m)
                    }
                }
            }
            Err(err) => eprintln!("Error parsing memory map: {:?}", err),
        }
    }
}
