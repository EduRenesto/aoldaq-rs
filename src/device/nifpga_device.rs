use super::Device;
use crate::NiFpgaArgs;
use crate::nifpga;

pub struct NiFpgaDevice {
    session: nifpga::NiFpga_Session,
    addrs: Vec<u32>,
}

impl NiFpgaDevice {
    pub fn new(args: *const NiFpgaArgs, n_channels: usize) -> Result<NiFpgaDevice, nifpga::NiFpga_Status> {
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

        Ok(NiFpgaDevice {
            session,
            addrs,
        })
    }

    pub fn get_nifpga_session(&self) -> nifpga::NiFpga_Session {
        self.session
    }
}

impl Drop for NiFpgaDevice {
    fn drop(&mut self) {
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

    fn read_into(&self, channel: usize, buf: &mut [u32]) -> usize {
        unsafe {
            nifpga::NiFpga_ReadFifoU32(
                self.session,
                self.addrs[channel],
                buf.as_mut_ptr() as *mut _,
                buf.len() as u64,
                0,
                std::ptr::null_mut()
            );
        }

        buf.len()
    }
}
