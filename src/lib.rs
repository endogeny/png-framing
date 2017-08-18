#![feature(specialization)]
#![warn(missing_docs)]

//! A simple PNG serialization library.
//!
//! # A (Tiny) Example
//!
//! ```rust,no_run
//! use png_framing::Png;
//!
//! // A tiny image.
//! let bytes = vec![255, 0, 0, 255, 0, 0, 255, 255]; 
//! let (width, height) = (2, 1);
//!
//! // Save it!
//! Png::from_bytes(width, height, bytes).save("sollux.png");
//! ```

extern crate lodepng;
extern crate framing;

use framing::{Image, Rgba, Chunky};
use lodepng::ffi::CVec;
use std::{mem, ptr, slice};
use std::path::Path;

/// A raw RGBA image that can be converted easily to/from a PNG.
pub struct Png<T> {
    width: usize,
    height: usize,
    buffer: T
}

impl Png<Native> {
    /// Decodes the image which has been encoded in the given bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        match lodepng::decode32(bytes) {
            Ok(bmp) => {
                assert_eq!(bmp.buffer.len(), bmp.width * bmp.height);
                Ok(Png {
                    width: bmp.width,
                    height: bmp.height,
                    buffer: Native::new(bmp.buffer)
                })
            },
            Err(_) => Err(Error)
        }
    }

    /// Loads the PNG at the given file path.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        match lodepng::decode32_file(path) {
            Ok(bmp) => {
                assert_eq!(bmp.buffer.len(), bmp.width * bmp.height);
                Ok(Png {
                    width: bmp.width,
                    height: bmp.height,
                    buffer: Native::new(bmp.buffer)
                })
            },
            Err(_) => Err(Error)
        }
    }
}

impl<T> Png<T> {
    /// Borrows the buffer that the PNG was created with.
    pub fn buffer(&self) -> &T {
        &self.buffer
    }

    /// Recovers the buffer that the PNG was created with.
    ///
    /// This operation destroys the PNG, since mutations to the buffer could
    /// alter the length of the buffer, and thus lead to undefined behavior.
    pub fn into_buffer(self) -> T {
        self.buffer
    }
}

impl<T> Png<T> where T: AsRef<[u8]> {
    /// Creates a new PNG given the width, height, and raw RGBA image data.
    ///
    /// # Panics
    ///
    /// Panics if the buffer's length is not exactly `width * height * 4`.
    pub fn from_bytes(
        width: usize,
        height: usize,
        buffer: T
    ) -> Png<T> {
        assert_eq!(width * height * 4, buffer.as_ref().len());
        Png { width, height, buffer }
    }

    /// Saves the PNG to the given file path.
    ///
    /// **Any existing file at the path will be overwritten.** This is basically
    /// the same as encoding the image and writing it yourself, but is a lot
    /// more convenient.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let result = lodepng::encode32_file(
            path,
            self.buffer.as_ref(),
            self.width,
            self.height
        );

        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(Error)
        }
    }

    /// Encodes the PNG, allocating the necessary memory for the encoded data.
    ///
    /// The output is an array of bytes with the compressed PNG data, suitable
    /// for sending over a network or writing to a file.
    pub fn encode(&self) -> Result<CVec<u8>, Error> {
        let result = lodepng::encode32(
            self.buffer.as_ref(),
            self.width,
            self.height
        );

        match result {
            Ok(vec) => Ok(vec),
            Err(_) => Err(Error)
        }
    }
}

impl Png<Vec<u8>> {
    /// Creates a new image from the given frame.
    ///
    /// Note that **this function allocates**, since the underlying library,
    /// `lodepng`, can unfortunately only operate on a contiguous buffer. Maybe
    /// when I get good enough to write my own encoder, this won't have to
    /// allocate any memory.
    pub fn new<T>(frame: T) -> Self
    where T: Image + Sync, T::Pixel: Into<Rgba> {
        Chunky::new(framing::map(|x| x.into(), frame)).into()
    }
}

impl From<Chunky<Rgba>> for Png<Vec<u8>> {
    fn from(frame: Chunky<Rgba>) -> Self {
        Png {
            width: frame.width(),
            height: frame.height(),
            buffer: frame.into_bytes()
        }
    }
}

impl<T> AsRef<[u8]> for Png<T> where T: AsRef<[u8]> {
    fn as_ref(&self) -> &[u8] {
        self.buffer.as_ref()
    }
}

impl<T> AsMut<[u8]> for Png<T> where T: AsMut<[u8]> {
    fn as_mut(&mut self) -> &mut [u8] {
        self.buffer.as_mut()
    }
}

impl<T> Image for Png<T> where T: AsRef<[u8]> {
    type Pixel = Rgba;

    fn width(&self) -> usize { self.width }
    fn height(&self) -> usize { self.height }

    unsafe fn pixel(&self, x: usize, y: usize) -> Self::Pixel {
        let mut bytes: [u8; 4] = mem::uninitialized();
        let offset = 4 * (y * self.width + x) as isize;

        ptr::copy_nonoverlapping(
            self.buffer.as_ref().as_ptr().offset(offset),
            bytes.as_mut_ptr(),
            4
        );

        bytes.into()
    }
}

/// A native C pixel array, allocated using malloc.
///
/// You probably won't have to worry about this struct, since it's just an
/// implementation detail. But if you see a Png<Native>, bear in mind that it
/// was created by the `lodepng` C library.
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
    use framing::Function;

    let ugly_gradient = Function::new(1920, 1080, |x, y| {
        Rgba((x % 256) as u8, (y % 256) as u8, 0, 255)
    });

    let png = Png::new(ugly_gradient);
    let recoded = Png::decode(png.encode().unwrap().as_ref()).unwrap();

    let before = png.buffer().as_slice();
    let after  = recoded.buffer().as_ref();

    assert_eq!(before, after);
}
