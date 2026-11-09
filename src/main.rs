mod engine;
mod game;
mod shaders;

use std::time::Instant;

use minifb::{Window, WindowOptions};

use engine::framebuffer::Framebuffer;
use engine::math::{Mat4, Vec3};
use engine::mesh::{self, Mesh};
use engine::obj;
use engine::rasterizer;
use game::camera::ChaseCamera;
use game::config;
use game::hud;
use game::input;
use game::ship::Ship;
use game::skybox::Skybox;
use shaders::ship::ShipShader;
use shaders::star::StarShader;
use shaders::Uniforms;

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

    // Faro de navegación: punto fijo que sirve de objetivo mientras no hay
    // planetas (Fase 3); indica dirección y dispara el límite de velocidad.
    let beacon_mesh = mesh::sphere(16, 20);
    let beacon_shader = StarShader::new(99);

    let skybox = Skybox::new(config::STARFIELD_SEED, config::STARFIELD_COUNT);

    let mut ship = Ship::new(Vec3::new(0.0, 0.0, 20.0));
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
            ship = Ship::new(Vec3::new(0.0, 0.0, 20.0));
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
            ship.update(&input_state, dt);
            camera.update(&ship, dt);
        }

        let view_proj = proj * camera.view();
        let skybox_view_proj = proj * camera.view_rotation_only();
        let eye = camera.position;
        let uniforms = Uniforms { time: elapsed, light_dir };

        fb.clear(config::BACKGROUND_COLOR);
        skybox.draw(&mut fb, skybox_view_proj);

        let beacon_model = Mat4::from_translation(config::NAV_TARGET_POINT) * Mat4::from_scale(Vec3::splat(1.2));
        rasterizer::draw_mesh(&mut fb, &beacon_mesh, beacon_model, view_proj, eye, &beacon_shader, &uniforms);

        let ship_model = Mat4::from_translation(ship.position) * ship.orientation.to_mat4() * Mat4::from_scale(Vec3::splat(0.58));
        rasterizer::draw_mesh(&mut fb, &ship_mesh, ship_model, view_proj, eye, &ship_shader, &uniforms);

        if show_manual {
            hud::draw_controls_manual(&mut fb);
        }

        window.update_with_buffer(&fb.pixels, config::WIDTH, config::HEIGHT)?;

        title_timer += dt;
        if title_timer > 0.2 {
            title_timer = 0.0;
            // Indicador de dirección al objetivo: rumbo relativo en grados al faro
            // de navegación (0 = de frente, negativo = a la izquierda).
            let target_dir = ship.direction_to_target();
            let bearing = target_dir.dot(ship.right()).atan2(target_dir.dot(ship.forward())).to_degrees();
            window.set_title(&format!(
                "Fantasticar | vel {:>5.1} u/s | rumbo faro {:>4.0} deg | crucero {} | estabilizacion {} | {}{}",
                ship.speed,
                bearing,
                if ship.cruise_enabled { "ON" } else { "off" },
                if ship.stabilize_assist { "ON" } else { "off" },
                if paused { "PAUSADO | " } else { "" },
                "F1 manual",
            ));
        }
    }

    Ok(())
}
