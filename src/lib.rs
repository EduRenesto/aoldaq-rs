use std::thread::JoinHandle;

use std::sync::{ Arc, Barrier };
use std::sync::atomic::{ AtomicBool, Ordering };

use ringbuf::{ Consumer, RingBuffer };

mod capi;
pub use capi::*;

mod device;
use device::{ Device, RandomDevice, NiFpgaDevice };

mod nifpga;

const BUCKET_SIZE: usize = 20;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum AoldaqMode {
    NiFpga,
    Random,
}

#[repr(C)]
pub struct AoldaqArgs {
    n_channels: usize,
    mode: AoldaqMode,
    nifpga: *const NiFpgaArgs
}

#[repr(C)]
pub struct NiFpgaArgs {
    bitfile: *const std::os::raw::c_char,
    signature: *const std::os::raw::c_char,
    resource: *const std::os::raw::c_char,
    attribute: u32,
    addrs: *const u32,
}

pub struct Aoldaq {
    n_channels: usize,
    mode: AoldaqMode,
    threads: Vec<JoinHandle<()>>,
    pause: Arc<AtomicBool>,
    run: Arc<AtomicBool>,
    fifos: Vec<Consumer<u32>>,
    device: Arc<dyn Device>,
}

impl Aoldaq {
    pub fn create(args: &AoldaqArgs) -> Aoldaq {
        let mut threads = Vec::with_capacity(args.n_channels);
        let mut fifos = Vec::with_capacity(args.n_channels);

        let pause = Arc::new(AtomicBool::new(true));
        let run = Arc::new(AtomicBool::new(true));

        let barrier = Arc::new(Barrier::new(args.n_channels));

        let device = match args.mode {
            AoldaqMode::Random => Arc::new(RandomDevice::new()) as Arc<dyn Device>,
            AoldaqMode::NiFpga => Arc::new(NiFpgaDevice::new(args.nifpga, args.n_channels, false)
                                           .expect("Failed to init NiFpga")) as Arc<dyn Device>,
        };

        for i in 0..args.n_channels {
            let buf = RingBuffer::new(1024 * 1024);
            let (mut tx, rx) = buf.split();
            //let (tx, rx) = crossbeam_channel::unbounded();
            //let (tx, rx) = crossbeam_channel::bounded(4 * 1024 * 1024);
            fifos.push(rx);

            let device = device.clone();
            let pause = pause.clone();
            let run = run.clone();
            let b = barrier.clone();

            let thread = std::thread::spawn(move || {
                let mut buf = vec![666; BUCKET_SIZE];
                //tx.send((0..10).into_iter().map(|n| n*i as u32).collect()).expect("Failed to send to fifo");
                b.wait();

                while run.load(Ordering::Relaxed) {
                    if pause.load(Ordering::Relaxed) {
                        println!("Parking thread {}", i);
                        std::thread::park();
                    }

                    device.read_into(i, &mut buf[..]);
                    tx.push_slice(&buf[..]);

                    //tx.send(device.read_data(i, BUCKET_SIZE)).expect("Failed to send to fifo");
                }
            });
            threads.push(thread);
        }

        Aoldaq {
            n_channels: args.n_channels,
            mode: args.mode,
            threads,
            pause,
            run,
            fifos,
            device,
        }
    }

    pub fn start(&self) {
        self.pause.store(false, Ordering::Relaxed);
        for t in &self.threads {
            t.thread().unpark();
        }
    }

    pub fn stop(&self) {
        self.pause.store(true, Ordering::Relaxed);
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

    pub fn get_fifo_size(&self, channel: usize) -> usize {
        self.fifos[channel].len()
    }

    pub fn flush_fifo(&mut self, channel: usize) {
        let should_restart = !self.pause.load(Ordering::Relaxed);

        if should_restart { self.stop(); }
        //while let Ok(data) = self.fifos[channel].try_recv() {
            //drop(data);
        //}
        let rx = unsafe { self.fifos.get_unchecked_mut(channel) };
        let n = rx.len();
        rx.discard(n);
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
        println!("AOLDAQ finishing...");
        self.run.store(false, Ordering::Relaxed);
        self.start();

        for t in self.threads.drain(..) {
            t.join().unwrap();
        }

        for fifo in self.fifos.drain(..) {
            drop(fifo);
        }

        println!("AOLDAQ finished.");
    }
}
