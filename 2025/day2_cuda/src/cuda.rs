mod abi {
    #![allow(nonstandard_style)]

    use std::ffi::{CStr, c_char, c_int, c_void};

    // 4 byte mystery value
    pub type cudaError_t = u32;
    pub type cudaStream_t = *mut c_void;
    pub type cudaEvent_t = *mut c_void;

    #[repr(C)]
    #[derive(Debug)]
    pub struct maybeCudaStream {
        pub err: cudaError_t,
        pub stream: cudaStream_t,
    }

    #[repr(C)]
    #[derive(Debug)]
    pub struct maybeCudaBuffer {
        pub err: cudaError_t,
        pub buffer_size: usize,
        pub buffer: *mut u64,
        pub event: cudaEvent_t,
    }

    #[repr(C)]
    #[derive(Debug)]
    pub struct maybeKernelResults {
        pub err: cudaError_t,
        pub cuda_buffer: *mut u64,
        pub event: cudaEvent_t,
    }

    #[repr(C)]
    #[derive(Debug)]
    pub struct maybeSum {
        pub err: cudaError_t,
        pub sum: u64,
    }

    #[link(name = "gpu_proc", kind = "static")]
    unsafe extern "C" {
        pub fn init_stream() -> maybeCudaStream;

        pub fn destroy_stream(stream: cudaStream_t) -> cudaError_t;

        pub fn copy_to_device(
            stream: cudaStream_t,
            size: usize,
            data: *const u64,
        ) -> maybeCudaBuffer;

        pub fn destroy_buffer(stream: cudaStream_t, buffer: *mut u64) -> cudaError_t;

        pub fn block_and_destroy_event(event: cudaEvent_t) -> cudaError_t;

        pub fn sum(stream: cudaStream_t, res: *const u64, buffer_size: usize) -> maybeSum;

        pub fn run_part1(stream: cudaStream_t, init: *const maybeCudaBuffer) -> maybeKernelResults;

        pub fn cuda_err_string(error: cudaError_t) -> *const c_char;

        pub fn cuda_success_value() -> cudaError_t;
    }
}

use std::{
    ffi::CStr,
    sync::{LazyLock, OnceLock},
};

use abi::*;

fn cuda_err_res(cuda_err: cudaError_t) -> Result<(), &'static str> {
    // SAFETY: Inner function is trival.
    static SUCCESS_VAL: LazyLock<cudaError_t> = LazyLock::new(|| {
        let val = unsafe { cuda_success_value() };
        assert_eq!(val, 0, "Cuda success value changed from 0 to {val}");
        val
    });

    if cuda_err == *SUCCESS_VAL {
        Ok(())
    } else {
        Err(unsafe { CStr::from_ptr(cuda_err_string(cuda_err)) }
            .to_str()
            .expect("If this failed, Nvidia gave an invalid C string."))
    }
}

/// SAFETY: Caller must clean up the stream.
unsafe fn new_stream() -> Result<cudaStream_t, &'static str> {
    let stream_res = unsafe { init_stream() };
    cuda_err_res(stream_res.err)?;
    Ok(stream_res.stream)
}

#[derive(Debug)]
pub struct InitStream<'a> {
    stream: cudaStream_t,
    // Binding data to be unmodified and unmoved.
    #[allow(unused)]
    data: &'a [u64],
    init_buffer: maybeCudaBuffer,
}

impl<'a> InitStream<'a> {
    pub fn init(data: &'a [u64]) -> Result<Self, &'static str> {
        // SAFETY: This only returns a value on non-error.
        // The allocation is cleaned up in drop or next op failure.
        let stream = unsafe { new_stream() }?;

        let init_buffer = {
            // SAFETY: This only returns a value on non-error.
            // SAFETY: data does not move while this struct exists.
            // The allocation is cleaned up in drop.
            let copy_res = unsafe { copy_to_device(stream, data.len(), data.as_ptr()) };

            let copy_err_res = cuda_err_res(copy_res.err);
            if copy_err_res.is_err() {
                // SAFETY: Previously initialized in the same function.
                unsafe { destroy_stream(stream) };
                copy_err_res?;
            }

            copy_res
        };

        Ok(Self {
            stream,
            data,
            init_buffer,
        })
    }
}

impl Drop for InitStream<'_> {
    fn drop(&mut self) {
        // Clean up all C mallocs
        // SAFETY: All of this was initialized in constructor

        unsafe { block_and_destroy_event(self.init_buffer.event) };
        unsafe { destroy_buffer(self.stream, self.init_buffer.buffer) };
        // Important that this is last; it invalidates the stream.
        unsafe { destroy_stream(self.stream) };
    }
}

#[derive(Debug)]
pub struct Task<'a> {
    stream: cudaStream_t,
    task: maybeKernelResults,
    init: &'a InitStream<'a>,
}

impl<'a> Task<'a> {
    fn create(
        init: &'a InitStream<'a>,
        func: unsafe extern "C" fn(
            stream: cudaStream_t,
            init: *const maybeCudaBuffer,
        ) -> maybeKernelResults,
    ) -> Result<Self, &'static str> {
        // SAFETY: This only returns a value on non-error.
        // The allocation is cleaned up in drop or next op failure.
        let stream = unsafe { new_stream() }?;

        let task = {
            // SAFETY: This only returns a value on non-error.
            // SAFETY: data does not move while this struct exists.
            // The allocation is cleaned up in drop.
            let kernel_res = unsafe { (func)(stream, &init.init_buffer) };

            let kernel_err_res = cuda_err_res(kernel_res.err);
            if kernel_err_res.is_err() {
                // SAFETY: Previously initialized in the same function.
                unsafe { destroy_stream(stream) };
                kernel_err_res?;
            }

            kernel_res
        };

        Ok(Self { stream, task, init })
    }

    pub fn part1(init: &'a InitStream<'a>) -> Result<Self, &'static str> {
        Self::create(init, run_part1)
    }

    pub fn resolve(self) -> Result<u64, &'static str> {
        // SAFETY: This only returns a value on non-error.
        // SAFETY: This cleans up its own allocations.
        let res = unsafe {
            sum(
                self.stream,
                self.task.cuda_buffer,
                self.init.init_buffer.buffer_size,
            )
        };
        cuda_err_res(res.err)?;
        Ok(res.sum)
    }
}

impl Drop for Task<'_> {
    fn drop(&mut self) {
        // Clean up all C mallocs
        // SAFETY: All of this was initialized in constructor

        unsafe { block_and_destroy_event(self.task.event) };
        unsafe { destroy_buffer(self.stream, self.task.cuda_buffer) };
        // Important that this is last; it invalidates the stream.
        unsafe { destroy_stream(self.stream) };
    }
}
