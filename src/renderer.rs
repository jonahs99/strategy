use wgpu::util::DeviceExt;
use slab::Slab;

use super::{model, texture};
use super::model::VertexDesc;

#[derive(Copy, Clone)]
pub struct InstanceHandle {
    model: u16,
    index: u16,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    pub view_proj: [[f32; 4]; 4],
}

impl Uniforms {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }
}

pub struct Renderer {
    models: Vec<ModelBuffers>,
    device: wgpu::Device,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    render_pipeline: wgpu::RenderPipeline,
    depth_texture: texture::Texture,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
}

impl Renderer {
    pub async fn new(window: &winit::window::Window) -> Self {
        let size = window.inner_size();

        let gpu_instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { gpu_instance.create_surface(window) };
        let adapter = gpu_instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            flags: wgpu::ShaderFlags::all(),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let uniforms = Uniforms::new();

        let uniform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            }
        ); 

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("uniform_bind_group_layout"),
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }
            ],
            label: Some("uniform_bind_group"),
        });

        let depth_texture = texture::Texture::create_depth_texture(&device, &sc_desc, "depth_texture");

        let instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: &[0u8; 4096],
                usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            }
        );

        let render_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &uniform_bind_group_layout,
                ],
                push_constant_ranges: &[],
            }
        );

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "main",
                buffers: &[model::MeshVertex::desc(), model::ModelInstance::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: sc_desc.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                clamp_depth: false,
                conservative: false,
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        let models = Vec::new();

        Self {
            models,
            device,
            surface,
            queue,
            sc_desc,
            swap_chain,
            size,
            render_pipeline,
            depth_texture,
            uniform_buffer,
            uniform_bind_group,
        }
    }

    pub fn add_model(&mut self, model: Model) -> u16 {
        let index = self.models.len() as u16;

        let num_triangles = (model.indices.len() / 3) as u16;
        let vertex_buffer = create_buffer_init(&mut self.device, "vertex", &model.vertices, wgpu::BufferUsage::VERTEX);
        let index_buffer = create_buffer_init(&mut self.device, "index", &model.indices, wgpu::BufferUsage::INDEX);
        let instance_buffer = create_buffer_init(&mut self.device, "instance", &[0u8; 2usize.pow(14)], wgpu::BufferUsage::VERTEX);
        let instances = DenseMap::new();

        self.models.push(ModelBuffers {
            vertex_buffer,
            index_buffer,
            num_triangles,
            instance_buffer,
            instance_buffer_dirty: false,
            instances,
        });

        index
    }

    pub fn add_instance(&mut self, model: u16, instance: Instance) -> InstanceHandle {
        assert!(model as usize <= self.models.len());
        let model_buffers = &mut self.models[model as usize];
        let index = model_buffers.instances.insert(instance) as u16;
        model_buffers.instance_buffer_dirty = true;
        InstanceHandle {
            model,
            index,
        }
    }

    pub fn remove_instance(&mut self, instance: InstanceHandle) {
        self.models[instance.model as usize].instances.remove(instance.index as usize);
    }
}

fn create_buffer_init(device: &mut wgpu::Device, label: &str, contents: &[impl bytemuck::Pod], usage: wgpu::BufferUsage) -> wgpu::Buffer {
    device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(contents),
            usage,
        }
    )
}

struct ModelBuffers {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_triangles: u16,

    instance_buffer: wgpu::Buffer,
    instance_buffer_dirty: bool,
    instances: DenseMap<Instance>,
}

pub struct Model {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

/// Element in the vertex buffer
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}

/// Element in the instance buffer
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
struct Instance {
    position: [f32; 3],
    color: [f32; 3],
}

struct DenseMap<T> {
    // The dense block of stored values
    values: Vec<T>,
    // Same size as values, each key indexes into indices
    keys: Vec<usize>,
    // Maps keys to indexes in values
    indices: Slab<usize>,
}

impl<T> DenseMap<T> {
    fn new() -> Self {
        Self {
            values: Vec::new(),
            keys: Vec::new(),
            indices: Slab::new(),
        }
    }

    fn values(&self) -> &[T] {
        return &self.values
    }

    fn insert(&mut self, value: T) -> usize {
        let index = self.values.len();
        let key = self.indices.insert(index);
        self.values.push(value);
        self.keys.push(key);
        key
    }

    fn remove(&mut self, key: usize) {
        let index = *self.indices.get(key).expect("The key does not exist");

        let swap_key = *self.keys.last().unwrap();
        self.indices[swap_key] = index;

        self.values.swap_remove(index);
        self.keys.swap_remove(index);
        self.indices.remove(key);
    }
}
