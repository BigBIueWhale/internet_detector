use std::sync::{Arc, atomic};
use cancellation::{CancellationToken, CancellationTokenSource};
use std::thread;

#[derive(Debug)]
pub struct InternetConnectivityMonitor
{
    cts: CancellationTokenSource,
    is_internet_connected: Arc<BooleanPredicate>,
    handle: thread::JoinHandle<()>,
}

#[derive(Debug, Clone, Copy)]
pub struct PollSettings
{
    remote_ip: std::net::IpAddr,
    timeout: std::time::Duration,
}
impl PollSettings
{
    pub fn new(remote_ip: std::net::IpAddr, timeout: std::time::Duration) -> Self
    {
        Self {
            remote_ip,
            timeout,
        }
    }
}

impl InternetConnectivityMonitor
{
    pub fn start(poll_settings: PollSettings) -> Result<Self, Box<dyn std::error::Error>>
    {
        let cts = CancellationTokenSource::new();
        let is_internet_connected = Arc::new(BooleanPredicate{ predicate: atomic::AtomicBool::new(false) });
        let iic = is_internet_connected.clone();
        let ct = cts.token().clone();
        let handle = thread::spawn(move || {
            match monitor_internet_connectivity(ct, poll_settings, iic)
            {
                Ok(_) => (),
                Err(e) => {
                    println!("{}", e);
                }
            };
        });
        Ok(Self {
            cts,
            is_internet_connected,
            handle,
        })
    }
    pub fn stop(self) -> Result<(), Box<dyn std::error::Error>>
    {
        self.cts.cancel();
        match self.handle.join()
        {
            Ok(_) => Ok(()),
            Err(_e) => Err("Could not join thread")
        }?;
        Ok(())
    }
    pub fn is_internet_connected(&self) -> bool
    {
        self.is_internet_connected.predicate.load(atomic::Ordering::SeqCst)
    }
}

#[derive(Debug)]
struct BooleanPredicate
{
    predicate: atomic::AtomicBool,
}

fn monitor_internet_connectivity(thread_stop: Arc<CancellationToken>, poll_settings: PollSettings, is_internet_connected: Arc<BooleanPredicate>) -> Result<(), Box<dyn std::error::Error>>
{
    while !thread_stop.is_canceled() {
        if check_internet_connectivity(poll_settings)? {
            is_internet_connected.predicate.store(true, atomic::Ordering::SeqCst);
        } else {
            is_internet_connected.predicate.store(false, atomic::Ordering::SeqCst);
        }
    }
    Ok(())
}

// Implement the function check_internet_connectivity using the ping library
use ping::ping;

fn check_internet_connectivity(poll_settings: PollSettings) -> Result<bool, Box<dyn std::error::Error>> {
    match ping(poll_settings.remote_ip, Some(poll_settings.timeout), None, None, None, None) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(true, true);
    }
}
