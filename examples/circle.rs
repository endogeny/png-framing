extern crate framing;
extern crate png_framing;

use framing::video::{ChunkyFrame, Function, Rgba};
use png_framing::Png;

fn main() {
    let image = ChunkyFrame::new(
        Function::new(512, 512, |x, y| {
            let x = x as f64 / 256.0 - 1.0;
            let y = y as f64 / 256.0 - 1.0;
            let z = x * x + y * y;

            if z <= 0.5 {
                Rgba(0, 0, 0, 255)
            } else if z <= 0.505 {
                Rgba(0, 255, 0, 255)
            } else {
                Rgba(255, 255, 255, 255)
            }
        })
    );

    match Png::from(image).save("circle.png") {
        Ok(_) => println!("Image saved to `circle.png`!"),
        Err(_) => println!("Could not save image.")
    }
}
