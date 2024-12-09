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



    let mut path_array= Vec::new();
    let radius = 6.0;
    for i in -80..80 {
        let mut path = Vec::new();
        for j in -45..45 {
            let z = ((i as f32 + j as f32) * 0.1).sin() + radius * (j as f32 * 0.08).cos();
            let s = 0.2;
            path.push(vec3(i as f32 * s, j as f32 * s, z));
        }
        path_array.push(path);
    }

    let ribbon = cpu_ribbon(&path_array);

    // Mesh
    let mut  mesh = Gm::new(
        Mesh::new(&context, &ribbon),
        material,
    );

    window.render_loop(move |mut frame_input| {
        camera.set_viewport(frame_input.viewport);
        control.handle_events(&mut camera, &mut frame_input.events);
        let t = frame_input.accumulated_time as f32;
        update_array_path(&mut path_array, t);
        morph_ribbon(&mut mesh.geometry, &mut &path_array);

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
    let mut u_distances = vec![vec![0.0; l]; p]; // distance along the horizontal paths
    let mut v_distances = vec![vec![0.0; p]; l]; // distance along the vertical paths
    let mut u_total_distance = 0.0;
    let mut v_total_distance = 0.0;

    // positions
    for i in 0..p {
        for j in 0..l {
            let v3 = path_array[i][j].clone();
            positions.push(v3);
            if j > 0 {
                u_total_distance += (path_array[i][j] - path_array[i][j - 1]).magnitude();
                u_distances[i][j] = u_total_distance;
            }
        }
    }

    // uvs

    // compute vertical distances
    for j in 0..l {
        for i in 0..p {
            if i > 0 {
                v_total_distance += (path_array[i][j] - path_array[i - 1][j]).magnitude();
                v_distances[j][i] = v_total_distance;
            }
        }
    }
    for i in 0..p {
        for j in 0..l {

            let u = u_distances[i][j] / u_total_distance;
            let v = 1.0 - v_distances[j][i] / v_total_distance;
            uvs.push(vec2(u,v));
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


fn morph_ribbon(mesh: &mut Mesh, path_array: &mut &Vec<Vec<Vec3>>) {
    let mut positions = Vec::new();
    for i in 0..path_array.len() {
        for j in 0..path_array[i].len() {
            let v3 = path_array[i][j].clone();
            positions.push(v3);
        }
    }
    let vb_pos = mesh.positions_mut();
    vb_pos.fill(&positions);
}

fn update_array_path(path_array: &mut Vec<Vec<Vec3>>, t: f32) {
    for i in 0..path_array.len() {
        for j in 0..path_array[i].len() {
            path_array[i][j].z = path_array[i][j].x * ((i + j) as f32 * 0.1).sin() * (t * 0.01).cos() * 0.3;
        }
    }
}