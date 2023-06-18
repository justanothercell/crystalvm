use std::io::{Error, BufReader, Write};

use image::GenericImageView;

fn main() -> Result<(), Error> {
    let imgfile = std::fs::File::open("font.png")?;
    let image = image::load(BufReader::new(imgfile), image::ImageFormat::Png).unwrap();
    let mut data = [0u8; 256*8*8];
    for cy in 0..16 {
        for cx in 0..16 {
            let c = cy * 16 + cx;
            for dy in 0..8 {
                for dx in 0..8 {
                    let l = image.get_pixel(cx * 8 + dx, cy * 8 + dy);
                    data[(c * 64 + dy * 8 + dx) as usize] = l[0];
                }
            }
        }
    }
    let mut outfile = std::fs::File::create("target/font.rbmf")?;
    outfile.write_all(&data)?;
    Ok(())
}