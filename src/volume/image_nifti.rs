use anyhow::{Result};
use nifti::{NiftiObject, ReaderOptions, NiftiVolume, IntoNdArray};
use nalgebra::Matrix4;

use super::{Volume};

pub fn load_nifti(path: &str) -> Result<Volume> {
    let data = ReaderOptions::new().read_file(path)?;
    let data_header = data.header().clone();
    let data_volume = data.volume();
    let dims = data_volume.dim();

    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;

    let affine: Matrix4<f64> = data_header.affine();

    let nifti_array = data_volume.into_ndarray::<f32>()?;

    let mut scalars = Vec::with_capacity(nx * ny * nz);
    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..nx {
                scalars.push(nifti_array[[ix, iy, iz]]);
            }
        }
    }

    let x: Vec<f64> = (0..nx).map(|i| -{
        affine[(0,0)] * i as f64 + affine[(0,3)]
    }).collect();
    let y: Vec<f64> = (0..ny).map(|i| -{
        affine[(1,1)] * i as f64 + affine[(1,3)]
    }).collect();
    let z: Vec<f64> = (0..nz).map(|i| {
        affine[(2,2)] * i as f64 + affine[(2,3)]
    }).collect();

    Volume::new(x, y, z, scalars)
}