const { invoke } = window.__TAURI__.core;

async function recvF32Array() {
  const buf = await invoke("recv_f32");
  // buf is ArrayBuffer — Float32Array shares the same memory (zero-copy)
  return new Float32Array(buf);
}

async function recvU32Array() {
  const buf = await invoke("recv_u32");
  return new Uint32Array(buf);
}

export async function getTemplateParams() {
  return invoke("get_template_params");
}

export async function getMeshMetadata() {
  return invoke("get_mesh_metadata");
}

// Mesh binary commands — backend sends raw f32/u32 bytes via tauri::ipc::Response,
// frontend wraps in typed arrays (zero-copy on JS side).
export async function getMeshVertices() {
  const buf = await invoke("get_mesh_vertices");
  return new Float32Array(buf);
}

export async function getMeshFaces() {
  const buf = await invoke("get_mesh_faces");
  return new Uint32Array(buf);
}

export async function getMeshColors() {
  const buf = await invoke("get_mesh_colors");
  return new Float32Array(buf);
}

export async function getMeshVertexValues() {
  const buf = await invoke("get_mesh_vertex_values");
  return new Float32Array(buf);
}

// mesh_ct binary variants
export async function getMeshCtMetadata() { 
  return invoke("get_mesh_ct_metadata"); 
}

export async function getMeshCtVertices() {
  const buf = await invoke("get_mesh_ct_vertices");
  return new Float32Array(buf);
}

export async function getMeshCtFaces() {
  const buf = await invoke("get_mesh_ct_faces");
  return new Uint32Array(buf);
}

export async function getMeshCtColors() {
  const buf = await invoke("get_mesh_ct_colors");
  return new Float32Array(buf);
}

export async function getMeshCtVertexValues() {
  const buf = await invoke("get_mesh_ct_vertex_values");
  return new Float32Array(buf);
}

export async function getCtSlice(plane, index) { 
  return invoke("get_ct_slice", { plane, index }); 
}
