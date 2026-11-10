mod engine;
mod game;
mod shaders;

use std::time::Instant;

use minifb::{Window, WindowOptions};

use engine::culling::Frustum;
use engine::framebuffer::Framebuffer;
use engine::lod::{self, LodLevel, LodSet};
use engine::math::{Mat4, Quat, Vec3, Vec4};
use engine::mesh::{self, Mesh};
use engine::noise::Fbm;
use engine::obj;
use engine::rasterizer::RenderQueue;
use game::bodies::{self, ShaderKind};
use game::camera::ChaseCamera;
use game::combat::{Combat, CombatEvent};
use game::config;
use game::hud::{self, Messages, Telemetry};
use game::input;
use game::ship::Ship;
use game::skybox::Skybox;
use game::surface;
use game::world::World;
use shaders::earth::EarthShader;
use shaders::galactus::GalactusShader;
use shaders::ice::IceShader;
use shaders::laser::LaserShader;
use shaders::lava::LavaShader;
use shaders::metal::MetalShader;
use shaders::moon::MoonShader;
use shaders::rocky::RockyShader;
use shaders::ship::ShipShader;
use shaders::star::StarShader;
use shaders::surface::SurfaceShader;
use shaders::{FragmentShader, Uniforms};

/// Malla deformada por nivel de detalle: icosaedro subdividido y desplazado
/// con ruido multioctava (Fase 5). La usan tanto el Planeta Fragmentado como
/// Galactus (Fase 7), cada uno con su propia frecuencia/amplitud de ruido,
/// en vez de la esfera UV que comparten el resto de los cuerpos.
fn build_fractured_lod(seed: u32, octaves: u32, frequency: f32, amplitude: f32) -> LodSet<Mesh> {
    let build = |subdivisions: u32| {
        let mut m = mesh::icosphere(subdivisions);
        let noise = Fbm::new(seed).set_octaves(octaves).set_frequency(frequency);
        mesh::displace(&mut m, |dir| 1.0 + noise.sample3(dir) * amplitude);
        m
    };
    LodSet::new(
        build(config::FRACTURED_ICOSPHERE_SUBDIV_HIGH),
        build(config::FRACTURED_ICOSPHERE_SUBDIV_MEDIUM),
        build(config::FRACTURED_ICOSPHERE_SUBDIV_LOW),
    )
}

/// Proyecta un punto del mundo a espacio de pantalla; `None` si queda detrás
/// de la cámara. Devuelve también la profundidad NDC, para comparar contra
/// el z-buffer ya escrito por la escena.
fn project_to_screen(view_proj: Mat4, width: f32, height: f32, p: Vec3) -> Option<(f32, f32, f32)> {
    let clip = view_proj * p.extend(1.0);
    if clip.w <= 0.0001 {
        return None;
    }
    let inv_w = 1.0 / clip.w;
    let ndc_x = clip.x * inv_w;
    let ndc_y = clip.y * inv_w;
    let ndc_z = clip.z * inv_w;
    Some(((ndc_x * 0.5 + 0.5) * width, (1.0 - (ndc_y * 0.5 + 0.5)) * height, ndc_z))
}

/// Halo radial barato en espacio de pantalla (Fase 7): un mismo truco para
/// el brillo del sol y la atmósfera de la Tierra, solo cambiando color,
/// intensidad y cuánto se extiende más allá del disco real. No pinta donde
/// ya hay algo dibujado más cerca que el cuerpo (para no tapar lo que pasa
/// por delante), así que el halo solo se ve alrededor de la silueta real.
#[allow(clippy::too_many_arguments)]
fn draw_glow(
    fb: &mut Framebuffer,
    view_proj: Mat4,
    center: Vec3,
    cam_right: Vec3,
    body_radius: f32,
    color: Vec3,
    intensity: f32,
    screen_scale: f32,
) {
    let width = fb.width as f32;
    let height = fb.height as f32;
    let Some((cx, cy, cz)) = project_to_screen(view_proj, width, height, center) else { return };
    let Some((ex, ey, _)) = project_to_screen(view_proj, width, height, center + cam_right * body_radius) else { return };
    let glow_radius = (ex - cx).hypot(ey - cy) * screen_scale;
    if !glow_radius.is_finite() || glow_radius < 1.0 {
        return;
    }

    let min_x = (cx - glow_radius).floor().max(0.0) as i32;
    let max_x = (cx + glow_radius).ceil().min(width - 1.0) as i32;
    let min_y = (cy - glow_radius).floor().max(0.0) as i32;
    let max_y = (cy + glow_radius).ceil().min(height - 1.0) as i32;
    if min_x > max_x || min_y > max_y {
        return;
    }

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            let d = (dx * dx + dy * dy).sqrt() / glow_radius;
            if d >= 1.0 {
                continue;
            }
            // Ocluido por algo ya dibujado más cerca que el cuerpo: sin halo ahí.
            if fb.depth_at(x as usize, y as usize) < cz - 0.0005 {
                continue;
            }
            let alpha = ((1.0 - d).powf(2.0) * intensity).clamp(0.0, 1.0);
            if alpha <= 0.002 {
                continue;
            }
            fb.blend_pixel(x as usize, y as usize, cz, Vec4::new(color.x, color.y, color.z, alpha));
        }
    }
}

/// Shader de superficie según el tipo de dato de `bodies.rs`; construido una
/// sola vez al inicio (las tablas de ruido no se recrean por frame).
fn build_planet_shader(def: &bodies::BodyDef) -> Box<dyn FragmentShader> {
    match def.shader {
        ShaderKind::Earth => Box::new(EarthShader::new(def.seed)),
        ShaderKind::Lava => Box::new(LavaShader::new(def.seed)),
        ShaderKind::Ice => Box::new(IceShader::new(def.seed)),
        ShaderKind::Rocky => Box::new(RockyShader::new(def.seed)),
        ShaderKind::Metal => Box::new(MetalShader::new(def.seed)),
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
    let sphere_lod = LodSet::new(
        mesh::sphere(config::SPHERE_STACKS_HIGH, config::SPHERE_SLICES_HIGH),
        mesh::sphere(config::SPHERE_STACKS_MEDIUM, config::SPHERE_SLICES_MEDIUM),
        mesh::sphere(config::SPHERE_STACKS_LOW, config::SPHERE_SLICES_LOW),
    );
    let sun_shader = StarShader::new(bodies::SUN_SEED);
    let moon_shader = MoonShader::new(bodies::MOON.seed);
    let planet_shaders: Vec<Box<dyn FragmentShader>> = bodies::PLANETS.iter().map(build_planet_shader).collect();
    // Solo el Planeta Fragmentado (`fractured: true` en bodies.rs) usa una
    // malla propia deformada; el resto comparte `sphere_lod` (dato, no `if`).
    let planet_lods: Vec<Option<LodSet<Mesh>>> = bodies::PLANETS
        .iter()
        .map(|def| {
            if def.fractured {
                Some(build_fractured_lod(
                    def.seed,
                    config::FRACTURED_DEFORM_OCTAVES,
                    config::FRACTURED_DEFORM_FREQUENCY,
                    config::FRACTURED_DEFORM_AMPLITUDE,
                ))
            } else {
                None
            }
        })
        .collect();

    let surface_models = surface::build_models(config::SURFACE_MODEL_SEED);
    let surface_shaders: Vec<SurfaceShader> = surface::KINDS.iter().map(|&kind| SurfaceShader::for_kind(kind)).collect();
    let surface_instances = surface::scatter(config::SURFACE_SCATTER_SEED, config::SURFACE_SCATTER_COUNT);

    let skybox = Skybox::new(config::STARFIELD_SEED, config::STARFIELD_COUNT);

    // Galactus (Fase 7): malla propia deformada igual que el Planeta
    // Fragmentado, pero con su propia frecuencia/amplitud (grietas más
    // extremas) y su shader de amenaza.
    let galactus_lod = build_fractured_lod(
        bodies::GALACTUS.seed,
        config::GALACTUS_DEFORM_OCTAVES,
        config::GALACTUS_DEFORM_FREQUENCY,
        config::GALACTUS_DEFORM_AMPLITUDE,
    );
    let galactus_shader = GalactusShader::new(bodies::GALACTUS.seed);

    // Rayo de la nave (Fase 7): malla reutilizada entre todos los disparos,
    // solo cambia la matriz de modelo por proyectil.
    let laser_mesh = mesh::cylinder(6, config::LASER_RADIUS, config::LASER_LENGTH);
    let laser_shader = LaserShader::new(config::LASER_RADIUS);
    let mut combat = Combat::new();
    let mut messages = Messages::new();
    let mut last_collision_notified: Option<&'static str> = None;
    let mut was_warping = false;

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
    // `F1` oculta todo el HUD del jugador (manual, telemetría, mensajes); el
    // panel de rendimiento queda aparte porque es una herramienta de
    // depuración, no parte de la dirección artística del juego.
    let mut show_hud = true;
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
            show_hud = !show_hud;
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
            world.update_galactus(dt);
            ship.update(&input_state, &world, dt);
            camera.update(&ship, dt);

            for event in combat.update(input_state.fire, &ship, &mut world, dt) {
                match event {
                    CombatEvent::Fired => {}
                    CombatEvent::Hit => messages.push("IMPACTO EN GALACTUS", config::HERO_COLOR),
                    CombatEvent::Retreated => messages.push("GALACTUS SE REPLIEGA", config::THREAT_COLOR),
                }
            }

            // Mensaje solo al entrar en colisión con un cuerpo nuevo, no en
            // cada frame mientras se sigue tocando (evita spam en pantalla).
            if ship.last_collision != last_collision_notified
                && let Some(name) = ship.last_collision
            {
                messages.push(format!("COLISION: {}", name.to_uppercase()), config::THREAT_COLOR);
            }
            last_collision_notified = ship.last_collision;

            let is_warping = ship.warp_charge > 0.5;
            if is_warping && !was_warping {
                messages.push("IMPULSO DE WARP", config::HERO_COLOR);
            }
            was_warping = is_warping;

            messages.update(dt);
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
        skybox.draw(&mut fb, skybox_view_proj, ship.warp_charge);

        render_queue.clear();

        if let Some(level) = visible_lod(world.sun_position, bodies::SUN_RADIUS) {
            let sun_model = Mat4::from_translation(world.sun_position) * Mat4::from_scale(Vec3::splat(bodies::SUN_RADIUS));
            render_queue.push_mesh(sphere_lod.get(level), sun_model, view_proj, eye, &sun_shader, (config::WIDTH, config::HEIGHT));
        }

        for (i, (planet, shader)) in world.planets.iter().zip(planet_shaders.iter()).enumerate() {
            if let Some(level) = visible_lod(planet.position, planet.def.radius) {
                let model = Mat4::from_translation(planet.position)
                    * Quat::from_axis_angle(Vec3::Y, planet.rotation).to_mat4()
                    * Mat4::from_scale(Vec3::splat(planet.def.radius));
                let planet_mesh = planet_lods[i].as_ref().map_or_else(|| sphere_lod.get(level), |set| set.get(level));
                render_queue.push_mesh(planet_mesh, model, view_proj, eye, shader.as_ref(), (config::WIDTH, config::HEIGHT));
            }
        }

        if let Some(level) = visible_lod(world.moon_position, bodies::MOON.radius) {
            let moon_model = Mat4::from_translation(world.moon_position)
                * Quat::from_axis_angle(Vec3::Y, world.moon_rotation).to_mat4()
                * Mat4::from_scale(Vec3::splat(bodies::MOON.radius));
            render_queue.push_mesh(sphere_lod.get(level), moon_model, view_proj, eye, &moon_shader, (config::WIDTH, config::HEIGHT));
        }

        if let Some(level) = visible_lod(world.galactus_position, bodies::GALACTUS.radius) {
            let galactus_model = Mat4::from_translation(world.galactus_position)
                * Quat::from_axis_angle(Vec3::Y, world.galactus_rotation).to_mat4()
                * Mat4::from_scale(Vec3::splat(bodies::GALACTUS.radius));
            render_queue.push_mesh(galactus_lod.get(level), galactus_model, view_proj, eye, &galactus_shader, (config::WIDTH, config::HEIGHT));
        }

        // Objetos de superficie de la Tierra (Fase 5): solo de cerca, cada
        // uno con su propio descarte por frustum (no tienen niveles de LOD).
        let earth = &world.planets[bodies::EARTH_INDEX];
        if eye.distance(earth.position) < config::SURFACE_DRAW_DISTANCE {
            let earth_transform = Mat4::from_translation(earth.position) * Quat::from_axis_angle(Vec3::Y, earth.rotation).to_mat4();
            let earth_radius = earth.def.radius;
            for instance in &surface_instances {
                let model = surface::instance_model(instance, earth_transform, earth_radius);
                let world_pos = model.transform_point3(Vec3::ZERO);
                if frustum_culling_enabled && !frustum.sphere_visible(world_pos, instance.scale * 1.5) {
                    objects_culled += 1;
                    continue;
                }
                objects_visible += 1;
                let shader = &surface_shaders[instance.kind.index()];
                render_queue.push_mesh(surface_models.get(instance.kind), model, view_proj, eye, shader, (config::WIDTH, config::HEIGHT));
            }
        }

        let ship_model = Mat4::from_translation(ship.position) * ship.orientation.to_mat4() * Mat4::from_scale(Vec3::splat(0.58));
        render_queue.push_mesh(&ship_mesh, ship_model, view_proj, eye, &ship_shader, (config::WIDTH, config::HEIGHT));

        // Rayos en vuelo (Fase 7): el cilindro va de la posición actual hacia
        // atrás, orientado con `align_y_to_normal` a lo largo de su dirección.
        for laser in &combat.lasers {
            let center = laser.position - laser.direction * (config::LASER_LENGTH * 0.5);
            let laser_model = Mat4::align_y_to_normal(laser.direction, center);
            render_queue.push_mesh(&laser_mesh, laser_model, view_proj, eye, &laser_shader, (config::WIDTH, config::HEIGHT));
        }

        render_queue.rasterize_parallel(&mut fb, &uniforms, config::RENDER_BAND_ROWS);

        // Resplandores baratos en espacio de pantalla (Fase 7): brillo del
        // sol y atmósfera de la Tierra, el mismo truco con otro color/escala.
        let cam_forward = (camera.look_at - camera.position).normalize();
        let cam_right = cam_forward.cross(camera.up).normalize();
        draw_glow(
            &mut fb,
            view_proj,
            world.sun_position,
            cam_right,
            bodies::SUN_RADIUS,
            config::SUN_GLOW_COLOR,
            config::SUN_GLOW_INTENSITY,
            config::SUN_GLOW_SCREEN_SCALE,
        );
        draw_glow(
            &mut fb,
            view_proj,
            earth.position,
            cam_right,
            earth.def.radius,
            config::EARTH_GLOW_COLOR,
            config::EARTH_GLOW_INTENSITY,
            config::EARTH_GLOW_SCREEN_SCALE,
        );

        if show_hud {
            hud::draw_controls_manual(&mut fb);

            let nearest = world.nearest_body(ship.position);
            let target_dir = ship.direction_to_target(&world);
            let bearing = target_dir.dot(ship.right()).atan2(target_dir.dot(ship.forward())).to_degrees();
            hud::draw_telemetry_panel(
                &mut fb,
                &Telemetry {
                    speed: ship.speed,
                    cruise: ship.cruise_enabled,
                    stabilize: ship.stabilize_assist,
                    orbit: ship.orbit_assist,
                    warp_charge: ship.warp_charge,
                    target_name: nearest.name,
                    target_distance: nearest.surface_distance.max(0.0),
                    target_bearing_deg: bearing,
                    target_is_threat: nearest.name == bodies::GALACTUS.name,
                },
            );

            messages.draw(&mut fb);
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
            window.set_title(&format!(
                "Fantasticar: Defensa de la Tierra{}",
                if paused { " | PAUSADO" } else { "" },
            ));
        }
    }

    Ok(())
}
