#![feature(specialization)]
#![warn(missing_docs)]

//! A simple PNG serialization library.

extern crate lodepng;
extern crate framing;

use framing::{Image, Rgba, Chunky};
use lodepng::ffi::CVec;
use std::{mem, slice};
use std::path::Path;

/// Decodes the PNG represented by the given bytes into an RGBA image.
pub fn decode(bytes: &[u8]) -> Result<Chunky<Rgba, Native>, Error> {
    match lodepng::decode32(bytes) {
        Ok(bmp) => {
            assert_eq!(bmp.buffer.len(), bmp.width * bmp.height);
            Ok(Chunky::from_bytes(
                bmp.width,
                bmp.height,
                Native::new(bmp.buffer)
            ))
        },
        Err(_) => Err(Error)
    }
}

/// Loads and decodes at the given file path.
pub fn load<P: AsRef<Path>>(path: P) -> Result<Chunky<Rgba, Native>, Error> {
    match lodepng::decode32_file(path) {
        Ok(bmp) => {
            assert_eq!(bmp.buffer.len(), bmp.width * bmp.height);
            Ok(Chunky::from_bytes(
                bmp.width,
                bmp.height,
                Native::new(bmp.buffer)
            ))
        },
        Err(_) => Err(Error)
    }
}

/// Saves the image to the given file path as a PNG.
///
/// **Any existing file at the path will be overwritten.** This is basically
/// the same as encoding the image and writing it yourself, but is a lot
/// more convenient.
pub fn save<P, T>(image: &Chunky<Rgba, T>, path: P) -> Result<(), Error>
where
    P: AsRef<Path>,
    T: AsRef<[u8]>
{
    let result = lodepng::encode32_file(
        path,
        image.bytes().as_ref(),
        image.width(),
        image.height()
    );

    match result {
        Ok(_) => Ok(()),
        Err(_) => Err(Error)
    }
}

/// Encodes the image as a PNG, allocating memory for the encoded data.
///
/// The output is a dynamically-allocated array of bytes containing the
/// compressed PNG data, suitable for sending over a network, saving as a file,
/// or writing to `/dev/null`.
pub fn encode<T>(image: &Chunky<Rgba, T>) -> Result<CVec<u8>, Error>
where
    T: AsRef<[u8]>
{
    let result = lodepng::encode32(
        image.bytes().as_ref(),
        image.width(),
        image.height()
    );

    match result {
        Ok(vec) => Ok(vec),
        Err(_) => Err(Error)
    }
}

/// A native C pixel array, allocated using malloc.
///
/// You probably won't have to worry about this struct, since it's just an
/// implementation detail.
pub struct Native(CVec<lodepng::RGBA<u8>>, usize);

impl Native {
    fn new(buffer: CVec<lodepng::RGBA<u8>>) -> Self {
        let length = buffer.len() * mem::size_of::<lodepng::RGBA<u8>>();
        Native(buffer, length)
    }
}

impl AsRef<[u8]> for Native {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                self.0.as_ref().as_ptr() as *const _ as *const _,
                self.1
            )
        }
    }
}

/// An unknown error.
///
/// Usually the error is obvious, though. For example, when decoding, the error
/// was probably caused by an invalid PNG. In other cases, the error's source
/// might be ambiguous, in which case you're out of luck.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Error;

#[test]
fn lossless() {
    use framing::{Chunky, Function};

    let ugly_gradient = Function::new(1920, 1080, |x, y| {
        Rgba((x % 256) as u8, (y % 256) as u8, 0, 255)
    });

    let png = Chunky::new(ugly_gradient);
    let recoded = decode(encode(&png).unwrap().as_ref()).unwrap();

    let before = png.bytes().as_slice();
    let after  = recoded.bytes().as_ref();

    assert_eq!(before, after);
}
