mod internet_connectivity_monitor;
use internet_connectivity_monitor::InternetConnectivityMonitor;

use rodio::{Source, Sink};
use std::io::BufReader;

use std::net::IpAddr;
use std::str::FromStr;

// Import atomic
use std::sync::{Arc, atomic};
use crate::internet_connectivity_monitor::PollSettings;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match run_program() {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("{}", e);
            Ok(())
        }
    }
}

pub fn run_program() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Rodio and get the default audio device
    let (_stream, device) = rodio::OutputStream::try_default()?;

    // Create a sink for the default audio device
    let sink = Sink::try_new(&device)?;

    // All IP addresses of Google's global DNS server.
    // When using some mobile data providers, only the IPv6 addresses work.
    // When using most home internet connections, only the IPv4 addresses work.
    let ip_addresses = vec![
        "8.8.8.8",
        "8.8.4.4",
        "2001:4860:4860::8888",
        "2001:4860:4860::8844",
    ];
    let mut internet_connectivity_monitors =
        create_internet_connectivity_monitors(&ip_addresses, std::time::Duration::from_secs(3))?;
    
    // Continue loop until CTRL+C is pressed
    let was_ctrl_c_pressed = Arc::new(atomic::AtomicBool::new(false));
    let wccp = was_ctrl_c_pressed.clone();
    ctrlc::set_handler(move || {
        wccp.store(true, atomic::Ordering::SeqCst);
    })?;
    // Check for internet connectivity every 4 seconds
    while !was_ctrl_c_pressed.load(atomic::Ordering::SeqCst) {
        let num_connected = internet_connectivity_monitors.iter().filter(
                                        |monitor| monitor.is_internet_connected())
                                        .count();
        println!("num_connected: \"{}\"", num_connected);
        if num_connected >= 1 {
            // Sleep for the minimum amount of time to avoid busy-waiting
            std::thread::sleep(std::time::Duration::from_millis(14));
        } else {
            // If there is no internet connectivity, play a custom wav file using the Rodio library
            let file_path = "./alarm.wav";
            let file: std::fs::File = std::fs::File::open(file_path)?;
            let source: rodio::Decoder<BufReader<std::fs::File>> = rodio::Decoder::new(BufReader::new(file))?;
            let source_duration = source.total_duration().ok_or("Could not get duration")?;
            sink.append(source);
            sink.play();

            // Wait for the mp3 file to finish playing before checking for internet connectivity again
            std::thread::sleep(source_duration);
        }
    }
    while !internet_connectivity_monitors.is_empty() {
        internet_connectivity_monitors.pop().ok_or("pop on empty Vec")?.stop()?;
    }
    Ok(())
}

fn create_internet_connectivity_monitors(ip_addresses: &[&str], timeout: std::time::Duration) -> Result<Vec<InternetConnectivityMonitor>, Box<dyn std::error::Error>>
{
    ip_addresses.iter().map(
        |ip_address_str| {
            let ip_address = IpAddr::from_str(ip_address_str)?;
            InternetConnectivityMonitor::start(PollSettings::new(ip_address, timeout))
        }).collect()
}
