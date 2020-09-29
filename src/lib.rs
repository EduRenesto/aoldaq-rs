use std::thread::JoinHandle;

use std::sync::{ Arc, Barrier };
use std::sync::atomic::{ AtomicBool, Ordering };

use ringbuf::{ Consumer, RingBuffer };

#[cfg(unix)]
use simplelog::TermLogger;
#[cfg(not(unix))]
use simplelog::WriteLogger;

use simplelog::{ 
    LevelFilter,
    TerminalMode,
    Config
};

mod capi;
pub use capi::*;

mod device;
use device::{ Device, RandomDevice, NiFpgaDevice };

mod nifpga;

const _BUCKET_SIZE: usize = 2000;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum AoldaqMode {
    NiFpga,
    Random,
}

#[repr(C)]
pub struct AoldaqArgs {
    pub block_size: usize,
    pub n_channels: usize,
    pub mode: AoldaqMode,
    pub nifpga: *const NiFpgaArgs
}

#[repr(C)]
pub struct NiFpgaArgs {
    pub bitfile: *const std::os::raw::c_char,
    pub signature: *const std::os::raw::c_char,
    pub resource: *const std::os::raw::c_char,
    pub attribute: u32,
    pub addrs: *const u32,
}

pub struct Aoldaq {
    n_channels: usize,
    mode: AoldaqMode,
    threads: Vec<JoinHandle<()>>,
    can_acquire: Arc<AtomicBool>,
    pause: Arc<AtomicBool>,
    run: Arc<AtomicBool>,
    fifos: Vec<Consumer<u32>>,
    device: Arc<dyn Device>,
}

impl Aoldaq {
    pub fn create(args: &AoldaqArgs) -> Aoldaq {
        // Init logging
        #[cfg(unix)]
        TermLogger::init(LevelFilter::max(), Config::default(), TerminalMode::Mixed).unwrap_or(());

        #[cfg(not(unix))]
        {
            let tmp = std::env::var("TEMP").unwrap_or("/tmp".to_string());
            let mut tmp = std::path::PathBuf::from(tmp);
            tmp.push("aoldaq.log");
            
            WriteLogger::init(LevelFilter::max(), Config::default(), std::fs::File::create(tmp).unwrap()).unwrap_or(());
        }

        let mut threads = Vec::with_capacity(args.n_channels);
        let mut fifos = Vec::with_capacity(args.n_channels);

        let pause = Arc::new(AtomicBool::new(true));
        let run = Arc::new(AtomicBool::new(true));
        let can_acquire = Arc::new(AtomicBool::new(true));

        let barrier = Arc::new(Barrier::new(args.n_channels));

        let device = match args.mode {
            AoldaqMode::Random => Arc::new(RandomDevice::new()) as Arc<dyn Device>,
            AoldaqMode::NiFpga => Arc::new(NiFpgaDevice::new(args.nifpga, args.n_channels, false)
                                           .expect("Failed to init NiFpga")) as Arc<dyn Device>,
        };

        let block_size = args.block_size;

        for i in 0..args.n_channels {
            let buf = RingBuffer::new(4 * 268435456); // 4GB worth of points. If needed, can safely be increased
            //let buf = RingBuffer::new(512*512);
            let (mut tx, rx) = buf.split();
            //let (tx, rx) = crossbeam_channel::unbounded();
            //let (tx, rx) = crossbeam_channel::bounded(4 * 1024 * 1024);
            fifos.push(rx);

            let can_acquire = can_acquire.clone();
            let device = device.clone();
            let pause = pause.clone();
            let run = run.clone();
            let b = barrier.clone();

            let thread = std::thread::spawn(move || {
                let mut buf = vec![666; block_size];
                //tx.send((0..10).into_iter().map(|n| n*i as u32).collect()).expect("Failed to send to fifo");
                b.wait();

                while run.load(Ordering::Relaxed) {
                    if pause.load(Ordering::Relaxed) {
                        //println!("Parking thread {}", i);
                        log::info!("Parking thread {}", i);
                        std::thread::park();
                    }

                    match device.read_into(i, &mut buf[..]) {
                        Ok(_n) => {
                            let mut written = 0;

                            while written < block_size && can_acquire.load(Ordering::Relaxed) {
                                written += tx.push_slice(&buf[written..]);

                                if written < block_size {
                                    log::debug!("Overflow: Full fifo for channel {}, wrote {} out of {}", i, written, block_size);
                                }
                            }

                            //tx.send(device.read_data(i, BUCKET_SIZE)).expect("Failed to send to fifo");
                        }
                        Err(e) => {
                            log::error!("Device read error: {}", e);
                        }
                    };
                }
            });
            threads.push(thread);
        }

        log::info!("AOLDAQ started.");

        Aoldaq {
            n_channels: args.n_channels,
            mode: args.mode,
            threads,
            can_acquire,
            pause,
            run,
            fifos,
            device,
        }
    }

    pub fn start(&self) {
        self.pause.store(false, Ordering::SeqCst);
        self.can_acquire.store(true, Ordering::SeqCst);
        for t in &self.threads {
            t.thread().unpark();
        }
    }

    pub fn stop(&self) {
        self.pause.store(true, Ordering::SeqCst);
        self.can_acquire.store(false, Ordering::SeqCst);
    }

    pub fn get_data_into(&mut self, channel: usize, buf: &mut [u32]) -> usize {
        if channel > self.n_channels {
            return 0;
        }

        let rx = unsafe { self.fifos.get_unchecked_mut(channel) };

        //if n > rx.len() {
            //let n = rx.len();
            //return Some(rx.iter().flatten().take(n).collect());
            ////return None;
        //}

        //Some( rx.iter().flatten().take(n).collect() )

        rx.pop_slice(buf)
    }

    pub fn get_data_into_blocking(&mut self, channel: usize, buf: &mut [u32], timeout: std::time::Duration) -> usize {
        if channel > self.n_channels {
            return 0;
        }

        let rx = unsafe { self.fifos.get_unchecked_mut(channel) };

        if buf.len() > rx.len() {
            log::debug!("Underflow: Tried to get {} points from channel {} which has {} points",
                        buf.len(),
                        channel,
                        rx.len());
        }

        let mut time_spent = std::time::Duration::from_micros(0);
        let wait_interval = std::time::Duration::from_millis(1);

        while time_spent < timeout && buf.len() > rx.len() {
            std::thread::sleep(wait_interval);
            time_spent += wait_interval;
        }

        if time_spent >= wait_interval {
            log::debug!("Slept for {}ms total waiting for data for channel {}", time_spent.as_millis(), channel);
        }
        rx.pop_slice(buf)
    }

    pub fn get_fifo_size(&self, channel: usize) -> usize {
        self.fifos[channel].len()
    }

    pub fn flush_fifo(&mut self, channel: usize) {
        let should_restart = !self.pause.load(Ordering::Relaxed);
        let was_acquiring = self.can_acquire.load(Ordering::Relaxed);

        if should_restart { self.stop(); }
        //while let Ok(data) = self.fifos[channel].try_recv() {
            //drop(data);
        //}

        log::debug!("flush_fifo({}) requested", channel);
        log::debug!("current total points in sw fifo: {:?}",
                    (0..self.n_channels)
                    .into_iter()
                    .map(|i| self.get_fifo_size(i))
                    .collect::<Vec<_>>());

        let rx = unsafe { self.fifos.get_unchecked_mut(channel) };
        if was_acquiring { self.can_acquire.store(false, Ordering::SeqCst); }
        let mut n = rx.len();
        while n > 0 {
            rx.discard(n);
            n = rx.len();
        }

        log::debug!("flush_fifo done");
        log::debug!("current total points in sw fifo: {:?}",
                    (0..self.n_channels)
                    .into_iter()
                    .map(|i| self.get_fifo_size(i))
                    .collect::<Vec<_>>());

        if was_acquiring { self.can_acquire.store(true, Ordering::SeqCst); }
        if should_restart { self.start(); }
    }

    pub fn get_nifpga_session(&self) -> Option<nifpga::NiFpga_Session> {
        match self.mode {
            AoldaqMode::Random => None,
            AoldaqMode::NiFpga => {
                let ptr = Arc::as_ptr(&self.device);
                let device: *const device::NiFpgaDevice = ptr as *const _;
                Some(unsafe { (*device).get_nifpga_session() })
            }
        }
    }
}

impl Drop for Aoldaq {
    fn drop(&mut self) {
        log::info!("AOLDAQ finishing...");
        self.run.store(false, Ordering::Relaxed);
        self.start();

        for t in self.threads.drain(..) {
            t.join().unwrap();
        }

        for fifo in self.fifos.drain(..) {
            drop(fifo);
        }

        log::info!("AOLDAQ finished.");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_ringbuffer() {
        let fifo = ringbuf::RingBuffer::new(5);
        let (mut tx, mut rx) = fifo.split();

        let from = vec![2, 3, 4, 5, 6, 7, 8];

        let n1 = tx.push_slice(&from[..]);
        assert_eq!(n1, 5);

        let n1 = tx.push_slice(&from[n1..]);
        assert_eq!(n1, 0);

        let mut to = vec![0; 5];
        rx.pop_slice(&mut to[..]);

        assert_eq!(&to[..], &[2, 3, 4, 5, 6]);
    }
}
