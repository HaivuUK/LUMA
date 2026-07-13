use anyhow::{Result, anyhow};

pub mod image_vtk;
pub mod image_dicom;
pub mod image_nifti;
pub mod image_nrrd;

#[derive(Debug, Clone)]
pub struct Volume {
	pub x: Vec<f64>,
	pub y: Vec<f64>,
	pub z: Vec<f64>,
	pub scalars: Vec<f32>,
	scalar_range: (f32, f32),
	pub voxel_spacing: [f64; 3], // dx, dy, dz - voxel dimensions
	nx: usize,
	ny: usize,
	stride_xy: usize,
}

impl Volume {
	pub fn new(x: Vec<f64>, y: Vec<f64>, z: Vec<f64>, scalars: Vec<f32>) -> Result<Self> {
		let nx = x.len(); let ny = y.len(); let nz = z.len();
		if nx == 0 || ny == 0 || nz == 0 { return Err(anyhow!("Empty axis in volume")); }
		let (scalar_min, scalar_max) = scalars.iter()
			.filter(|&&v| v.is_finite())
			.fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), &val| {
				(min.min(val), max.max(val))
			});

		let scalar_range = if scalar_min.is_infinite() || scalar_max.is_infinite() {
			(0.0, 1.0)
		} else {
			(scalar_min, scalar_max)
		};
		// Calculate voxel spacing from coordinate arrays
		let voxel_spacing = [
			if nx > 1 { (x[nx-1] - x[0]) / (nx-1) as f64 } else { 1.0 },
			if ny > 1 { (y[ny-1] - y[0]) / (ny-1) as f64 } else { 1.0 },
			if nz > 1 { (z[nz-1] - z[0]) / (nz-1) as f64 } else { 1.0 },
		];
		Ok(Self { stride_xy: nx*ny, nx, ny, x, y, z, scalars, scalar_range, voxel_spacing })
	}

	#[inline]
	pub fn trilinear(&self, px: f64, py: f64, pz: f64) -> f64 {
		// Proper boundary handling for trilinear interpolation
		let xi1 = self.x.partition_point(|&v| v < px).clamp(1, self.x.len().saturating_sub(1));
		let yi1 = self.y.partition_point(|&v| v < py).clamp(1, self.y.len().saturating_sub(1));
		let zi1 = self.z.partition_point(|&v| v < pz).clamp(1, self.z.len().saturating_sub(1));
		
		let xi0 = xi1 - 1;
		let yi0 = yi1 - 1;
		let zi0 = zi1 - 1;
		
		// Load coordinates once
		let x0 = self.x[xi0]; let x1 = self.x[xi1];
		let y0 = self.y[yi0]; let y1 = self.y[yi1];
		let z0 = self.z[zi0]; let z1 = self.z[zi1];
		
		// Compute ratios with branch avoidance
		let rx = if x1 != x0 { (px - x0) / (x1 - x0) } else { 0.0 };
		let ry = if y1 != y0 { (py - y0) / (y1 - y0) } else { 0.0 };
		let rz = if z1 != z0 { (pz - z0) / (z1 - z0) } else { 0.0 };
		
		// Compute indices once
		let i000 = xi0 + yi0 * self.nx + zi0 * self.stride_xy;
		let i001 = xi0 + yi0 * self.nx + zi1 * self.stride_xy;
		let i010 = xi0 + yi1 * self.nx + zi0 * self.stride_xy;
		let i011 = xi0 + yi1 * self.nx + zi1 * self.stride_xy;
		let i100 = xi1 + yi0 * self.nx + zi0 * self.stride_xy;
		let i101 = xi1 + yi0 * self.nx + zi1 * self.stride_xy;
		let i110 = xi1 + yi1 * self.nx + zi0 * self.stride_xy;
		let i111 = xi1 + yi1 * self.nx + zi1 * self.stride_xy;
		
		// Load all 8 scalar values at once for better cache usage
		let s000 = self.scalars[i000] as f64; let s001 = self.scalars[i001] as f64;
		let s010 = self.scalars[i010] as f64; let s011 = self.scalars[i011] as f64;
		let s100 = self.scalars[i100] as f64; let s101 = self.scalars[i101] as f64;
		let s110 = self.scalars[i110] as f64; let s111 = self.scalars[i111] as f64;
		
		// Optimized trilinear interpolation
		let inv_rx = 1.0 - rx;
		let inv_ry = 1.0 - ry;
		let inv_rz = 1.0 - rz;
		
		let c00 = s000 * inv_rx + s100 * rx; 
		let c01 = s001 * inv_rx + s101 * rx;
		let c10 = s010 * inv_rx + s110 * rx; 
		let c11 = s011 * inv_rx + s111 * rx;
		
		let c0 = c00 * inv_ry + c10 * ry; 
		let c1 = c01 * inv_ry + c11 * ry;
		
		c0 * inv_rz + c1 * rz
	}

	pub fn value_range(&self) -> (f32, f32) {
		self.scalar_range
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PixelType {
	Int8,
	Uint8,
	Int16,
	Uint16,
	Int32,
	Uint32,
	Float32,
	Float64,
	/// 1-bit packed pixels (e.g. bilevel TIFF).
	Bit,
}

impl PixelType {
	/// Size in bytes of one decoded sample.
	/// `Bit` planes are exposed by readers as one byte per unpacked sample, not
	/// as packed on-disk bits.
	pub fn bytes_per_sample(self) -> usize {
		match self {
			PixelType::Int8 | PixelType::Uint8 => 1,
			PixelType::Int16 | PixelType::Uint16 => 2,
			PixelType::Int32 | PixelType::Uint32 | PixelType::Float32 => 4,
			PixelType::Float64 => 8,
			PixelType::Bit => 1,
		}
	}
}

pub fn load_volume(path: &str) -> Result<Volume> {
	if path.ends_with(".vtk") {
		image_vtk::load_vtk(path)
	} else if path.ends_with(".nii") || path.ends_with(".nii.gz") {
		image_nifti::load_nifti(path)
	} else if path.ends_with(".nrrd") || path.ends_with(".nhdr") {
		image_nrrd::load_nrrd(path)
	} else {
		image_dicom::load_dicom_dir(path)
	}
}

#[cfg(test)]
mod tests {
	use std::path::{Path};
	use super::*;

	#[test]
	fn discovers_multiple_series_in_post_mortem_fixture() {
		let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/abaqus_scanip_test/CT_Post_Mortem_Series");
		let series = image_dicom::discover_dicom_series(&root).expect("discover DICOM series");
		assert!(series.len() > 1, "expected multiple series, got {}", series.len());
	}

	#[test]
	fn loads_single_series_reformat_stack() {
		let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/abaqus_scanip_test/CT_reformat");
		let volume = image_dicom::load_dicom_dir(root.to_str().expect("UTF-8 path")).expect("load single-series DICOM stack");
		assert!(!volume.x.is_empty(), "X axis should not be empty");
		assert!(!volume.y.is_empty(), "Y axis should not be empty");
		assert!(!volume.z.is_empty(), "Z axis should not be empty");
		assert_eq!(volume.scalars.len(), volume.x.len() * volume.y.len() * volume.z.len());
	}
}

	#[test]
	fn bit_pixels_have_decoded_sample_width() {
		assert_eq!(PixelType::Bit.bytes_per_sample(), 1);
	}
