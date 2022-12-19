use rodio::{Source, Sink};
use std::time::Duration;
use std::io::BufReader;
use std::sync::{Arc, atomic};
use cancellation::{CancellationToken, CancellationTokenSource};
use std::thread;


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

    let cts: CancellationTokenSource = CancellationTokenSource::new();
    let is_internet_connected = Arc::new(BooleanPredicate {
        predicate: atomic::AtomicBool::new(true),
    });
    let iic = is_internet_connected.clone();
    let ct = cts.token().clone();

    // Launch the function as a thread
    let handle = thread::spawn(move || {
        match monitor_internet_connectivity(ct, iic)
        {
            Ok(_) => (),
            Err(e) => {
                println!("{}", e);
            }
        };
    });

    // Continue loop until CTRL+C is pressed
    let was_ctrl_c_pressed = Arc::new(atomic::AtomicBool::new(false));
    let wccp = was_ctrl_c_pressed.clone();
    ctrlc::set_handler(move || {
        wccp.store(true, atomic::Ordering::SeqCst);
    })?;
    // Check for internet connectivity every 4 seconds
    while !was_ctrl_c_pressed.load(atomic::Ordering::SeqCst) {
        if is_internet_connected.predicate.load(atomic::Ordering::SeqCst) {
            // Sleep for the minimum amount of time
            std::thread::sleep(Duration::from_millis(14));
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
    cts.cancel();
    match handle.join()
    {
        Ok(_) => Ok(()),
        Err(_e) => Err("Could not join thread")
    }?;
    Ok(())
}

pub struct BooleanPredicate
{
    predicate: atomic::AtomicBool,
}

pub fn monitor_internet_connectivity(thread_stop: Arc<CancellationToken>, is_internet_connected: Arc<BooleanPredicate>) -> Result<(), Box<dyn std::error::Error>>
{
    while !thread_stop.is_canceled() {
        if check_internet_connectivity(Duration::from_secs(1))? {
            is_internet_connected.predicate.store(true, atomic::Ordering::SeqCst);
        } else {
            is_internet_connected.predicate.store(false, atomic::Ordering::SeqCst);
        }
    }
    Ok(())
}

// Implement the function check_internet_connectivity using the ping library
use ping::ping;

fn check_internet_connectivity(timeout: Duration) -> Result<bool, Box<dyn std::error::Error>> {
    // IP address "54.243.56.11" is my personal Amazon AWS EC2 server.
    let host: std::net::IpAddr = std::net::Ipv4Addr::new(54, 243, 56, 11).into();
    match ping(host, Some(timeout), None, None, None, None) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
