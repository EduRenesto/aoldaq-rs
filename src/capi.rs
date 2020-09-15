use super::{ Aoldaq, AoldaqArgs };

/// Creates an AOLDAQ instance.
#[no_mangle]
pub unsafe extern fn aoldaq_create_instance(args: *const AoldaqArgs) -> *mut Aoldaq {
    let instance = Box::new(Aoldaq::create(&*args));
    Box::into_raw(instance)
}

/// Destroys an AOLDAQ instance, stopping the threads and dropping everything.
#[no_mangle]
pub unsafe extern fn aoldaq_destroy_instance(instance: *mut Aoldaq) {
    if !instance.is_null() {
        let instance = Box::from_raw(instance);
        drop(instance)
    }
}

/// Tries to return `n` `uint32_t`s of data, returning 0 if unsuccesufl.
/// Assumes that `buf` is a preallocated buffer capable of receiving all the data.
#[no_mangle]
pub extern fn aoldaq_get_data(instance: *mut Aoldaq, channel: usize, n: usize, buf: *mut u32) -> usize {
    let instance = unsafe { instance.as_mut().expect("Instance is null!") };

    // Just return the amount of data in the fifo
    if n == 0 {
        return instance.get_fifo_size(channel);
    }

    //let data = instance.get_data(channel, n);

    //match data {
        //Some(data) => {
            //let n = data.len();
            //let ptr = unsafe { std::slice::from_raw_parts_mut(buf, n) };
            //ptr.copy_from_slice(&data[..]);
            //n
        //},
        //None => 0
    //}

    let ptr = unsafe { std::slice::from_raw_parts_mut(buf, n) };

    instance.get_data_into(channel, ptr)
}

/// Consumes and frees everything in the specified channel.
#[no_mangle]
pub extern fn aoldaq_flush_fifo(instance: *mut Aoldaq, channel: usize) {
    let instance = unsafe { instance.as_mut().expect("Instance is null!") };

    instance.flush_fifo(channel);
}

/// Unparks the threads and starts the acquisition.
#[no_mangle]
pub extern fn aoldaq_start(instance: *mut Aoldaq) {
    let instance = unsafe { instance.as_mut().expect("Instance is null!") };
    instance.start();
}

/// Parks the threads, pausing the acquisition.
#[no_mangle]
pub extern fn aoldaq_stop(instance: *mut Aoldaq) {
    let instance = unsafe { instance.as_mut().expect("Instance is null!") };
    instance.stop();
}

/// Returns the underlying NiFPGA session object.
#[no_mangle]
pub extern fn aoldaq_get_nifpga_session(instance: *mut Aoldaq) -> u32 {
    let instance = unsafe { instance.as_mut().expect("Instance is null!") };
    instance.get_nifpga_session().unwrap_or(0)
}
