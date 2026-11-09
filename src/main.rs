mod engine;
mod game;
mod shaders;

use std::time::Instant;

use glam::{Mat4, Vec3};
use minifb::{Key, KeyRepeat, Window, WindowOptions};

use engine::framebuffer::Framebuffer;
use engine::mesh::{self, Mesh};
use engine::rasterizer;
use game::config;
use shaders::gas_giant::GasGiantShader;
use shaders::moon::MoonShader;
use shaders::rings::RingShader;
use shaders::rocky::RockyShader;
use shaders::star::StarShader;
use shaders::{FragmentShader, Uniforms};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut window = Window::new(
        "Cuerpos Celestes - Shaders (1: estrella, 2: rocoso, 3: gigante gaseoso)",
        config::WIDTH,
        config::HEIGHT,
        WindowOptions::default(),
    )?;
    window.limit_update_rate(Some(std::time::Duration::from_micros(16_600)));

    let mut fb = Framebuffer::new(config::WIDTH, config::HEIGHT);

    // Cámara fija para todas las escenas, sin rotación ni traslación.
    let aspect = config::WIDTH as f32 / config::HEIGHT as f32;
    let proj = Mat4::perspective_rh(config::FOV_DEGREES.to_radians(), aspect, config::NEAR_PLANE, config::FAR_PLANE);
    let eye = Vec3::new(0.0, 2.0, 8.0);
    let view = Mat4::look_at_rh(eye, Vec3::ZERO, Vec3::Y);
    let view_proj = proj * view;
    let light_dir = Vec3::new(0.6, 0.5, 0.6).normalize();

    // Geometría compartida: una sola esfera reutilizada con distintas matrices
    // de modelo para la estrella, el rocoso, el gaseoso y la luna.
    let unit_sphere = mesh::sphere(config::SPHERE_STACKS, config::SPHERE_SLICES);
    let ring_mesh = mesh::ring(config::RING_INNER_RADIUS, config::RING_OUTER_RADIUS, config::RING_SEGMENTS);

    let star_shader = StarShader::new(1);
    let rocky_shader = RockyShader::new(2);
    let moon_shader = MoonShader::new(3);
    let gas_shader = GasGiantShader::new(4);

    let gas_giant_center = Vec3::ZERO;
    let gas_giant_radius = config::GAS_GIANT_RADIUS;
    let ring_shader = RingShader::new(
        5,
        gas_giant_center,
        gas_giant_radius,
        config::RING_INNER_RADIUS,
        config::RING_OUTER_RADIUS,
    );

    // Escena 1: solo la estrella.
    let star_model = Mat4::from_scale(Vec3::splat(1.6));
    let scene_star: Vec<(&Mesh, Mat4, &dyn FragmentShader)> = vec![(&unit_sphere, star_model, &star_shader)];

    // Escena 2: planeta rocoso con su luna (geometría aparte, offset fijo).
    let rocky_model = Mat4::from_translation(Vec3::new(-0.5, 0.0, 0.0)) * Mat4::from_scale(Vec3::splat(1.1));
    let moon_model =
        Mat4::from_translation(Vec3::new(1.7, 0.55, 0.5)) * Mat4::from_scale(Vec3::splat(0.3));
    let scene_rocky: Vec<(&Mesh, Mat4, &dyn FragmentShader)> = vec![
        (&unit_sphere, rocky_model, &rocky_shader),
        (&unit_sphere, moon_model, &moon_shader),
    ];

    // Escena 3: gigante gaseoso con sus anillos (geometría aparte, se dibuja después para mezclar con alfa).
    let gas_model = Mat4::from_scale(Vec3::splat(gas_giant_radius));
    let scene_gas: Vec<(&Mesh, Mat4, &dyn FragmentShader)> = vec![
        (&unit_sphere, gas_model, &gas_shader),
        (&ring_mesh, Mat4::IDENTITY, &ring_shader),
    ];

    let mut active: u8 = 1;
    let start = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.is_key_pressed(Key::Key1, KeyRepeat::No) {
            active = 1;
        }
        if window.is_key_pressed(Key::Key2, KeyRepeat::No) {
            active = 2;
        }
        if window.is_key_pressed(Key::Key3, KeyRepeat::No) {
            active = 3;
        }

        let uniforms = Uniforms { time: start.elapsed().as_secs_f32(), light_dir };
        fb.clear(config::BACKGROUND_COLOR);

        let objects = match active {
            1 => &scene_star,
            2 => &scene_rocky,
            _ => &scene_gas,
        };
        for (mesh, model, shader) in objects {
            rasterizer::draw_mesh(&mut fb, mesh, *model, view_proj, eye, *shader, &uniforms);
        }

        window.update_with_buffer(&fb.pixels, config::WIDTH, config::HEIGHT)?;
    }

    Ok(())
}
