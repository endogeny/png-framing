extern crate framing;
extern crate png_framing;

use framing::{Function, Rgba};
use png_framing::Png;

fn main() {
    let (w, h): (usize, usize) = (7680, 4320);
    let scale = 3.0 / w as f64;

    let image = Function::new(w, h, |x, y| {
        let x = scale * (x as f64) - 2.0;
        let y = scale * (y as f64 - h as f64 / 2.0);

        if let Some(n) = mandelbrot(x, y) {
            let lum = (n * 255.0) as u8;
            Rgba(0, lum, 0, 255)
        } else {
            Rgba(0, 0, 0, 255)
        }
    });

    match Png::new(image).save("mandelbrot.png") {
        Ok(_) => println!("Image saved to `mandelbrot.png`!"),
        Err(_) => println!("Could not save image.")
    }
}

fn mandelbrot(x: f64, y: f64) -> Option<f64> {
    const MAX: usize = 256;

    let mut i = 0;
    let (mut u, mut v, mut u2, mut v2) = (0.0, 0.0, 0.0, 0.0);

    while i <= MAX && u2 + v2 < 4.0 {
        v = 2.0 * u * v + y;
        u = u2 - v2 + x;
        u2 = u * u;
        v2 = v * v;

        i += 1;
    }

    if i <= MAX {
        Some(i as f64 / MAX as f64)
    } else {
        None
    }
}
