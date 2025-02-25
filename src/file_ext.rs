use crate::align::*;
use crate::reerr::FileParseError;
use nalgebra_glm::*;
use std::convert::TryInto;
use std::error::Error;
use std::io::{Read, Seek};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[allow(dead_code)]
pub trait ReadExt {
    fn read_bool(&mut self) -> Result<bool>;
    fn read_u8_n(&mut self, n: usize) -> Result<Vec<u8>>;
    fn read_u8(&mut self) -> Result<u8>;
    fn read_u16(&mut self) -> Result<u16>;
    fn read_u32(&mut self) -> Result<u32>;
    fn read_u64(&mut self) -> Result<u64>;
    fn read_i8(&mut self) -> Result<i8>;
    fn read_i16(&mut self) -> Result<i16>;
    fn read_i32(&mut self) -> Result<i32>;
    fn read_i64(&mut self) -> Result<i64>;
    fn read_magic(&mut self) -> Result<[u8; 4]>;
    fn read_u16str(&mut self) -> Result<String>;
    fn read_utf16str(&mut self) -> Result<String>;
    fn read_u8str(&mut self) -> Result<String>;
    fn read_f32(&mut self) -> Result<f32>;
    fn read_f64(&mut self) -> Result<f64>;
    fn read_f32vec2(&mut self) -> Result<Vec2>;
    fn read_f32vec3(&mut self) -> Result<Vec3>;
    fn read_f32vec4(&mut self) -> Result<Vec4>;
    fn read_f32m4x4(&mut self) -> Result<Mat4x4>;
}

#[allow(dead_code)]
pub trait SeekExt {
    fn seek_noop(&mut self, from_start: u64) -> Result<u64>;
    fn seek_assert_align_up(&mut self, from_start: u64, align: u64) -> Result<u64>;
    fn seek_align_up(&mut self, align: u64) -> Result<u64>;
    fn tell(&mut self) -> Result<u64>;
}

impl<T: Read + ?Sized> ReadExt for T {
    fn read_bool(&mut self) -> Result<bool> {
        let v = self.read_u8()?;
        if v > 1 {
            return Err(Box::new(FileParseError::InvalidBool(v)))
        }
        Ok(v != 0)
    }
    fn read_u8_n(&mut self, n: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; n];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }
    fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }
    fn read_u16(&mut self) -> Result<u16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }
    fn read_u32(&mut self) -> Result<u32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }
    fn read_u64(&mut self) -> Result<u64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }
    fn read_i8(&mut self) -> Result<i8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0] as i8)
    }
    fn read_i16(&mut self) -> Result<i16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(i16::from_le_bytes(buf))
    }
    fn read_i32(&mut self) -> Result<i32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(i32::from_le_bytes(buf))
    }
    fn read_i64(&mut self) -> Result<i64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(i64::from_le_bytes(buf))
    }
    fn read_magic(&mut self) -> Result<[u8; 4]> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn read_utf16str(&mut self) -> Result<String> {
        let mut s = vec![];
        let n = self.read_u32()?;
        for _i in 0..n {
            let c = self.read_u16()?;
            s.push(c);
        }
        Ok(String::from_utf16(&s)?)
    }

    fn read_u16str(&mut self) -> Result<String> {
        let mut u16str = vec![];
        loop {
            let c = self.read_u16()?;
            if c == 0 {
                break;
            }
            u16str.push(c);
        }
        Ok(String::from_utf16(&u16str)?)
    }

    fn read_u8str(&mut self) -> Result<String> {
        let mut u8str = vec![];
        loop {
            let c = self.read_u8()?;
            if c == 0 {
                break;
            }
            u8str.push(c);
        }
        Ok(String::from_utf8(u8str)?)
    }
    fn read_f32(&mut self) -> Result<f32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(f32::from_le_bytes(buf))
    }
    fn read_f64(&mut self) -> Result<f64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(f64::from_le_bytes(buf))
    }
    fn read_f32vec4(&mut self) -> Result<Vec4> {
        Ok(vec4(
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
        ))
    }

    fn read_f32vec3(&mut self) -> Result<Vec3> {
        Ok(vec3(self.read_f32()?, self.read_f32()?, self.read_f32()?))
    }

    fn read_f32vec2(&mut self) -> Result<Vec2> {
        Ok(vec2(self.read_f32()?, self.read_f32()?))
    }

    fn read_f32m4x4(&mut self) -> Result<Mat4x4> {
        let data: Vec<f32> = std::iter::from_fn(|| Some(self.read_f32()))
            .take(16)
            .collect::<Result<_>>()?;
        Ok(make_mat4x4(&data))
    }
}

impl<T: Seek + Read + ?Sized> SeekExt for T {
    fn seek_noop(&mut self, from_start: u64) -> Result<u64> {
        let pos = self.stream_position()?;
        if pos != from_start {
            assert_eq!(
                pos,
                from_start,
                "This seek is expected to be no-op. At 0x{pos:08X}, seeking to 0x{from_start:08X}",
            );
        }
        Ok(pos)
    }

    fn seek_assert_align_up(&mut self, from_start: u64, align: u64) -> Result<u64> {
        let pos = self.stream_position()?;
        if align_up(pos, align) != from_start {
            assert_eq!(
                align_up(pos, align),
                from_start,
                "This seek is expected to only align up {align}. At 0x{pos:08X}, seeking to 0x{from_start:08X}",
            );
        }
        if pos != from_start {
            let mut buf = vec![0; (from_start - pos).try_into()?];
            self.read_exact(&mut buf)?;
            if buf.into_iter().any(|x| x != 0) {
                return Err(Box::new(FileParseError::BadAlign(pos, align)))
            }
        }

        Ok(from_start)
    }

    fn seek_align_up(&mut self, align: u64) -> Result<u64> {
        let pos = self.stream_position()?;
        let aligned = align_up(pos, align);
        if aligned != pos {
            let mut buf = vec![0; (aligned - pos).try_into()?];
            self.read_exact(&mut buf)?;
            if buf.into_iter().any(|x| x != 0) {
                return Err(Box::new(FileParseError::BadAlign(pos, align)))
            }
        }

        Ok(aligned)
    }

    fn tell(&mut self) -> Result<u64> {
        Ok(self.stream_position()?)
    }
}
