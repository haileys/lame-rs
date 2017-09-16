#[macro_use]
extern crate error_chain;

mod ffi;

use std::ptr;
use std::ops::Drop;
use ffi::LamePtr;
use std::os::raw::c_int;
use std::convert::From;

error_chain! {
    errors {
        Uninitialized { }
        GenericError { }
        NoMem { }
        BadBitRate { }
        BadSampleFreq { }
        InternalError { }
        Unknown(errcode: c_int) { }
    }
}

impl From<c_int> for ErrorKind {
    fn from(errcode: c_int) -> ErrorKind {
        match errcode {
            -1 => ErrorKind::GenericError,
            -10 => ErrorKind::NoMem,
            -11 => ErrorKind::BadBitRate,
            -12 => ErrorKind::BadSampleFreq,
            -13 => ErrorKind::InternalError,
            e => ErrorKind::Unknown(e),
        }
    }
}

pub mod encode {
    use super::c_int;

    error_chain! {
        errors {
            OutputBufferTooSmall { }
            NoMem { }
            InitParamsNotCalled { }
            PsychoAcousticError { }
            Unknown(errcode: c_int) { }
        }
    }

    impl From<c_int> for ErrorKind {
        fn from(errcode: c_int) -> ErrorKind {
            match errcode {
                -1 => ErrorKind::OutputBufferTooSmall,
                -2 => ErrorKind::NoMem,
                -3 => ErrorKind::InitParamsNotCalled,
                -4 => ErrorKind::PsychoAcousticError,
                e => ErrorKind::Unknown(e),
            }
        }
    }
}

macro_rules! check_error {
    ($retn:expr) => {
        if $retn == 0 {
            Ok(())
        } else {
            Err(ErrorKind::from($retn).into())
        }
    }
}

pub struct Lame {
    ptr: LamePtr,
}

impl Lame {
    pub fn new() -> Result<Lame> {
        let ctx = unsafe { ffi::lame_init() };

        if ctx == ptr::null_mut() {
            Err(ErrorKind::Uninitialized.into())
        } else {
            Ok(Lame { ptr: ctx })
        }
    }

    pub fn sample_rate(&self) -> u32 {
        unsafe { ffi::lame_get_in_samplerate(self.ptr) as u32 }
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) -> Result<()> {
        check_error!(unsafe {
            ffi::lame_set_in_samplerate(self.ptr, sample_rate as c_int) })
    }

    pub fn channels(&self) -> u8 {
        unsafe { ffi::lame_get_num_channels(self.ptr) as u8 }
    }

    pub fn set_channels(&mut self, channels: u8) -> Result<()> {
        check_error!(unsafe {
            ffi::lame_set_num_channels(self.ptr, channels as c_int) })
    }

    pub fn quality(&self) -> u8 {
        unsafe { ffi::lame_get_quality(self.ptr) as u8 }
    }

    pub fn set_quality(&self, quality: u8) -> Result<()> {
        check_error!(unsafe {
            ffi::lame_set_quality(self.ptr, quality as c_int) })
    }

    pub fn init_params(&mut self) -> Result<()> {
        check_error!(unsafe {
            ffi::lame_init_params(self.ptr) })
    }

    pub fn encode(&self, pcm_left: &[i16], pcm_right: &[i16], mp3_buffer: &mut [u8]) -> encode::Result<usize> {
        if pcm_left.len() != pcm_right.len() {
            panic!("left and right channels must have same number of samples!");
        }

        let retn = unsafe {
            ffi::lame_encode_buffer(self.ptr,
                pcm_left.as_ptr(), pcm_right.as_ptr(), pcm_left.len() as c_int,
                mp3_buffer.as_mut_ptr(), mp3_buffer.len() as c_int)
        };

        if retn < 0 {
            Err(encode::ErrorKind::from(retn).into())
        } else {
            Ok(retn as usize)
        }

    }
}

impl Drop for Lame {
    fn drop(&mut self) {
        unsafe { ffi::lame_close(self.ptr) };
    }
}
