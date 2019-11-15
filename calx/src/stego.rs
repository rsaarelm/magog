//! Steganographic data encoding

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use crc32fast::Hasher;
use image::{GenericImage, Pixel, Rgb, RgbImage};
use libflate::gzip;
use log::{info, warn};
use std::io::{self, Cursor, Read};

// The algorithm is hardcoded to a somewhat noisy version that uses the 3 low bits in R and B
// channels and 2 low bits in G, packing one byte into each pixel. It could be rewritten to be more
// stealthy and pack just a nibble in each pixel with 1 bit in R and G and 2 bits in B channel, in
// exchange for needing twice the number of pixels for the same data. (Going by the idea that
// humans eyes have worse blue-sensitivity than red or green sensitivity by always making the B
// channel use more bits.)

// Magic numbers for identifying uncompressed and gzip-compressed stego files.
const STGO_MAGIC: u32 = 0x5354_474F;
const STGZ_MAGIC: u32 = 0x5354_475A;

fn encode_capacity(w: u32, h: u32) -> usize {
    // One byte per pixel encoding.
    (w * h) as usize
}

fn embed_raw(
    cover: &impl GenericImage<Pixel = Rgb<u8>>,
    data: &[u8],
) -> RgbImage {
    info!(
        "Embedding {} bytes into {} x {} cover",
        data.len(),
        cover.width(),
        cover.height()
    );
    // Grow image by integer multiples if it doesn't look like it'll fit the data.
    let mut scale = 1;

    while encode_capacity(cover.width() * scale, cover.height() * scale)
        < data.len()
    {
        scale += 1;
    }

    if scale > 1 {
        info!("Enlarging image cover by {}x", scale);
    }

    let mut result = RgbImage::from_fn(
        cover.width() * scale,
        cover.height() * scale,
        |x, y| cover.get_pixel(x / scale, y / scale).to_rgb(),
    );

    // The data is made to repeat with .cycle() so that it covers the entire image surface.
    // Dropping out halfway can look very conspicuous on the resulting image.
    for (p, b) in result.pixels_mut().zip(data.iter().cycle()) {
        // Zero out low bits from R, G, B channels.
        // Use less bits from green channel because humans are sensitive to that.
        p.0[0] &= 0b1111_1000;
        p.0[1] &= 0b1111_1100;
        p.0[2] &= 0b1111_1000;

        // Embed data bits in the zeroed out space.
        p.0[0] |= b >> 5u8;
        p.0[1] |= (b >> 3u8) & 0b0000_0011;
        p.0[2] |= b & 0b0000_0111;
    }

    result
}

struct StegRead<I> {
    pixels: I,
}

impl<I> Iterator for StegRead<I>
where
    I: Iterator<Item = (u32, u32, Rgb<u8>)>,
{
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        match self.pixels.next() {
            Some((_, _, p)) => Some(
                (p.0[0] << 5u8)
                    | ((p.0[1] & 0b0000_0011) << 3u8)
                    | (p.0[2] & 0b0000_0111),
            ),
            None => None,
        }
    }
}

impl<I> Read for StegRead<I>
where
    I: Iterator<Item = (u32, u32, Rgb<u8>)>,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        for (i, a) in buf.iter_mut().enumerate() {
            if let Some(b) = self.next() {
                *a = b;
            } else {
                return Ok(i);
            }
        }
        Ok(buf.len())
    }
}

fn embed_base(
    cover: &impl GenericImage<Pixel = Rgb<u8>>,
    magic_number: u32,
    data: &[u8],
) -> RgbImage {
    let mut hasher = Hasher::new();
    hasher.update(data);
    let checksum = hasher.finalize();

    let mut buf = Cursor::new(Vec::new());
    let _ = buf.write_u32::<BigEndian>(magic_number);
    let _ = buf.write_u32::<BigEndian>(checksum);
    let _ = buf.write_u32::<BigEndian>(data.len() as u32);
    let mut buf = buf.into_inner();
    buf.extend_from_slice(data);
    embed_raw(cover, &buf)
}

/// Embed uncompressed binary data into an image.
///
/// You should use `embed_gzipped` instead unless you know your data is noise-like. Data
/// regularities will show as visible artifacts in the steganographic image.
///
/// Data is written one byte per pixel in the 3, 2 and 3 lowest bits of the R, G and B channels
/// respectively.
///
/// Data format:
/// ```notrust
/// 0:  'STGO'
/// 4:  big-endian u32 crc32 checksum of data payload bytes
/// 8:  big-endian u32 data payload byte count
/// 12: data payload bytes
/// ```
pub fn embed(
    cover: &impl GenericImage<Pixel = Rgb<u8>>,
    data: &[u8],
) -> RgbImage {
    embed_base(cover, STGO_MAGIC, data)
}

/// Embed given data compressed with gzip into an image.
///
/// The input data will start from the top left and go left to right along horizontal lines of the
/// image and will repeat over the entire surface of the generated image. If the image is too small
/// for the amount of data encoded, it will be grown until it fits the data.
///
/// Data is written one byte per pixel in the 3, 2 and 3 lowest bits of the R, G and B channels
/// respectively.
///
/// Data format:
/// ```notrust
/// 0:  'STGZ'
/// 4:  big-endian u32 crc32 checksum of gzipped data payload bytes
/// 8:  big-endian u32 gzipped data payload byte count
/// 12: gzipped data payload bytes
/// ```
pub fn embed_gzipped(
    cover: &impl GenericImage<Pixel = Rgb<u8>>,
    data: &[u8],
) -> RgbImage {
    let mut encoder = gzip::Encoder::new(Vec::new()).unwrap();
    io::copy(&mut Cursor::new(data), &mut encoder).unwrap();
    let data = encoder.finish().into_result().unwrap();
    embed_base(cover, STGZ_MAGIC, &data)
}

/// Extract steganographically embedded data from an image.
///
/// Works for data encoded with `embed` or `embed_gzipped`.
pub fn extract(
    cover: &impl GenericImage<Pixel = Rgb<u8>>,
) -> Result<Vec<u8>, ()> {
    let mut bytes = StegRead {
        pixels: cover.pixels(),
    };
    let magic_number = bytes.read_u32::<BigEndian>().map_err(|_| ())?;
    let expected_checksum = bytes.read_u32::<BigEndian>().map_err(|_| ())?;

    let is_gzipped = match magic_number {
        STGO_MAGIC => {
            info!("Extracting uncompressed data");
            false
        }
        STGZ_MAGIC => {
            info!("Extracting gzip compressed data");
            true
        }
        _ => return Err(()),
    };

    let payload_length =
        bytes.read_u32::<BigEndian>().map_err(|_| ())? as usize;
    info!("Data payload of {} bytes", payload_length);

    // Sanity check. If the header specs more data than the image can hold, assume it's corrupted
    // and bail out.
    if payload_length > encode_capacity(cover.width(), cover.height()) {
        warn!(
            "Payload size larger than image can hold, assuming corrupt header."
        );
        return Err(());
    }

    let mut payload = vec![0; payload_length];
    bytes.read(&mut payload).map_err(|_| ())?;

    let mut hasher = Hasher::new();
    hasher.update(&payload);
    let data_checksum = hasher.finalize();
    if data_checksum != expected_checksum {
        warn!("Crc32 integrity check failed, data is corrupt.");
        return Err(());
    }

    if is_gzipped {
        info!("Unzipping payload");
        let mut decoder = gzip::Decoder::new(&payload[..]).map_err(|_| ())?;
        let mut decoded_payload = Vec::new();
        decoder.read_to_end(&mut decoded_payload).map_err(|_| ())?;
        payload = decoded_payload;
    }

    Ok(payload)
}

#[cfg(test)]
mod test {
    use super::*;
    use image::{Rgb, RgbImage};

    #[test]
    fn test_simple() {
        let cover =
            RgbImage::from_pixel(8, 8, Rgb::from_channels(255, 255, 255, 255));
        let payload: Vec<u8> =
            "squeamish ossifrage".as_bytes().iter().cloned().collect();

        let stego = embed(&cover, &payload);
        let stegz = embed_gzipped(&cover, &payload);

        assert_eq!(extract(&stego), Ok(payload.clone()));
        assert_eq!(extract(&stegz), Ok(payload.clone()));
    }
}
