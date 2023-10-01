// Uncomment these following global attributes to silence most warnings of "low" interest:
/*
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unreachable_code)]
#![allow(unused_mut)]
#![allow(unused_unsafe)]
#![allow(unused_variables)]
*/
extern crate nalgebra_glm as glm;
use std::mem::ManuallyDrop;
use std::pin::Pin;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::{mem, os::raw::c_void, ptr};

mod mesh;
mod obj_reader;
mod scene_graph;
mod shader;
mod shape_generator;
mod toolbox;
mod util;

use glm::pi;
use glutin::event::{
    DeviceEvent,
    ElementState::{Pressed, Released},
    Event, KeyboardInput,
    VirtualKeyCode::{self, *},
    WindowEvent,
};
use glutin::event_loop::ControlFlow;
use obj_reader::ObjReader;
use rand::Rng;
use scene_graph::SceneNode;
use shape_generator::ShapeGenerator;

// initial window size
const INITIAL_SCREEN_W: u32 = 800;
const INITIAL_SCREEN_H: u32 = 600;

// == // Helper functions to make interacting with OpenGL a little bit prettier. You *WILL* need these! // == //

// Get the size of an arbitrary array of numbers measured in bytes
// Example usage:  pointer_to_array(my_array)
fn byte_size_of_array<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}

// Get the OpenGL-compatible pointer to an arbitrary array of numbers
// Example usage:  pointer_to_array(my_array)
fn pointer_to_array<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
}

// Get the size of the given type in bytes
// Example usage:  size_of::<u64>()
fn size_of<T>() -> i32 {
    mem::size_of::<T>() as i32
}

// Get an offset in bytes for n units of type T, represented as a relative pointer
// Example usage:  offset::<u64>(4)
fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}

struct Position {
    x: f32,
    y: f32,
    z: f32,
    pitch: f32,
    yaw: f32,
}

struct HelicopterNode {
    body: ManuallyDrop<Pin<Box<SceneNode>>>,
    door: ManuallyDrop<Pin<Box<SceneNode>>>,
    main_rotor: ManuallyDrop<Pin<Box<SceneNode>>>,
    tail_rotor: ManuallyDrop<Pin<Box<SceneNode>>>,
}

static mut uniform_time_location: i32 = 0;
static mut uniform_matrix_location: i32 = 0;
static mut uniform_normal_matrix_location: i32 = 0;

// Get a null pointer (equivalent to an offset of 0)
// ptr::null()

// == // Generate your VAO here
unsafe fn create_vao(
    vertices: &Vec<f32>,
    indices: &Vec<u32>,
    colors: &Vec<f32>,
    normals: &Vec<f32>,
) -> u32 {
    // Generate a VAO and bind it
    let n_vao: gl::types::GLsizei = 1;
    let mut vao_ids = 0;

    gl::GenVertexArrays(n_vao, &mut vao_ids);
    gl::BindVertexArray(vao_ids);

    // Generate a VBO and bind it
    let n_vbo = 1;
    let mut vbo_ids = 0;

    gl::GenBuffers(n_vbo, &mut vbo_ids);
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo_ids);

    // Fill VBO with data
    gl::BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(vertices),
        pointer_to_array(vertices),
        gl::STATIC_DRAW,
    );

    // Configure a VAP for the vertex-position and enable it
    let vap_index = 0;
    gl::VertexAttribPointer(
        vap_index,
        3,
        gl::FLOAT,
        gl::FALSE,
        0 * size_of::<f32>(),
        offset::<f32>(0),
    );
    gl::EnableVertexAttribArray(vap_index);

    // TASK 2.1 a)
    // Generate a CBO for colors and bind it
    let n_cbo = 1;
    let mut cbo_ids = 0;

    gl::GenBuffers(n_cbo, &mut cbo_ids);
    gl::BindBuffer(gl::ARRAY_BUFFER, cbo_ids);

    // Fill CBO with data
    gl::BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(colors),
        pointer_to_array(colors),
        gl::STATIC_DRAW,
    );

    // Configure a VAP for the vertex-color and enable it
    gl::VertexAttribPointer(
        vap_index + 1,
        4,
        gl::FLOAT,
        gl::FALSE,
        0 * size_of::<f32>(),
        offset::<f32>(0),
    );
    gl::EnableVertexAttribArray(vap_index + 1);

    // TASK 3.1 b)

    // Generate a NBO for colors and bind it
    if normals.len() > 0 {
        let n_nbo = 1;
        let mut nbo_ids = 0;

        gl::GenBuffers(n_nbo, &mut nbo_ids);
        gl::BindBuffer(gl::ARRAY_BUFFER, nbo_ids);

        // Fill NBO with data
        gl::BufferData(
            gl::ARRAY_BUFFER,
            byte_size_of_array(normals),
            pointer_to_array(normals),
            gl::STATIC_DRAW,
        );

        // Configure a VAP for the vertex-color and enable it
        gl::VertexAttribPointer(
            vap_index + 2,
            3,
            gl::FLOAT,
            gl::FALSE,
            0 * size_of::<f32>(),
            offset::<f32>(0),
        );
        gl::EnableVertexAttribArray(vap_index + 2);
    }

    // Generate a IBO and bind it
    let n_ibo = 1;
    let mut ibo_ids = 0;

    gl::GenBuffers(n_ibo, &mut ibo_ids);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo_ids);

    // Fill IBO with data
    gl::BufferData(
        gl::ELEMENT_ARRAY_BUFFER,
        byte_size_of_array(indices),
        pointer_to_array(indices),
        gl::STATIC_DRAW,
    );

    // Return the ID of the VAO
    vao_ids
}

fn main() {
    // Set up the necessary objects to deal with windows and event handling
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize::new(
            INITIAL_SCREEN_W,
            INITIAL_SCREEN_H,
        ));
    let cb = glutin::ContextBuilder::new().with_vsync(true);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    // Uncomment these if you want to use the mouse for controls, but want it to be confined to the screen and/or invisible.
    // windowed_context.window().set_cursor_grab(true).expect("failed to grab cursor");
    // windowed_context.window().set_cursor_visible(false);

    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Make a reference of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    // Set up shared tuple for tracking mouse movement between frames
    let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));
    // Make a reference of this tuple to send to the render thread
    let mouse_delta = Arc::clone(&arc_mouse_delta);

    // Set up shared tuple for tracking changes to the window size
    let arc_window_size = Arc::new(Mutex::new((INITIAL_SCREEN_W, INITIAL_SCREEN_H, false)));
    // Make a reference of this tuple to send to the render thread
    let window_size = Arc::clone(&arc_window_size);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers.
        // This has to be done inside of the rendering thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        let mut window_aspect_ratio = INITIAL_SCREEN_W as f32 / INITIAL_SCREEN_H as f32;

        // Set up openGL
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());

            // Print some diagnostics
            println!(
                "{}: {}",
                util::get_gl_string(gl::VENDOR),
                util::get_gl_string(gl::RENDERER)
            );
            println!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
            println!(
                "GLSL\t: {}",
                util::get_gl_string(gl::SHADING_LANGUAGE_VERSION)
            );
        }

        // == // Set up your VAO around here

        // TASK 1.1 c)
        // let (verticies, indicies) = ShapeGenerator::generate_n_force(2, 1., 1.);

        // TASK 1.2 a)
        // let verticies = vec![0.6, -0.8, -1.2, 0., 0.4, 0., -0.8, -0.2, 1.2];
        // let indicies = vec![0, 1, 2];

        // TASK 1.2 b)
        // let front_facing_indicies = vec![0, 1, 2]; // [1, 2, 0], [2, 0, 1]
        // let back_facing_indicies = vec![2, 1, 0]; // [1, 0, 2], [0, 2, 1]
        // indicies.splice(0..3, back_facing_indicies);

        // TASK 1.3
        // let (verticies, indicies) = ShapeGenerator::generate_circle(20, 0.5);
        // let (verticies, indicies) = ShapeGenerator::generate_spiral(50, 0.05, 3., 1., 0.15);
        // let (verticies, indicies) = ShapeGenerator::generate_square(2.0);
        // let (verticies, indicies) = ObjReader::read("./resources/monke.obj");
        // let (verticies, indicies) = ShapeGenerator::generate_sine(100, 2., 0.19);

        // TASK 2.1 b)
        // let (verticies, indicies) = ShapeGenerator::generate_n_force(2, 1., 1.);
        // let mut rng = rand::thread_rng();
        // let mut colors = vec![];
        // for _ in 0..verticies.len() {
        //     colors = vec![colors, vec![rng.gen(), rng.gen(), rng.gen(), 1.]].concat();
        // }

        // TASK 2.2 a) b)

        // let (verticies, indicies) = ShapeGenerator::overlapping_triangles(1., 1., 0.2);
        // let mut colors = vec![0.; 9 * 4];
        // let alpha = 0.5;
        // for n in 0..3 {
        //     // Create red, green and blue triangle
        //     // 2 = closest, 0 = farthermost
        //     let color = [
        //         if n == 0 { 1. } else { 0. },
        //         if n == 1 { 1. } else { 0. },
        //         if n == 2 { 1. } else { 0. },
        //         alpha,
        //     ];
        //     let color2 = [
        //         if n == 1 { 1. } else { 0. },
        //         if n == 2 { 1. } else { 0. },
        //         if n == 0 { 1. } else { 0. },
        //         alpha,
        //     ];
        //     // Each triangle consist of 3 verticies, hence three splices
        //     colors.splice((n * 12)..(n * 12 + 4), color);
        //     colors.splice((n * 12 + 4)..(n * 12 + 8), color2);
        //     colors.splice((n * 12 + 8)..(n * 12 + 12), color);
        // }

        // TASK 2.5 b)
        // let (verticies, indicies) = ShapeGenerator::flat_thing();
        // let colors = vec![1., 0., 0., 1., 0., 1., 0., 1., 0., 0., 1., 1.];

        // TASK 3.1 a)
        let terrain_mesh = mesh::Terrain::load("./resources/lunarsurface.obj");

        unsafe fn create_vao_from_mesh(m: &mesh::Mesh) -> u32 {
            create_vao(&m.vertices, &m.indices, &m.colors, &m.normals)
        }

        let terrain_vao = unsafe { create_vao_from_mesh(&terrain_mesh) };

        // let colors = vec![0.5, 0.5, 0.5, 1.];
        // let normals = Vec::new();
        // let my_vao = unsafe { create_vao(&verticies, &indicies, &colors, &normals) };

        // TASK 3.2 a)
        // TASK 3.2 b)

        let mut terrain_node = SceneNode::from_vao(terrain_vao, terrain_mesh.indices.len() as i32);

        let helicopter_mesh = mesh::Helicopter::load("./resources/helicopter.obj");

        let heli_body_vao = unsafe { create_vao_from_mesh(&helicopter_mesh.body) };
        let heli_door_vao = unsafe { create_vao_from_mesh(&helicopter_mesh.door) };
        let heli_main_rotor_vao = unsafe { create_vao_from_mesh(&helicopter_mesh.main_rotor) };
        let heli_tail_rotor_vao = unsafe { create_vao_from_mesh(&helicopter_mesh.tail_rotor) };

        // Create array of helicopters from vao
        let mut helicopter_nodes: Vec<HelicopterNode> = Vec::new();
        for _ in 0..5 {
            let mut heli_body_node =
                SceneNode::from_vao(heli_body_vao, helicopter_mesh.body.indices.len() as i32);
            let mut heli_door_node =
                SceneNode::from_vao(heli_door_vao, helicopter_mesh.door.indices.len() as i32);
            let mut heli_main_rotor_node = SceneNode::from_vao(
                heli_main_rotor_vao,
                helicopter_mesh.main_rotor.indices.len() as i32,
            );
            let mut heli_tail_rotor_node = SceneNode::from_vao(
                heli_tail_rotor_vao,
                helicopter_mesh.tail_rotor.indices.len() as i32,
            );
            heli_tail_rotor_node.reference_point = glm::vec3(0.35, 2.3, 10.4);

            heli_body_node.add_child(&mut heli_door_node);
            heli_body_node.add_child(&mut heli_main_rotor_node);
            heli_body_node.add_child(&mut heli_tail_rotor_node);

            helicopter_nodes.push(HelicopterNode {
                body: heli_body_node,
                door: heli_door_node,
                main_rotor: heli_main_rotor_node,
                tail_rotor: heli_tail_rotor_node,
            });
        }

        helicopter_nodes
            .iter()
            .for_each(|h| terrain_node.add_child(&h.body));

        let mut scene_node = SceneNode::new();
        scene_node.add_child(&terrain_node);

        // == // Set up your shaders here

        // Create shader object
        let simple_shader = unsafe {
            shader::ShaderBuilder::new()
                .attach_file("./shaders/simple.vert")
                .attach_file("./shaders/sunlight.frag")
                .link()
        };

        // Set bindings for shader, VAO and uniform variable
        unsafe {
            simple_shader.activate();
            // gl::BindVertexArray(my_vao);

            // Used by changing.frag
            uniform_time_location = simple_shader.get_uniform_location("time");

            uniform_matrix_location = simple_shader.get_uniform_location("matrix");
            uniform_normal_matrix_location = simple_shader.get_uniform_location("normal_matrix");
        };

        // Perspective projection properties
        let fovy = 20.;
        let near = 1.;
        let far = 1000.;

        let mut camera_pos = Position {
            x: 0.,
            y: 0.,
            z: 0.,
            pitch: 0.,
            yaw: 0.,
        };

        let mut perspective: glm::Mat4;
        let move_z: glm::Mat4 = glm::translation(&glm::vec3(0., 0., 0.));
        let mut translate_camera: glm::Mat4 =
            glm::translation(&glm::vec3(-camera_pos.x, -camera_pos.y, -camera_pos.z));
        let mut pitch_camera: glm::Mat4 = glm::rotation(camera_pos.pitch, &glm::vec3(1., 0., 0.));
        let mut yaw_camera: glm::Mat4 = glm::rotation(camera_pos.yaw, &glm::vec3(0., 1., 0.));

        let mut view_direction = glm::vec3(
            (camera_pos.yaw as f64).sin() as f32,
            (camera_pos.pitch as f64).sin() as f32,
            -(camera_pos.yaw as f64).cos() as f32,
        );
        let mut normal_view_direction = glm::cross(&view_direction, &glm::vec3(0., 1., 0.));
        let movement_speed = 10.;

        unsafe fn draw_scene(
            node: &scene_graph::SceneNode,
            view_projection_matrix: &glm::Mat4,
            transformation_so_far: &glm::Mat4,
        ) {
            // Perform any logic needed before drawing the node
            let position_transformation = glm::translation(&glm::vec3(
                node.position.x,
                node.position.y,
                node.position.z,
            ));

            let rotation_transformation = // Comment here to get nice formatting
                glm::translation(&glm::vec3(
                    node.reference_point.x,
                    node.reference_point.y,
                    node.reference_point.z,
                )) // Comment there to get nice formatting
                * glm::rotation(node.rotation.x, &glm::vec3(1., 0., 0.))
                * glm::rotation(node.rotation.y, &glm::vec3(0., 1., 0.))
                * glm::rotation(node.rotation.z, &glm::vec3(0., 0., 1.))
                * glm::translation(&glm::vec3(
                    -node.reference_point.x,
                    -node.reference_point.y,
                    -node.reference_point.z,
                ));

            let next_transformation: glm::Mat4 =
                transformation_so_far * position_transformation * rotation_transformation;

            // Check if node is drawable, if so: set uniforms, bind VAO and draw VAO
            if node.vao_id != 0 {
                let uniform_matrix = view_projection_matrix * next_transformation;
                gl::UniformMatrix4fv(
                    uniform_matrix_location,
                    1,
                    gl::FALSE,
                    uniform_matrix.as_ptr(),
                );

                gl::UniformMatrix4fv(
                    uniform_normal_matrix_location,
                    1,
                    gl::FALSE,
                    next_transformation.as_ptr(),
                );

                gl::BindVertexArray(node.vao_id);
                gl::DrawElements(
                    gl::TRIANGLES,
                    node.index_count,
                    gl::UNSIGNED_INT,
                    0 as *const _,
                )
            }
            // Recurse
            for &child in &node.children {
                draw_scene(&*child, view_projection_matrix, &next_transformation);
            }
        }

        // The main rendering loop
        let first_frame_time = std::time::Instant::now();
        let mut prevous_frame_time = first_frame_time;
        loop {
            // Compute time passed since the previous frame and since the start of the program
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(prevous_frame_time).as_secs_f32();
            prevous_frame_time = now;

            // Handle resize events
            if let Ok(mut new_size) = window_size.lock() {
                if new_size.2 {
                    context.resize(glutin::dpi::PhysicalSize::new(new_size.0, new_size.1));
                    window_aspect_ratio = new_size.0 as f32 / new_size.1 as f32;
                    (*new_size).2 = false;
                    println!("Resized");
                    unsafe {
                        gl::Viewport(0, 0, new_size.0 as i32, new_size.1 as i32);
                    }
                }
            }

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        // Movement along camera direction
                        VirtualKeyCode::A => {
                            camera_pos.x -= normal_view_direction.x * delta_time * movement_speed;
                            camera_pos.y -= normal_view_direction.y * delta_time * movement_speed;
                            camera_pos.z -= normal_view_direction.z * delta_time * movement_speed;
                            translate_camera = glm::translation(&glm::vec3(
                                -camera_pos.x,
                                -camera_pos.y,
                                -camera_pos.z,
                            ));
                        }
                        VirtualKeyCode::D => {
                            camera_pos.x += normal_view_direction.x * delta_time * movement_speed;
                            camera_pos.y += normal_view_direction.y * delta_time * movement_speed;
                            camera_pos.z += normal_view_direction.z * delta_time * movement_speed;
                            translate_camera = glm::translation(&glm::vec3(
                                -camera_pos.x,
                                -camera_pos.y,
                                -camera_pos.z,
                            ));
                        }
                        VirtualKeyCode::W => {
                            camera_pos.x += view_direction.x * delta_time * movement_speed;
                            camera_pos.y += view_direction.y * delta_time * movement_speed;
                            camera_pos.z += view_direction.z * delta_time * movement_speed;
                            translate_camera = glm::translation(&glm::vec3(
                                -camera_pos.x,
                                -camera_pos.y,
                                -camera_pos.z,
                            ));
                        }
                        VirtualKeyCode::S => {
                            camera_pos.x -= view_direction.x * delta_time * movement_speed;
                            camera_pos.y -= view_direction.y * delta_time * movement_speed;
                            camera_pos.z -= view_direction.z * delta_time * movement_speed;
                            translate_camera = glm::translation(&glm::vec3(
                                -camera_pos.x,
                                -camera_pos.y,
                                -camera_pos.z,
                            ));
                        }
                        VirtualKeyCode::Space => {
                            camera_pos.y += 1. * delta_time * movement_speed;
                            translate_camera = glm::translation(&glm::vec3(
                                -camera_pos.x,
                                -camera_pos.y,
                                -camera_pos.z,
                            ));
                        }
                        VirtualKeyCode::LShift => {
                            camera_pos.y -= 1. * delta_time * movement_speed;
                            translate_camera = glm::translation(&glm::vec3(
                                -camera_pos.x,
                                -camera_pos.y,
                                -camera_pos.z,
                            ));
                        }
                        VirtualKeyCode::Right => {
                            camera_pos.yaw += 1. * delta_time;

                            yaw_camera = glm::rotation(camera_pos.yaw, &glm::vec3(0., 1., 0.));
                            view_direction = glm::vec3(
                                (camera_pos.yaw as f64).sin() as f32,
                                (camera_pos.pitch as f64).sin() as f32,
                                -(camera_pos.yaw as f64).cos() as f32,
                            );
                            normal_view_direction =
                                glm::cross(&view_direction, &glm::vec3(0., 1., 0.));
                        }
                        VirtualKeyCode::Left => {
                            camera_pos.yaw -= 1. * delta_time;

                            yaw_camera = glm::rotation(camera_pos.yaw, &glm::vec3(0., 1., 0.));
                            view_direction = glm::vec3(
                                (camera_pos.yaw as f64).sin() as f32,
                                (camera_pos.pitch as f64).sin() as f32,
                                -(camera_pos.yaw as f64).cos() as f32,
                            );
                            normal_view_direction =
                                glm::cross(&view_direction, &glm::vec3(0., 1., 0.));
                        }
                        VirtualKeyCode::Up => {
                            camera_pos.pitch += 1. * delta_time;

                            pitch_camera = glm::rotation(-camera_pos.pitch, &glm::vec3(1., 0., 0.));

                            view_direction = glm::vec3(
                                (camera_pos.yaw as f64).sin() as f32,
                                (camera_pos.pitch as f64).sin() as f32,
                                -(camera_pos.yaw as f64).cos() as f32,
                            );
                            normal_view_direction =
                                glm::cross(&view_direction, &glm::vec3(0., 1., 0.));
                        }
                        VirtualKeyCode::Down => {
                            camera_pos.pitch -= 1. * delta_time;

                            pitch_camera = glm::rotation(-camera_pos.pitch, &glm::vec3(1., 0., 0.));
                            view_direction = glm::vec3(
                                (camera_pos.yaw as f64).sin() as f32,
                                (camera_pos.pitch as f64).sin() as f32,
                                -(camera_pos.yaw as f64).cos() as f32,
                            );
                            normal_view_direction =
                                glm::cross(&view_direction, &glm::vec3(0., 1., 0.));
                        }

                        // default handler:
                        _ => {}
                    }
                }
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {
                // == // Optionally access the acumulated mouse movement between
                // == // frames here with `delta.0` and `delta.1`

                *delta = (0.0, 0.0); // reset when done
            }

            // == // Please compute camera transforms here (exercise 2 & 3)

            unsafe {
                // heli_body_node.rotation.y = 0.5 * elapsed;

                let helicoper_nodes_n = helicopter_nodes.len();

                for (i, h) in helicopter_nodes.iter_mut().enumerate() {
                    let lap = 2. * std::f32::consts::PI;
                    let heading = toolbox::simple_heading_animation(
                        elapsed + lap / (helicoper_nodes_n as f32) * (i as f32),
                    );

                    h.body.position.x = heading.x;
                    h.body.position.z = heading.z;
                    h.body.rotation.x = heading.pitch;
                    h.body.rotation.z = heading.roll;
                    h.body.rotation.y = heading.yaw;

                    h.main_rotor.rotation.y = 10. * elapsed;
                    h.tail_rotor.rotation.x = 10. * elapsed;
                }

                // Clear the color and depth buffers
                gl::ClearColor(0.035, 0.046, 0.078, 1.0); // night sky, full opacity
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                // == // Issue the necessary gl:: commands to draw your scene here
                perspective = glm::perspective(window_aspect_ratio, fovy, near, far);

                // Update uniform variables
                perspective *= pitch_camera;
                perspective *= yaw_camera;
                perspective *= translate_camera;
                perspective *= move_z;

                gl::Uniform1f(uniform_time_location, elapsed);
                // gl::UniformMatrix4fv(uniform_matrix_location, 1, gl::FALSE, perspective.as_ptr());

                draw_scene(&scene_node, &perspective, &glm::identity());

                // Display the new color buffer on the display
                context.swap_buffers().unwrap(); // we use "double buffering" to avoid artifacts
            }
        }
    });

    // == //
    // == // From here on down there are only internals.
    // == //

    // Keep track of the health of the rendering thread
    let render_thread_healthy = Arc::new(RwLock::new(true));
    let render_thread_watchdog = Arc::clone(&render_thread_healthy);
    thread::spawn(move || {
        if !render_thread.join().is_ok() {
            if let Ok(mut health) = render_thread_watchdog.write() {
                println!("Render thread panicked!");
                *health = false;
            }
        }
    });

    // Start the event loop -- This is where window events are initially handled
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(physical_size),
                ..
            } => {
                println!(
                    "New window size! width: {}, height: {}",
                    physical_size.width, physical_size.height
                );
                if let Ok(mut new_size) = arc_window_size.lock() {
                    *new_size = (physical_size.width, physical_size.height, true);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: key_state,
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        }
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }

                // Handle Escape and Q keys separately
                match keycode {
                    Escape => {
                        *control_flow = ControlFlow::Exit;
                    }
                    Q => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                }
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            }
            _ => {}
        }
    });
}
