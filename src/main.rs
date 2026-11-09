mod engine;
mod game;
mod shaders;

use std::time::Instant;

use minifb::{Window, WindowOptions};

use engine::framebuffer::Framebuffer;
use engine::math::{Mat4, Quat, Vec3};
use engine::mesh::{self, Mesh};
use engine::obj;
use engine::rasterizer;
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

    // Geometría esférica única, reutilizada para el sol, cada planeta y la
    // luna; el tamaño real lo da la matriz de modelo.
    let sphere_mesh = mesh::sphere(config::SPHERE_STACKS, config::SPHERE_SLICES);
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

    while window.is_open() && !input::quit_requested(&window) {
        // Delta time acotado: evita saltos de animación tras una pausa o un frame lento.
        let now = Instant::now();
        let dt = (now - last_frame).as_secs_f32().min(1.0 / 20.0);
        last_frame = now;

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

        fb.clear(config::BACKGROUND_COLOR);
        skybox.draw(&mut fb, skybox_view_proj);

        let sun_model = Mat4::from_translation(world.sun_position) * Mat4::from_scale(Vec3::splat(bodies::SUN_RADIUS));
        rasterizer::draw_mesh(&mut fb, &sphere_mesh, sun_model, view_proj, eye, &sun_shader, &uniforms);

        for (planet, shader) in world.planets.iter().zip(planet_shaders.iter()) {
            let model = Mat4::from_translation(planet.position)
                * Quat::from_axis_angle(Vec3::Y, planet.rotation).to_mat4()
                * Mat4::from_scale(Vec3::splat(planet.def.radius));
            rasterizer::draw_mesh(&mut fb, &sphere_mesh, model, view_proj, eye, shader.as_ref(), &uniforms);
        }

        let moon_model = Mat4::from_translation(world.moon_position)
            * Quat::from_axis_angle(Vec3::Y, world.moon_rotation).to_mat4()
            * Mat4::from_scale(Vec3::splat(bodies::MOON.radius));
        rasterizer::draw_mesh(&mut fb, &sphere_mesh, moon_model, view_proj, eye, &moon_shader, &uniforms);

        let ship_model = Mat4::from_translation(ship.position) * ship.orientation.to_mat4() * Mat4::from_scale(Vec3::splat(0.58));
        rasterizer::draw_mesh(&mut fb, &ship_mesh, ship_model, view_proj, eye, &ship_shader, &uniforms);

        if show_manual {
            hud::draw_controls_manual(&mut fb);
        }

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
