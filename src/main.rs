// Entry point for non-wasm
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    run().await;
}


use three_d::*;


pub async fn run() {
    let window = Window::new(WindowSettings {
        title: "Shapes!".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl();

    let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(0.0, 0.0, 10.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        1000.0,
    );
    let mut control = OrbitControl::new(camera.target(), 1.0, 100.0);
    let light0 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, vec3(0.0, -0.5, -0.5));
    let light1 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, vec3(0.0, 0.5, 0.5));


    let mut loaded = three_d_asset::io::load_async(&["assets/spriteAtlas.png"]).await.unwrap();
    let mut cpu_texture: CpuTexture = loaded.deserialize("spriteAtlas").unwrap();
    let mipmap = Some(Mipmap { max_ratio: 1, max_levels: 8, filter: Interpolation::Nearest, });
    cpu_texture.min_filter = Interpolation::Nearest;
    cpu_texture.mag_filter = Interpolation::Nearest;
    cpu_texture.wrap_s = Wrapping::Repeat;
    cpu_texture.wrap_t = Wrapping::Repeat;
    cpu_texture.mipmap = mipmap;
    cpu_texture.data.to_color();
    let cpu_material = CpuMaterial {
        albedo: Srgba { r: 220, g: 220, b: 255, a: 220, },
        albedo_texture: Some(cpu_texture),
        ..Default::default()
    };
    let material = PhysicalMaterial::new_transparent(&context, &cpu_material);



    let mut paths= Vec::new();
    let radius = 6.0;
    for i in -80..80 {
        let mut path = Vec::new();
        for j in -45..45 {
            let z = ((i as f32 + j as f32) * 0.1).sin() + radius * (j as f32 * 0.08).cos();
            let s = 0.2;
            path.push(vec3(i as f32 * s, j as f32 * s, z));
        }
        paths.push(path);
    }

    let cpu_mesh: CpuMesh = CpuMesh::ribbon(&paths);

    // Mesh
    let mut  mesh = Gm::new(
        Mesh::new(&context, &cpu_mesh),
        material,
    );

    window.render_loop(move |mut frame_input| {
        camera.set_viewport(frame_input.viewport);
        control.handle_events(&mut camera, &mut frame_input.events);
        let t = frame_input.accumulated_time as f32;
        update_paths(&mut paths, t);
        morph_ribbon(&mut mesh.geometry, &mut &paths);

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.1, 0.1, 0.1, 1.0, 1.0))
            .render(
                &camera,
                &mesh,
                &[&light0, &light1],
            );

        FrameOutput::default()
    });
}




fn morph_ribbon(mesh: &mut Mesh, paths: &mut &Vec<Vec<Vec3>>) {
    let mut positions = Vec::new();
    for i in 0..paths.len() {
        for j in 0..paths[i].len() {
            let v3 = paths[i][j].clone();
            positions.push(v3);
        }
    }
    let vb_pos = mesh.positions_mut();
    vb_pos.fill(&positions);
}

fn update_paths(paths: &mut Vec<Vec<Vec3>>, t: f32) {
    for i in 0..paths.len() {
        for j in 0..paths[i].len() {
            paths[i][j].z = paths[i][j].x * ((i + j) as f32 * 0.1).sin() * (t * 0.01).cos() * 0.3;
        }
    }
}