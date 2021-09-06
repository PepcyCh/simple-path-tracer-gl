mod core;
mod loader;
mod opengl;
mod renderer;
mod uniforms;

use glfw::Context;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 1 {
        println!("Usage: simple-path-tracer-gl <path-to-json>");
        return Ok(());
    }

    let mut renderer = loader::load(&args[0])?;

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let (mut window, events) = glfw
        .create_window(
            renderer.output_config.width * renderer.output_config.scale,
            renderer.output_config.height * renderer.output_config.scale,
            "simple-path-tracer-gl",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create glfw window");

    gl::load_with(|s| window.get_proc_address(s) as *const _);
    let mut gl_major: gl::types::GLint = 0;
    let mut gl_minor: gl::types::GLint = 0;
    unsafe {
        gl::GetIntegerv(gl::MAJOR_VERSION, &mut gl_major as *mut _);
        gl::GetIntegerv(gl::MINOR_VERSION, &mut gl_minor as *mut _);
    }
    if gl_major < 4 || gl_minor < 5 {
        anyhow::bail!(
            "[FATAL ERROR] OpenGL {}.{} is loaded, but OpenGL 4.5 is needed.",
            gl_major,
            gl_minor
        );
    } else {
        println!(
            "OpenGL {}.{} is loaded, OpenGL 4.5 is needed.",
            gl_major, gl_minor
        );
    }

    let size_of_scene_uniform = std::mem::size_of::<uniforms::SceneUniform>() as f64 / 1048576.0;
    if size_of_scene_uniform >= 128.0 {
        anyhow::bail!("[FATAL ERROR] Size of 'SceneUniform' is too large (more than 128 MB)");
    } else {
        println!(
            "Size of 'SceneUniform' is {} MB, it is ok.",
            size_of_scene_uniform
        );
    }

    window.set_key_polling(true);
    window.make_current();

    renderer.init();

    while !window.should_close() {
        glfw.poll_events();
        window.swap_buffers();

        renderer.render();

        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
                    window.set_should_close(true);
                }
                _ => {}
            }
        }
    }

    Ok(())
}
