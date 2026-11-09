mod engine;
mod game;
mod shaders;

use std::time::Instant;

use minifb::{Window, WindowOptions};

use engine::culling::Frustum;
use engine::framebuffer::Framebuffer;
use engine::lod::{self, LodLevel};
use engine::math::{Mat4, Quat, Vec3};
use engine::mesh::{self, Mesh};
use engine::obj;
use engine::rasterizer::RenderQueue;
use game::bodies::{self, ShaderKind};
use game::camera::ChaseCamera;
use game::config;
use game::hud;
use game::input;
use game::ship::Ship;
use game::skybox::Skybox;
use game::world::World;
use shaders::gas_giant::GasGiantShader;
use shaders::moon::MoonShader;
use shaders::rocky::RockyShader;
use shaders::ship::ShipShader;
use shaders::star::StarShader;
use shaders::{FragmentShader, Uniforms};

/// Shader de superficie según el tipo de dato de `bodies.rs`; construido una
/// sola vez al inicio (las tablas de ruido no se recrean por frame).
fn build_planet_shader(def: &bodies::BodyDef) -> Box<dyn FragmentShader> {
    match def.shader {
        ShaderKind::Rocky => Box::new(RockyShader::new(def.seed)),
        ShaderKind::GasGiant => Box::new(GasGiantShader::new(def.seed)),
        ShaderKind::Moon => Box::new(MoonShader::new(def.seed)),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut window = Window::new(
        "Fantasticar: Defensa de la Tierra",
        config::WIDTH,
        config::HEIGHT,
        WindowOptions::default(),
    )?;
    window.limit_update_rate(Some(std::time::Duration::from_micros(16_600)));

    let mut fb = Framebuffer::new(config::WIDTH, config::HEIGHT);

    let aspect = config::WIDTH as f32 / config::HEIGHT as f32;
    let proj = Mat4::perspective_rh(config::FOV_DEGREES.to_radians(), aspect, config::NEAR_PLANE, config::FAR_PLANE);
    let light_dir = Vec3::new(0.6, 0.5, 0.6).normalize();

    let ship_mesh: Mesh = obj::load_obj(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/models/fantasticar.obj"))
        .expect("no se pudo cargar assets/models/fantasticar.obj");
    let ship_shader = ShipShader::new();

    // Tres mallas esféricas (alta/media/baja), reutilizadas para el sol, cada
    // planeta y la luna; cuál se usa depende de la distancia a la cámara
    // (LOD, Fase 4). El tamaño real lo da la matriz de modelo.
    let sphere_high = mesh::sphere(config::SPHERE_STACKS_HIGH, config::SPHERE_SLICES_HIGH);
    let sphere_medium = mesh::sphere(config::SPHERE_STACKS_MEDIUM, config::SPHERE_SLICES_MEDIUM);
    let sphere_low = mesh::sphere(config::SPHERE_STACKS_LOW, config::SPHERE_SLICES_LOW);
    let lod_mesh = |level: LodLevel| -> &Mesh {
        match level {
            LodLevel::High => &sphere_high,
            LodLevel::Medium => &sphere_medium,
            LodLevel::Low => &sphere_low,
        }
    };
    let sun_shader = StarShader::new(bodies::SUN_SEED);
    let moon_shader = MoonShader::new(bodies::MOON.seed);
    let planet_shaders: Vec<Box<dyn FragmentShader>> = bodies::PLANETS.iter().map(build_planet_shader).collect();

    let skybox = Skybox::new(config::STARFIELD_SEED, config::STARFIELD_COUNT);

    let mut world = World::new();
    let spawn_point = |world: &World| {
        let earth = &world.planets[bodies::EARTH_INDEX];
        earth.position + Vec3::new(0.0, 3.0, earth.def.radius + 14.0)
    };
    let mut ship = Ship::new(spawn_point(&world));
    let mut camera = ChaseCamera::new(&ship);

    let mut last_frame = Instant::now();
    let mut elapsed = 0.0f32;
    let mut title_timer = 0.0f32;
    let mut paused = false;
    let mut show_manual = true;
    // Depuración de rendimiento (Fase 4): comparar con F2/F3 el efecto real
    // del descarte por frustum y del LOD.
    let mut frustum_culling_enabled = true;
    let mut force_max_lod = false;
    let mut fps_smoothed = 60.0f32;

    // Cola de triángulos del frame, reutilizada entre iteraciones (ver
    // `RenderQueue::clear`): ninguna asignación nueva por frame en el camino caliente.
    let mut render_queue = RenderQueue::new();

    while window.is_open() && !input::quit_requested(&window) {
        // Delta time acotado: evita saltos de animación tras una pausa o un frame lento.
        let now = Instant::now();
        let dt = (now - last_frame).as_secs_f32().min(1.0 / 20.0);
        last_frame = now;
        fps_smoothed = fps_smoothed * 0.9 + (1.0 / dt.max(1e-4)) * 0.1;

        let input_state = input::read(&window);
        if input_state.reset {
            ship = Ship::new(spawn_point(&world));
            camera = ChaseCamera::new(&ship);
        }
        if input_state.pause {
            paused = !paused;
        }
        if input_state.toggle_manual {
            show_manual = !show_manual;
        }
        if input_state.toggle_frustum_culling {
            frustum_culling_enabled = !frustum_culling_enabled;
        }
        if input_state.toggle_max_lod {
            force_max_lod = !force_max_lod;
        }

        if !paused {
            elapsed += dt;
            world.update(elapsed);
            ship.update(&input_state, &world, dt);
            camera.update(&ship, dt);
        }

        let view_proj = proj * camera.view();
        let skybox_view_proj = proj * camera.view_rotation_only();
        let eye = camera.position;
        let uniforms = Uniforms { time: elapsed, light_dir };
        let frustum = Frustum::from_view_proj(view_proj);

        let mut objects_visible = 0u32;
        let mut objects_culled = 0u32;
        // Esfera visible según el frustum actual (o siempre visible con `F2` apagado),
        // y nivel de LOD según distancia relativa al radio (o máximo con `F3`).
        let mut visible_lod = |position: Vec3, radius: f32| -> Option<LodLevel> {
            if frustum_culling_enabled && !frustum.sphere_visible(position, radius) {
                objects_culled += 1;
                return None;
            }
            objects_visible += 1;
            Some(if force_max_lod {
                LodLevel::High
            } else {
                lod::select_lod(eye.distance(position), radius, config::LOD_MEDIUM_RATIO, config::LOD_LOW_RATIO)
            })
        };

        fb.clear(config::BACKGROUND_COLOR);
        skybox.draw(&mut fb, skybox_view_proj);

        render_queue.clear();

        if let Some(level) = visible_lod(world.sun_position, bodies::SUN_RADIUS) {
            let sun_model = Mat4::from_translation(world.sun_position) * Mat4::from_scale(Vec3::splat(bodies::SUN_RADIUS));
            render_queue.push_mesh(lod_mesh(level), sun_model, view_proj, eye, &sun_shader, (config::WIDTH, config::HEIGHT));
        }

        for (planet, shader) in world.planets.iter().zip(planet_shaders.iter()) {
            if let Some(level) = visible_lod(planet.position, planet.def.radius) {
                let model = Mat4::from_translation(planet.position)
                    * Quat::from_axis_angle(Vec3::Y, planet.rotation).to_mat4()
                    * Mat4::from_scale(Vec3::splat(planet.def.radius));
                render_queue.push_mesh(lod_mesh(level), model, view_proj, eye, shader.as_ref(), (config::WIDTH, config::HEIGHT));
            }
        }

        if let Some(level) = visible_lod(world.moon_position, bodies::MOON.radius) {
            let moon_model = Mat4::from_translation(world.moon_position)
                * Quat::from_axis_angle(Vec3::Y, world.moon_rotation).to_mat4()
                * Mat4::from_scale(Vec3::splat(bodies::MOON.radius));
            render_queue.push_mesh(lod_mesh(level), moon_model, view_proj, eye, &moon_shader, (config::WIDTH, config::HEIGHT));
        }

        let ship_model = Mat4::from_translation(ship.position) * ship.orientation.to_mat4() * Mat4::from_scale(Vec3::splat(0.58));
        render_queue.push_mesh(&ship_mesh, ship_model, view_proj, eye, &ship_shader, (config::WIDTH, config::HEIGHT));

        render_queue.rasterize_parallel(&mut fb, &uniforms, config::RENDER_BAND_ROWS);

        if show_manual {
            hud::draw_controls_manual(&mut fb);
        }
        hud::draw_performance_panel(
            &mut fb,
            &hud::PerfStats {
                fps: fps_smoothed,
                triangles_drawn: render_queue.stats.triangles_drawn,
                triangles_culled: render_queue.stats.triangles_culled,
                objects_visible,
                objects_culled,
                frustum_culling_enabled,
                max_lod_forced: force_max_lod,
            },
        );

        window.update_with_buffer(&fb.pixels, config::WIDTH, config::HEIGHT)?;

        title_timer += dt;
        if title_timer > 0.2 {
            title_timer = 0.0;
            // Indicador de dirección al objetivo: rumbo relativo en grados al
            // cuerpo celeste más cercano (0 = de frente, negativo = a la izquierda).
            let target_dir = ship.direction_to_target(&world);
            let bearing = target_dir.dot(ship.right()).atan2(target_dir.dot(ship.forward())).to_degrees();
            window.set_title(&format!(
                "Fantasticar | vel {:>5.1} u/s | rumbo {:>4.0} deg | crucero {} | estabilizacion {} | orbita {} | {}{}",
                ship.speed,
                bearing,
                if ship.cruise_enabled { "ON" } else { "off" },
                if ship.stabilize_assist { "ON" } else { "off" },
                if ship.orbit_assist { "ON" } else { "off" },
                if paused { "PAUSADO | " } else { "" },
                "F1 manual",
            ));
        }
    }

    Ok(())
}
