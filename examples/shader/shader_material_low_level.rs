//! A shader and a material that uses it usingBevy's low level API.
//! See shader_material example for a higher level implementation.

use bevy::{
    core_pipeline::core_3d::Transparent3d,
    ecs::system::{lifetimeless::*, SystemParamItem},
    pbr::{
        DrawMesh, MeshPipeline, MeshPipelineKey, MeshUniform, SetMeshBindGroup,
        SetMeshViewBindGroup,
    },
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_component::ExtractComponentPlugin,
        mesh::MeshVertexBufferLayout,
        render_asset::{PrepareAssetError, RenderAsset, RenderAssetPlugin, RenderAssets},
        render_phase::{
            AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline, TrackedRenderPass,
        },
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        texture::{DefaultImageSampler, FallbackImage},
        view::{ExtractedView, Msaa},
        RenderApp, RenderStage,
    },
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(CustomMaterialPlugin)
        .add_startup_system(setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // cube
    commands.spawn().insert_bundle((
        meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        Transform::from_xyz(0.0, 0.5, 0.0),
        GlobalTransform::default(),
        Visibility::default(),
        ComputedVisibility::default(),
        materials.add(CustomMaterial {
            color: Color::GREEN,
            color_texture: Some(asset_server.load("branding/icon.png")),
            alpha_mode: AlphaMode::Blend,
        }),
    ));

    // camera
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct CustomMaterial {
    color: Color,
    color_texture: Option<Handle<Image>>,
    alpha_mode: AlphaMode,
}

/// The GPU-representation of an [`Image`].
/// Consists of the [`Texture`], its [`TextureView`] and the corresponding [`Sampler`], and the texture's size.
#[derive(Debug, Clone)]
pub struct GpuCustomMaterial {
    pub texture: Texture,
    pub texture_view: TextureView,
    pub texture_format: TextureFormat,
    pub sampler: Sampler,
    pub size: Vec2,
}

impl RenderAsset for CustomMaterial {
    type ExtractedAsset = CustomMaterial;
    type PreparedAsset = GpuCustomMaterial;
    type Param = (
        SRes<RenderDevice>,
        SRes<RenderQueue>,
        SRes<DefaultImageSampler>,
    );

    /// Clones the Image.
    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    /// Converts the extracted image into a [`GpuImage`].
    fn prepare_asset(
        image: Self::ExtractedAsset,
        (render_device, render_queue, default_sampler): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let texture = if image.texture_descriptor.mip_level_count > 1 || image.is_compressed() {
            render_device.create_texture_with_data(
                render_queue,
                &image.texture_descriptor,
                &image.data,
            )
        } else {
            let texture = render_device.create_texture(&image.texture_descriptor);
            let format_size = image.texture_descriptor.format.pixel_size();
            render_queue.write_texture(
                ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &image.data,
                ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(
                        std::num::NonZeroU32::new(
                            image.texture_descriptor.size.width * format_size as u32,
                        )
                        .unwrap(),
                    ),
                    rows_per_image: if image.texture_descriptor.size.depth_or_array_layers > 1 {
                        std::num::NonZeroU32::new(image.texture_descriptor.size.height)
                    } else {
                        None
                    },
                },
                image.texture_descriptor.size,
            );
            texture
        };

        let texture_view = texture.create_view(
            image
                .texture_view_descriptor
                .or_else(|| Some(TextureViewDescriptor::default()))
                .as_ref()
                .unwrap(),
        );
        let size = Vec2::new(
            image.texture_descriptor.size.width as f32,
            image.texture_descriptor.size.height as f32,
        );
        let sampler = match image.sampler_descriptor {
            ImageSampler::Default => (***default_sampler).clone(),
            ImageSampler::Descriptor(descriptor) => render_device.create_sampler(&descriptor),
        };

        Ok(GpuImage {
            texture,
            texture_view,
            texture_format: image.texture_descriptor.format,
            sampler,
            size,
        })
    }
}

pub struct CustomMaterialPlugin;

impl Plugin for CustomMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<CustomMaterial>()
            .add_plugin(ExtractComponentPlugin::<Handle<CustomMaterial>>::default())
            .add_plugin(RenderAssetPlugin::<CustomMaterial>::default());
        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCustom>()
            .init_resource::<CustomPipeline>()
            .init_resource::<SpecializedMeshPipelines<CustomPipeline>>()
            .add_system_to_stage(RenderStage::Queue, queue_custom);
    }
}

#[derive(Resource)]
pub struct CustomPipeline {
    mesh_pipeline: MeshPipeline,
    material_layout: BindGroupLayout,
    shader: Handle<Shader>,
}

impl SpecializedMeshPipeline for CustomPipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();
        descriptor.layout = Some(vec![
            self.mesh_pipeline.view_layout.clone(),
            self.material_layout.clone(),
            self.mesh_pipeline.mesh_layout.clone(),
        ]);
        Ok(descriptor)
    }
}

impl FromWorld for CustomPipeline {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        // Watch for changes, allowing for hot shader reloading
        // Try changing custom_material.wgsl while the app is running!
        asset_server.watch_for_changes().unwrap();

        let render_device = world.get_resource::<RenderDevice>().unwrap();
        let material_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 1u32,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2u32,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 0u32,
                    visibility: ShaderStages::all(),
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(<Color as ShaderType>::min_size()),
                    },
                    count: None,
                },
            ],
            label: None,
        });

        CustomPipeline {
            mesh_pipeline: world.get_resource::<MeshPipeline>().unwrap().clone(),
            shader: asset_server.load("shaders/custom_material.wgsl"),
            material_layout,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn queue_custom(
    transparent_3d_draw_functions: Res<DrawFunctions<Transparent3d>>,
    materials: Res<RenderAssets<CustomMaterial>>,
    render_meshes: Res<RenderAssets<Mesh>>,
    custom_pipeline: Res<CustomPipeline>,
    mut pipeline_cache: ResMut<PipelineCache>,
    mut specialized_pipelines: ResMut<SpecializedMeshPipelines<CustomPipeline>>,
    msaa: Res<Msaa>,
    material_meshes: Query<(Entity, &Handle<CustomMaterial>, &Handle<Mesh>, &MeshUniform)>,
    mut views: Query<(&ExtractedView, &mut RenderPhase<Transparent3d>)>,
) {
    let draw_custom = transparent_3d_draw_functions
        .read()
        .get_id::<DrawCustom>()
        .unwrap();
    let key = MeshPipelineKey::from_msaa_samples(msaa.samples);
    for (view, mut transparent_phase) in views.iter_mut() {
        let view_matrix = view.transform.compute_matrix();
        let view_row_2 = view_matrix.row(2);
        for (entity, material_handle, mesh_handle, mesh_uniform) in material_meshes.iter() {
            if materials.contains_key(material_handle) {
                if let Some(mesh) = render_meshes.get(mesh_handle) {
                    let key =
                        key | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology);
                    transparent_phase.add(Transparent3d {
                        entity,
                        pipeline: specialized_pipelines
                            .specialize(&mut pipeline_cache, &custom_pipeline, key, &mesh.layout)
                            .unwrap(),
                        draw_function: draw_custom,
                        distance: view_row_2.dot(mesh_uniform.transform.col(3)),
                    });
                }
            }
        }
    }
}

type DrawCustom = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetCustomMaterialBindGroup,
    SetMeshBindGroup<2>,
    DrawMesh,
);

struct SetCustomMaterialBindGroup;
impl EntityRenderCommand for SetCustomMaterialBindGroup {
    type Param = (
        SRes<RenderAssets<CustomMaterial>>,
        SQuery<Read<Handle<CustomMaterial>>>,
    );
    fn render<'w>(
        _view: Entity,
        item: Entity,
        (materials, query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let material_handle = query.get(item).unwrap();
        let material = materials.into_inner().get(material_handle).unwrap();
        pass.set_bind_group(1, &material.bind_group, &[]);
        RenderCommandResult::Success
    }
}
