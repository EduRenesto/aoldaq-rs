pub mod nifpga_device;
pub use nifpga_device::NiFpgaDevice;

pub trait Device: Send + Sync {
    fn read_data(&self, channel: usize, n: usize) ->  Vec<u32>;
    fn read_into(&self, channel: usize, buf: &mut [u32]) -> usize;
}

pub struct RandomDevice;

impl RandomDevice {
    pub fn new() -> RandomDevice {
        RandomDevice
    }
}

impl Device for RandomDevice {
    fn read_data(&self, _channel: usize, n: usize) -> Vec<u32> {
        (0..n).into_iter().map(|_| rand::random()).collect()
    }

    fn read_into(&self, channel: usize, buf: &mut [u32]) -> usize {
        for i in buf.iter_mut() {
            *i = rand::random();
        }

        buf.len()
    }
}

