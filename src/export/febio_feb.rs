use anyhow::Result;

use crate::mesh::Mesh;
use crate::params::Params;

pub fn write_feb(mesh: &Mesh, per_part_moduli: &[Vec<f64>], params: &Params, orig_mesh_path: &str) -> Result<String> {
	crate::mesh::febio_feb::write_feb(mesh, per_part_moduli, params, orig_mesh_path)
}
