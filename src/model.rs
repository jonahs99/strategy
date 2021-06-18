use wgpu::{
    vertex_attr_array,
    util::DeviceExt,
};

pub trait VertexDesc {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshVertex {
    position: [f32; 3],
    normal: [f32; 3],
}

const MESH_VERTEX_ATTRS: [wgpu::VertexAttribute; 2] = vertex_attr_array![
    0 => Float32x3,
    1 => Float32x3,
];

impl VertexDesc for MeshVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<MeshVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &MESH_VERTEX_ATTRS,
        }
    }
}

pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
}

impl Mesh {
    pub fn load(
        device: &wgpu::Device,
    ) -> Self {
        const VERTICES: &[MeshVertex] = &[
            // top (0, 0, 1)
            MeshVertex { position: [-1., -1., 1.], normal: [0., 0., 1.] },
            MeshVertex { position: [1., -1., 1.], normal: [0., 0., 1.] },
            MeshVertex { position: [1., 1., 1.], normal: [0., 0., 1.] },
            MeshVertex { position: [-1., 1., 1.], normal: [0., 0., 1.] },
            // bottom (0, 0, -1.)
            MeshVertex { position: [-1., 1., -1.], normal: [0., 0., -1.] },
            MeshVertex { position: [1., 1., -1.], normal: [0., 0., -1.] },
            MeshVertex { position: [1., -1., -1.], normal: [0., 0., -1.] },
            MeshVertex { position: [-1., -1., -1.], normal: [0., 0., -1.] },
            // right (1., 0, 0)
            MeshVertex { position: [1., -1., -1.], normal: [1., 0., 0.] },
            MeshVertex { position: [1., 1., -1.], normal: [1., 0., 0.] },
            MeshVertex { position: [1., 1., 1.], normal: [1., 0., 0.] },
            MeshVertex { position: [1., -1., 1.], normal: [1., 0., 0.] },
            // left (-1., 0, 0)
            MeshVertex { position: [-1., -1., 1.], normal: [-1., 0., 0.] },
            MeshVertex { position: [-1., 1., 1.], normal: [-1., 0., 0.] },
            MeshVertex { position: [-1., 1., -1.], normal: [-1., 0., 0.] },
            MeshVertex { position: [-1., -1., -1.], normal: [-1., 0., 0.] },
            // front (0, 1., 0)
            MeshVertex { position: [1., 1., -1.], normal: [0., 1., 0.] },
            MeshVertex { position: [-1., 1., -1.], normal: [0., 1., 0.] },
            MeshVertex { position: [-1., 1., 1.], normal: [0., 1., 0.] },
            MeshVertex { position: [1., 1., 1.], normal: [0., 1., 0.] },
            // back (0, -1., 0)
            MeshVertex { position: [1., -1., 1.], normal: [0., -1., 0.] },
            MeshVertex { position: [-1., -1., 1.], normal: [0., -1., 0.] },
            MeshVertex { position: [-1., -1., -1.], normal: [0., -1., 0.] },
            MeshVertex { position: [1., -1., -1.], normal: [0., -1., 0.] },
        ];

        const INDICES: &[u16] = &[
            0, 1, 2, 2, 3, 0, // top
            4, 5, 6, 6, 7, 4, // bottom
            8, 9, 10, 10, 11, 8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // front
            20, 21, 22, 22, 23, 20, // back
        ];

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsage::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsage::INDEX,
            }
        );

        Self {
            vertex_buffer,
            index_buffer,
            num_elements: INDICES.len() as u32,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelInstance {
    pub model: [[f32; 4]; 4],
    pub normal: [[f32; 3]; 3],
    pub color: [f32; 3],
}

const INSTANCE_RAW_ATTRS: [wgpu::VertexAttribute; 8] = wgpu::vertex_attr_array![
    5 => Float32x4,
    6 => Float32x4,
    7 => Float32x4,
    8 => Float32x4,
    9 => Float32x3,
    10 => Float32x3,
    11 => Float32x3,
    12 => Float32x3,
];

impl VertexDesc for ModelInstance {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &INSTANCE_RAW_ATTRS,
        }
    }
}
