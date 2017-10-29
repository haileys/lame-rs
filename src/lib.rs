mod ffi;

use std::ptr;
use std::ops::Drop;
use ffi::LamePtr;
use std::os::raw::c_int;
use std::convert::From;

#[derive(Debug)]
pub enum Error {
    Ok,
    GenericError,
    NoMem,
    BadBitRate,
    BadSampleFreq,
    InternalError,
    Unknown(c_int),
}

impl From<c_int> for Error {
    fn from(errcode: c_int) -> Error {
        match errcode {
            0 => Error::Ok,
            -1 => Error::GenericError,
            -10 => Error::NoMem,
            -11 => Error::BadBitRate,
            -12 => Error::BadSampleFreq,
            -13 => Error::InternalError,
            _ => Error::Unknown(errcode),
        }
    }
}

fn handle_simple_error(retn: c_int) -> Result<(), Error> {
    match retn.into() {
        Error::Ok => Ok(()),
        err => Err(err),
    }
}

#[derive(Debug)]
pub enum EncodeError {
    OutputBufferTooSmall,
    NoMem,
    InitParamsNotCalled,
    PsychoAcousticError,
}

pub struct Lame {
    ptr: LamePtr,
}

impl Lame {
    pub fn new() -> Option<Lame> {
        let ctx = unsafe { ffi::lame_init() };

        if ctx == ptr::null_mut() {
            None
        } else {
            Some(Lame { ptr: ctx })
        }
    }

    pub fn sample_rate(&self) -> u32 {
        unsafe { ffi::lame_get_in_samplerate(self.ptr) as u32 }
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) -> Result<(), Error> {
        handle_simple_error(unsafe {
            ffi::lame_set_in_samplerate(self.ptr, sample_rate as c_int) })
    }

    pub fn channels(&self) -> u8 {
        unsafe { ffi::lame_get_num_channels(self.ptr) as u8 }
    }

    pub fn set_channels(&mut self, channels: u8) -> Result<(), Error> {
        handle_simple_error(unsafe {
            ffi::lame_set_num_channels(self.ptr, channels as c_int) })
    }

    pub fn quality(&self) -> u8 {
        unsafe { ffi::lame_get_quality(self.ptr) as u8 }
    }

    pub fn set_quality(&mut self, quality: u8) -> Result<(), Error> {
        handle_simple_error(unsafe {
            ffi::lame_set_quality(self.ptr, quality as c_int) })
    }

    pub fn kilobitrate(&self) -> i32 {
        unsafe { ffi::lame_get_brate(self.ptr) as i32 }
    }

    pub fn set_kilobitrate(&mut self, quality: i32) -> Result<(), Error> {
        handle_simple_error(unsafe {
            ffi::lame_set_brate(self.ptr, quality as c_int) })
    }

    pub fn init_params(&mut self) -> Result<(), Error> {
        handle_simple_error(unsafe {
            ffi::lame_init_params(self.ptr) })
    }

    pub fn encode(&self, pcm_left: &[i16], pcm_right: &[i16], mp3_buffer: &mut [u8]) -> Result<usize, EncodeError> {
        if pcm_left.len() != pcm_right.len() {
            panic!("left and right channels must have same number of samples!");
        }

        let retn = unsafe {
            ffi::lame_encode_buffer(self.ptr,
                pcm_left.as_ptr(), pcm_right.as_ptr(), pcm_left.len() as c_int,
                mp3_buffer.as_mut_ptr(), mp3_buffer.len() as c_int)
        };

        match retn {
            -1 => Err(EncodeError::OutputBufferTooSmall),
            -2 => Err(EncodeError::NoMem),
            -3 => Err(EncodeError::InitParamsNotCalled),
            -4 => Err(EncodeError::PsychoAcousticError),
            sz => Ok(sz as usize),
        }
    }
}

impl Drop for Lame {
    fn drop(&mut self) {
        unsafe { ffi::lame_close(self.ptr) };
    }
}
