use serde::Deserialize;
use serde_json::{Value, json};
use chrono::Utc;
use shared_lib::storage_utils::{save_to_local, save_to_s3, store_memory};

use std::ops::{Add, Mul, Sub, Index, IndexMut};

use rayon::prelude::*;


const VLEN: usize = 8;

#[derive(Deserialize)]
struct Json {
    image_size: usize,
    storage_type: String,
    output_path: String,
    #[serde(default)]
    bucket: Option<String>,
}

#[derive(Clone, Copy)]
#[repr(align(32))]
struct F64x8([f64; 8]);

impl F64x8 {
    #[inline(always)]
    const fn splat(val: f64) -> Self {
        Self([val; 8])
    }
}

impl From<[f64; 8]> for F64x8 {
    #[inline(always)]
    fn from(value: [f64; 8]) -> F64x8 {
        F64x8(value)
    }
}

impl Index<usize> for F64x8 {
    type Output = f64;
    #[inline(always)]
    fn index(&self, index: usize) -> &f64 {
        &self.0[index]
    }
}

impl IndexMut<usize> for F64x8 {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut f64 {
        &mut self.0[index]
    }
}

macro_rules! impl_binary {
    ($trait:ident, $func:ident, $op:tt) => {
        impl $trait<F64x8> for F64x8 {
            type Output = F64x8;
            #[inline]
            fn $func(self, rhs: F64x8) -> F64x8 {
                F64x8([
                    self.0[0] $op rhs.0[0],
                    self.0[1] $op rhs.0[1],
                    self.0[2] $op rhs.0[2],
                    self.0[3] $op rhs.0[3],
                    self.0[4] $op rhs.0[4],
                    self.0[5] $op rhs.0[5],
                    self.0[6] $op rhs.0[6],
                    self.0[7] $op rhs.0[7]
                ])
            }
        }

        impl $trait<f64> for F64x8 {
            type Output = F64x8;
            #[inline]
            fn $func(self, rhs: f64) -> F64x8 {
                F64x8([
                    self.0[0] $op rhs,
                    self.0[1] $op rhs,
                    self.0[2] $op rhs,
                    self.0[3] $op rhs,
                    self.0[4] $op rhs,
                    self.0[5] $op rhs,
                    self.0[6] $op rhs,
                    self.0[7] $op rhs
                ])
            }
        }
    }
}

impl_binary!(Add, add, +);
impl_binary!(Sub, sub, -);
impl_binary!(Mul, mul, *);

#[inline(always)]
fn check_divergence(z: &F64x8) -> bool {
    ((z[0] > 4.) & (z[1] > 4.) & (z[2] > 4.) & (z[3] > 4.))
        && ((z[4] > 4.) & (z[5] > 4.) & (z[6] > 4.) & (z[7] > 4.))
}

#[inline(always)]
fn convert_to_bit(z: &F64x8) -> u8 {
    (((z[0] <= 4.0) as u8) << 0) |
    (((z[1] <= 4.0) as u8) << 1) |
    (((z[2] <= 4.0) as u8) << 2) |
    (((z[3] <= 4.0) as u8) << 3) |
    (((z[4] <= 4.0) as u8) << 4) |
    (((z[5] <= 4.0) as u8) << 5) |
    (((z[6] <= 4.0) as u8) << 6) |
    (((z[7] <= 4.0) as u8) << 7)
}

#[inline(never)]
fn mand8(cr: F64x8, ci: f64, last_char: u8) -> u8 {
    let mut zr = cr;
    let mut zi = F64x8::splat(ci);
    let mut tr = zr * zr;
    let mut ti = zi * zi;

    macro_rules! proceed {
        ($n:expr) => {
            for _ in 0..$n {
                zi = (zr + zr) * zi + ci;
                zr = tr - ti + cr;
                tr = zr * zr;
                ti = zi * zi;
            }
        };
    }

    for _ in 0..12 {
        proceed!(4);

        let absz = tr + ti;
        if last_char == 0 && check_divergence(&absz) {
            return 0;
        }
    }

    proceed!(1);

    let absz = tr + ti;
    convert_to_bit(&absz)
}

fn create_mandelbrot(input_size: usize) -> Vec<u8> {
    let size = input_size;
    let size = size / VLEN * VLEN;

    let inv = 2. / size as f64;

    let xloc = (0..size / VLEN).map(|i| {
        F64x8([
            (i * VLEN + 7) as f64,
            (i * VLEN + 6) as f64,
            (i * VLEN + 5) as f64,
            (i * VLEN + 4) as f64,
            (i * VLEN + 3) as f64,
            (i * VLEN + 2) as f64,
            (i * VLEN + 1) as f64,
            (i * VLEN + 0) as f64,
        ]) * inv - 1.5
    }).collect::<Vec<_>>();

    let mut rows = vec![0; size * size / VLEN];
    rows.par_chunks_mut(size / VLEN).enumerate().for_each(|(y, out)| {
        assert_eq!(xloc.len(), size / VLEN);
        assert_eq!(out.len(), size / VLEN);

        let ci = y as f64 * inv - 1.;
        let mut last_char = 0;

        for x in 0..size / VLEN {
            last_char = mand8(xloc[x], ci, last_char);
            out[x] = last_char;
        }
    });
   
    // Create PBM header
    let header = format!("P4\n{} {}\n", size, size);

    // Combine header and data
    let mut combined = Vec::new();
    combined.extend_from_slice(header.as_bytes());
    combined.extend_from_slice(&rows);

    combined
}


pub async fn handle_json(json: Value) -> Result<String, String> {

    let result: Result<Json, _> = serde_json::from_value(json);

    match result {
        Ok(valid_json) => {
            let bitmap_data = create_mandelbrot(valid_json.image_size);
            let mut retrieval_ms = 0.0;
            // Try to save the bitmap based on storage type
            let save_result = match valid_json.storage_type.as_str() {
                "local" => save_to_local(&valid_json.output_path, &bitmap_data).await?,
                "s3" => save_to_s3(&valid_json.bucket.unwrap(), &valid_json.output_path, &bitmap_data).await?,
                "memory" => {
                    let start_time = Utc::now();
                    println!("Start retrieval at {}", start_time);
                    let result = store_memory(&valid_json.output_path, bitmap_data).await?;
                    let mid_time = Utc::now();
                    println!("Retrieval finished at {}", mid_time);
                    let retrieval_ns = (mid_time - start_time).num_nanoseconds().unwrap_or(0);
                    retrieval_ms = retrieval_ns as f64 / 1_000_000.0;
                    result
                },
                _ => return Err("Unsupported storage type".to_string()),
            };

            // Construct a success response
            let response = json!({
                "status": "success",
                "runtime": "native",
                "data": {
                    "data_retrieval": retrieval_ms,
                    "serialization": 0,
                    "save_path": save_result
                }
            });

            Ok(response.to_string())
        },
        Err(e) => {
            let response = json!({
                "status": "error",
                "message": e.to_string()
            });
            Ok(response.to_string())
        }
    }
}