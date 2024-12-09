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


    let mut loaded = three_d_asset::io::load_async(&["assets/checkerboard.jpg"]).await.unwrap();
    let mut cpu_texture: CpuTexture = loaded.deserialize("checkerboard").unwrap();
    // cpu_texture.wrap_s = Wrapping::Repeat;
    // cpu_texture.wrap_t = Wrapping::Repeat;
    cpu_texture.data.to_color();
    let cpu_material = CpuMaterial {
        albedo: Srgba { r: 220, g: 220, b: 220, a: 200, },
        albedo_texture: Some(cpu_texture),
        ..Default::default()
    };
    let material = PhysicalMaterial::new_transparent(&context, &cpu_material);



    let mut path_array= Vec::new();
    for i in -100..100 {
        let mut path = Vec::new();
        for j in -40..40 {
            let z = ((i as f32 + j as f32) * 0.1).sin();
            let s = 0.2;
            path.push(vec3(i as f32 * s, j as f32 * s, z));
        }
        path_array.push(path);
    }

    let ribbon = cpu_ribbon(&path_array);

    // Mesh
    let mesh = Gm::new(
        Mesh::new(&context, &ribbon),
        material,
    );

    window.render_loop(move |mut frame_input| {
        camera.set_viewport(frame_input.viewport);
        control.handle_events(&mut camera, &mut frame_input.events);

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




fn cpu_ribbon(path_array: &Vec<Vec<Vec3>>) -> CpuMesh {
    // path lengths
    let p = path_array.len();
    let l = path_array[0].len();

    // vertex data arrays
    let mut positions = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut uvs = Vec::new();

    // variables for uv mapping
    let mut u_distance = vec![0.0; l];
    let mut v_distance = vec![0.0; p];
    let mut u_total_distance = 0.0;
    let mut v_total_distance = 0.0;

    // positions
    for i in 0..p {
        for j in 0..l {
            let v3 = path_array[i][j].clone();
            positions.push(v3);
            if j > 0 {
                u_total_distance += (path_array[i][j] - path_array[i][j - 1]).magnitude();
                u_distance[j] = u_total_distance;
            }
            if i > 0 {
                v_total_distance += (path_array[i][j] - path_array[i - 1][j]).magnitude();
                v_distance[i] = v_total_distance;
            }
        }
    }

    // uvs
    for i in 0..p {
        for j in 0..l {
            uvs.push(vec2(
                u_distance[j] / u_total_distance,
                v_distance[i] / v_total_distance,

            ));
        }
    }
     
    // indices
    for i in 0..p - 1 {
        for j in 0..l - 1 {
            let i0 = i * l + j;
            let i1 = i * l + j + 1;
            let j0 = (i + 1) * l + j;
            let j1 = (i + 1) * l + j + 1;

            indices.push(i0 as u32);
            indices.push(i1 as u32);
            indices.push(j1 as u32);

            indices.push(j1 as u32);
            indices.push(j0 as u32);
            indices.push(i0 as u32);
        }
    }

    let mut cpu_mesh = CpuMesh {
        positions: Positions::F32(positions),
        indices: Indices::U32(indices),
        uvs: Some(uvs),
        ..Default::default()
    };
    cpu_mesh.compute_normals();
    cpu_mesh.compute_tangents();
    cpu_mesh
}