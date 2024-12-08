use three_d::*;

pub fn main() {
    let window = Window::new(WindowSettings {
        title: "Shapes!".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl();

    let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(5.0, 2.0, 2.5),
        vec3(0.0, 0.0, -0.5),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        1000.0,
    );
    let mut control = OrbitControl::new(camera.target(), 1.0, 100.0);
    let light0 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, vec3(0.0, -0.5, -0.5));
    let light1 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, vec3(0.0, 0.5, 0.5));

    let cpu_material = CpuMaterial {
        albedo: Srgba { r: 255, g: 0, b: 0, a: 200, },
        ..Default::default()
    };
    let material = PhysicalMaterial::new_transparent(&context, &cpu_material);

    // Geometry
    let cpu_mesh = CpuMesh {
        positions: Positions::F32( vec![
            vec3(0.0, 0.0, 0.0),
            vec3(1.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            vec3(1.0, 1.0, 0.0),
        ]),
        indices: Indices::U8(vec![0, 1, 2, 1, 3, 2]),
        ..Default::default()
    };

    // Mesh
    let mesh = Gm::new(
        Mesh::new(&context, &cpu_mesh),
        material,
    );



    window.render_loop(move |mut frame_input| {
        camera.set_viewport(frame_input.viewport);
        control.handle_events(&mut camera, &mut frame_input.events);

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
            .render(
                &camera,
                &mesh,
                &[&light0, &light1],
            );

        FrameOutput::default()
    });
}
