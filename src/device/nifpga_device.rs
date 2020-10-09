use super::Device;
use crate::NiFpgaArgs;
use crate::nifpga;

use std::fs::{ File, OpenOptions };
use std::sync::{ Arc, atomic::AtomicUsize, atomic::Ordering, Mutex };
use std::io::Write;

pub struct NiFpgaDevice {
    session: nifpga::NiFpga_Session,
    pub addrs: Vec<u32>,
    out_file: Option<Mutex<File>>,
    counters: Vec<Arc<AtomicUsize>>,
}

impl NiFpgaDevice {
    pub fn new(args: *const NiFpgaArgs, n_channels: usize, dump: bool) -> Result<NiFpgaDevice, nifpga::NiFpga_Status> {
        let args = unsafe { args.as_ref().expect("NiFpgaArgs is null!") };

        let ret = unsafe { nifpga::NiFpga_Initialize() };

        if ret != nifpga::NiFpga_Status_Success {
            return Err(ret);
        }

        //let instance = unsafe { instance.as_mut().expect("Instance is null!") };

        let mut session = 0u32;

        let ret = unsafe {
            nifpga::NiFpga_Open(
                args.bitfile,
                args.signature,
                args.resource,
                args.attribute,
                &mut session as *mut _
            )
        };

        if ret != nifpga::NiFpga_Status_Success {
            return Err(ret);
        }

        let addrs = if !args.addrs.is_null() {
            unsafe { std::ptr::slice_from_raw_parts(args.addrs, n_channels).as_ref().unwrap().to_vec() }
        } else {
            (0..n_channels as u32).into_iter().collect()
        };

        let out_file = if dump {
            let tmp = std::env::var("TEMP").unwrap_or("/tmp".to_string());
            let mut tmp = std::path::PathBuf::from(tmp);
            tmp.push("aoldaq-nifpga-out.log");
            OpenOptions::new()
                .truncate(true)
                .write(true)
                .create(true)
                .open(tmp)
                .map(|f| Some(Mutex::new(f)))
                .unwrap_or(None)
        } else {
            None
        };

        let counters = ( 0..n_channels ).into_iter().map(|_| Arc::new(AtomicUsize::new(0))).collect::<Vec<_>>();

        Ok(NiFpgaDevice {
            session,
            addrs,
            out_file,
            counters,
        })
    }

    pub fn get_nifpga_session(&self) -> nifpga::NiFpga_Session {
        self.session
    }
}

impl Drop for NiFpgaDevice {
    fn drop(&mut self) {
        log::debug!("NiFpga device wrote {} {}", self.counters[0].load(Ordering::Relaxed), self.counters[1].load(Ordering::Relaxed));
        unsafe { nifpga::NiFpga_Close(self.session, 0) }; // TODO fix attribute
        unsafe { nifpga::NiFpga_Finalize() };
    }
}

impl Device for NiFpgaDevice {
    fn read_data(&self, channel: usize, n: usize) -> Vec<u32> {
        let mut buf = vec![42; n];

        unsafe {
            nifpga::NiFpga_ReadFifoU32(
                self.session,
                self.addrs[channel],
                buf.as_mut_ptr() as *mut _,
                n as u64,
                0,
                std::ptr::null_mut()
            )
        };

        buf
    }

    fn read_into(&self, channel: usize, buf: &mut [u32]) -> Result<usize, i32> {
        // TODO uncomment this after debugging!
        let ret = unsafe {
            nifpga::NiFpga_ReadFifoU32(
                self.session,
                self.addrs[channel],
                buf.as_mut_ptr() as *mut _,
                buf.len() as u64,
                nifpga::NiFpga_InfiniteTimeout,
                std::ptr::null_mut()
            )
        };

        if ret == nifpga::NiFpga_Status_Success {
            //for i in buf.iter_mut() {
            // This only runs on one thread, so relaxed is fine
            //*i = ( self.counters[channel].fetch_add(1, Ordering::Relaxed) % (512*512) ) as u32 + 1; 
            //}

            // This whole thing is horrible and slow.
            // This must only be used for debugging.
            if let Some(ref mutex) = self.out_file {
                let mut file = mutex.lock().unwrap();
                writeln!(file, "Channel {}: {:?}", channel, buf).unwrap();
                file.flush().unwrap();
            }

            Ok(buf.len())
        } else {
            Err(ret)
        }
    }

    fn poll(&self, channel: usize) -> Option<usize> {
        let mut n = 0u64;

        unsafe {
            nifpga::NiFpga_ReadFifoU32(self.session, 
                                       self.addrs[channel],
                                       std::ptr::null_mut(),
                                       0 as u64,
                                       nifpga::NiFpga_InfiniteTimeout,
                                       &mut n as *mut _,
                                       );
        }

        Some(n as usize)
    }
}
