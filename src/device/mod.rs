pub mod nifpga_device;
pub use nifpga_device::NiFpgaDevice;

pub trait Device: Send + Sync {
    fn read_data(&self, channel: usize, n: usize) ->  Vec<u32>;
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
}

