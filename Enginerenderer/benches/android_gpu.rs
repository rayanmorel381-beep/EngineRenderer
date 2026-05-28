//! Zero-dependency Android GPU micro-benchmarks.
//!
//! Run with:
//!   benches/run_android_bench.sh
//!
//! On any non-Android target the file compiles to a stub `main` so that
//! `cargo bench --bench android_gpu` on the desktop is a no-op.

#[cfg(not(target_os = "android"))]
fn main() {
    eprintln!("android_gpu bench is only available on target_os=\"android\"");
}

#[cfg(target_os = "android")]
fn main() {
    android::run();
}

#[cfg(target_os = "android")]
mod android {
    use core::ffi::{c_char, c_uint, c_void};
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use enginerenderer::api::display::NativeWindow;

    const WARMUP_ITERS: u32 = 32;
    const MEASURE_ITERS: u32 = 256;

    const MEMORY_BOUND_SRC: &[u8] = b"#version 310 es\nprecision highp float;\nlayout(local_size_x = 64) in;\nlayout(std430, binding = 0) buffer Data { float v[]; };\nvoid main() {\n    uint id = gl_GlobalInvocationID.x;\n    if (id < uint(v.length())) {\n        v[id] = v[id] * 2.0 + 1.0;\n    }\n}\n\0";

    const MANDELBROT_SRC: &[u8] = b"#version 310 es\nprecision highp float;\nlayout(local_size_x = 64) in;\nlayout(std430, binding = 0) buffer Data { float v[]; };\nvoid main() {\n    uint id = gl_GlobalInvocationID.x;\n    if (id >= uint(v.length())) return;\n    float seed = v[id];\n    float cx = -0.75 + (seed - 0.5) * 0.001;\n    float cy = 0.1 + (seed - 0.5) * 0.001;\n    float zx = 0.0;\n    float zy = 0.0;\n    int iter = 0;\n    for (int i = 0; i < 512; ++i) {\n        float zx2 = zx * zx - zy * zy + cx;\n        float zy2 = 2.0 * zx * zy + cy;\n        zx = zx2;\n        zy = zy2;\n        if (zx * zx + zy * zy > 4.0) break;\n        iter = i;\n    }\n    v[id] = float(iter) * 0.001;\n}\n\0";

    const MANDELBROT_SRC_16: &[u8] = b"#version 310 es\nprecision highp float;\nlayout(local_size_x = 128) in;\nlayout(std430, binding = 0) buffer Data { float v[]; };\nvoid main() {\n    uint id = gl_GlobalInvocationID.x;\n    if (id >= uint(v.length())) return;\n    float seed = v[id];\n    float cx = -0.75 + (seed - 0.5) * 0.001;\n    float cy = 0.1 + (seed - 0.5) * 0.001;\n    float zx = 0.0;\n    float zy = 0.0;\n    int iter = 0;\n    for (int i = 0; i < 512; ++i) {\n        float zx2 = zx * zx - zy * zy + cx;\n        float zy2 = 2.0 * zx * zy + cy;\n        zx = zx2;\n        zy = zy2;\n        if (zx * zx + zy * zy > 4.0) break;\n        iter = i;\n    }\n    v[id] = float(iter) * 0.001;\n}\n\0";

    const GL_COLOR_BUFFER_BIT: c_uint = 0x0000_4000;
    const GL_DEPTH_BUFFER_BIT: c_uint = 0x0000_0100;
    const GL_VENDOR: c_uint = 0x1F00;
    const GL_RENDERER: c_uint = 0x1F01;
    const GL_VERSION: c_uint = 0x1F02;

    const GL_COMPUTE_SHADER: c_uint = 0x91B9;
    const GL_SHADER_STORAGE_BUFFER: c_uint = 0x90D2;
    const GL_DYNAMIC_COPY: c_uint = 0x88EA;
    const GL_SHADER_STORAGE_BARRIER_BIT: c_uint = 0x0000_2000;
    const GL_COMPILE_STATUS: c_uint = 0x8B81;
    const GL_LINK_STATUS: c_uint = 0x8B82;
    const GL_INFO_LOG_LENGTH: c_uint = 0x8B84;

    type GlClearColor = unsafe extern "C" fn(f32, f32, f32, f32);
    type GlClear = unsafe extern "C" fn(c_uint);
    type GlGetString = unsafe extern "C" fn(c_uint) -> *const c_char;
    type GlGetError = unsafe extern "C" fn() -> c_uint;
    type GlViewport = unsafe extern "C" fn(i32, i32, i32, i32);
    type GlFinish = unsafe extern "C" fn();

    type GlCreateShader = unsafe extern "C" fn(c_uint) -> c_uint;
    type GlShaderSource = unsafe extern "C" fn(c_uint, i32, *const *const c_char, *const i32);
    type GlCompileShader = unsafe extern "C" fn(c_uint);
    type GlGetShaderiv = unsafe extern "C" fn(c_uint, c_uint, *mut i32);
    type GlGetShaderInfoLog = unsafe extern "C" fn(c_uint, i32, *mut i32, *mut c_char);
    type GlCreateProgram = unsafe extern "C" fn() -> c_uint;
    type GlAttachShader = unsafe extern "C" fn(c_uint, c_uint);
    type GlLinkProgram = unsafe extern "C" fn(c_uint);
    type GlGetProgramiv = unsafe extern "C" fn(c_uint, c_uint, *mut i32);
    type GlUseProgram = unsafe extern "C" fn(c_uint);
    type GlGenBuffers = unsafe extern "C" fn(i32, *mut c_uint);
    type GlBindBuffer = unsafe extern "C" fn(c_uint, c_uint);
    type GlBufferData = unsafe extern "C" fn(c_uint, isize, *const c_void, c_uint);
    type GlBindBufferBase = unsafe extern "C" fn(c_uint, c_uint, c_uint);
    type GlDispatchCompute = unsafe extern "C" fn(c_uint, c_uint, c_uint);
    type GlMemoryBarrier = unsafe extern "C" fn(c_uint);
    type GlDeleteBuffers = unsafe extern "C" fn(i32, *const c_uint);
    type GlDeleteProgram = unsafe extern "C" fn(c_uint);
    type GlDeleteShader = unsafe extern "C" fn(c_uint);
    type GlMapBufferRange = unsafe extern "C" fn(c_uint, isize, isize, c_uint) -> *mut c_void;
    type GlUnmapBuffer = unsafe extern "C" fn(c_uint) -> u8;

    unsafe fn load<T: Copy>(window: &NativeWindow, name: &[u8]) -> T {
        let p = window.gl_get_proc(name);
        assert!(
            !p.is_null(),
            "GL function {:?} not found",
            core::str::from_utf8(name).unwrap_or("?")
        );
        assert_eq!(
            core::mem::size_of::<T>(),
            core::mem::size_of::<*mut c_void>(),
            "function pointer size mismatch"
        );
        unsafe { core::mem::transmute_copy(&p) }
    }

    fn read_cstring(p: *const c_char) -> String {
        if p.is_null() {
            return String::new();
        }
        let mut len = 0usize;
        while unsafe { *p.add(len) } != 0 {
            len += 1;
            if len > 4096 {
                break;
            }
        }
        let bytes = unsafe { core::slice::from_raw_parts(p as *const u8, len) };
        String::from_utf8_lossy(bytes).into_owned()
    }

    struct Sample {
        name: &'static str,
        items: u64,
        timings_ns: Vec<u128>,
    }

    impl Sample {
        fn report(&self) {
            let mut sorted = self.timings_ns.clone();
            sorted.sort_unstable();
            let n = sorted.len();
            let min = sorted[0];
            let max = sorted[n - 1];
            let median = sorted[n / 2];
            let sum: u128 = sorted.iter().sum();
            let mean = sum / n as u128;
            let throughput = if median > 0 {
                (self.items as f64) * 1_000_000_000.0 / median as f64
            } else {
                f64::INFINITY
            };
            println!(
                "{:<48} median={:>10}ns  mean={:>10}ns  min={:>10}ns  max={:>10}ns  ~{:>12.0} items/s",
                self.name, median, mean, min, max, throughput
            );
        }
    }

    fn bench<F: FnMut()>(name: &'static str, items: u64, mut body: F) -> Sample {
        for _ in 0..WARMUP_ITERS {
            body();
        }
        let mut timings = Vec::with_capacity(MEASURE_ITERS as usize);
        for _ in 0..MEASURE_ITERS {
            let start = Instant::now();
            body();
            timings.push(start.elapsed().as_nanos());
        }
        Sample {
            name,
            items,
            timings_ns: timings,
        }
    }

    fn bench_native_window_open_drop() -> Sample {
        bench("native_window_open_drop_64x64", 1, || {
            let w = NativeWindow::open(64, 64, "bench").expect("open");
            black_box(&w);
            drop(w);
        })
    }

    fn bench_make_current(window: &NativeWindow) -> Sample {
        bench("egl_make_current", 1, || {
            let ok = window.make_current();
            black_box(ok);
        })
    }

    fn bench_gl_get_proc(window: &NativeWindow) -> Sample {
        let names: [&[u8]; 8] = [
            b"glClear\0",
            b"glClearColor\0",
            b"glGetString\0",
            b"glGetError\0",
            b"glViewport\0",
            b"glFinish\0",
            b"glEnable\0",
            b"glDisable\0",
        ];
        bench("gl_get_proc_lookup_8_symbols", names.len() as u64, || {
            for n in names.iter() {
                let p = window.gl_get_proc(black_box(n));
                black_box(p);
            }
        })
    }

    fn bench_gl_get_string(window: &NativeWindow) -> Sample {
        let gl_get_string: GlGetString = unsafe { load(window, b"glGetString\0") };
        bench("gl_get_string_vendor_renderer_version", 3, || unsafe {
            black_box(gl_get_string(GL_VENDOR));
            black_box(gl_get_string(GL_RENDERER));
            black_box(gl_get_string(GL_VERSION));
        })
    }

    fn bench_gl_get_error(window: &NativeWindow) -> Sample {
        let gl_get_error: GlGetError = unsafe { load(window, b"glGetError\0") };
        bench("gl_get_error", 1, || unsafe {
            black_box(gl_get_error());
        })
    }

    fn bench_gl_clear(window: &NativeWindow) -> Sample {
        let gl_clear_color: GlClearColor = unsafe { load(window, b"glClearColor\0") };
        let gl_clear: GlClear = unsafe { load(window, b"glClear\0") };
        let gl_finish: GlFinish = unsafe { load(window, b"glFinish\0") };
        bench("gl_clear_color_buffer", 1, || unsafe {
            gl_clear_color(0.1, 0.2, 0.3, 1.0);
            gl_clear(GL_COLOR_BUFFER_BIT);
            gl_finish();
        })
    }

    fn bench_gl_clear_color_depth(window: &NativeWindow) -> Sample {
        let gl_clear_color: GlClearColor = unsafe { load(window, b"glClearColor\0") };
        let gl_clear: GlClear = unsafe { load(window, b"glClear\0") };
        let gl_finish: GlFinish = unsafe { load(window, b"glFinish\0") };
        bench("gl_clear_color_and_depth", 2, || unsafe {
            gl_clear_color(0.0, 0.0, 0.0, 1.0);
            gl_clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
            gl_finish();
        })
    }

    fn bench_swap_buffers(window: &NativeWindow) -> Sample {
        bench("egl_swap_buffers_pbuffer", 1, || {
            window.swap_buffers();
        })
    }

    fn bench_full_frame(window: &NativeWindow) -> Sample {
        let gl_viewport: GlViewport = unsafe { load(window, b"glViewport\0") };
        let gl_clear_color: GlClearColor = unsafe { load(window, b"glClearColor\0") };
        let gl_clear: GlClear = unsafe { load(window, b"glClear\0") };
        let w = window.width as i32;
        let h = window.height as i32;
        bench("full_frame_viewport_clear_swap", 1, || unsafe {
            gl_viewport(0, 0, w, h);
            gl_clear_color(0.25, 0.5, 0.75, 1.0);
            gl_clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
            window.swap_buffers();
        })
    }

    struct ComputeProgram {
        program: c_uint,
        shader: c_uint,
        ssbo: c_uint,
        elements: u32,
        local_size_x: u32,
        gl_use_program: GlUseProgram,
        gl_bind_buffer: GlBindBuffer,
        gl_bind_buffer_base: GlBindBufferBase,
        gl_dispatch_compute: GlDispatchCompute,
        gl_memory_barrier: GlMemoryBarrier,
        gl_finish: GlFinish,
        gl_delete_buffers: GlDeleteBuffers,
        gl_delete_program: GlDeleteProgram,
        gl_delete_shader: GlDeleteShader,
    }

    impl Drop for ComputeProgram {
        fn drop(&mut self) {
            unsafe {
                (self.gl_delete_buffers)(1, &self.ssbo);
                (self.gl_delete_program)(self.program);
                (self.gl_delete_shader)(self.shader);
            }
        }
    }

    fn build_compute_program(
        window: &NativeWindow,
        elements: u32,
        src: &[u8],
        local_size_x: u32,
    ) -> Option<ComputeProgram> {
        let gl_create_shader: GlCreateShader = unsafe { load(window, b"glCreateShader\0") };
        let gl_shader_source: GlShaderSource = unsafe { load(window, b"glShaderSource\0") };
        let gl_compile_shader: GlCompileShader = unsafe { load(window, b"glCompileShader\0") };
        let gl_get_shader_iv: GlGetShaderiv = unsafe { load(window, b"glGetShaderiv\0") };
        let gl_get_shader_info_log: GlGetShaderInfoLog =
            unsafe { load(window, b"glGetShaderInfoLog\0") };
        let gl_create_program: GlCreateProgram = unsafe { load(window, b"glCreateProgram\0") };
        let gl_attach_shader: GlAttachShader = unsafe { load(window, b"glAttachShader\0") };
        let gl_link_program: GlLinkProgram = unsafe { load(window, b"glLinkProgram\0") };
        let gl_get_program_iv: GlGetProgramiv = unsafe { load(window, b"glGetProgramiv\0") };
        let gl_use_program: GlUseProgram = unsafe { load(window, b"glUseProgram\0") };
        let gl_gen_buffers: GlGenBuffers = unsafe { load(window, b"glGenBuffers\0") };
        let gl_bind_buffer: GlBindBuffer = unsafe { load(window, b"glBindBuffer\0") };
        let gl_buffer_data: GlBufferData = unsafe { load(window, b"glBufferData\0") };
        let gl_bind_buffer_base: GlBindBufferBase = unsafe { load(window, b"glBindBufferBase\0") };
        let gl_dispatch_compute: GlDispatchCompute =
            unsafe { load(window, b"glDispatchCompute\0") };
        let gl_memory_barrier: GlMemoryBarrier = unsafe { load(window, b"glMemoryBarrier\0") };
        let gl_finish: GlFinish = unsafe { load(window, b"glFinish\0") };
        let gl_delete_buffers: GlDeleteBuffers = unsafe { load(window, b"glDeleteBuffers\0") };
        let gl_delete_program: GlDeleteProgram = unsafe { load(window, b"glDeleteProgram\0") };
        let gl_delete_shader: GlDeleteShader = unsafe { load(window, b"glDeleteShader\0") };

        let local_size_x: u32 = local_size_x;
        let _ = local_size_x;
        let shader = unsafe { gl_create_shader(GL_COMPUTE_SHADER) };
        if shader == 0 {
            return None;
        }
        let src_ptr: *const c_char = src.as_ptr() as *const c_char;
        let src_len: i32 = (src.len() - 1) as i32;
        unsafe {
            gl_shader_source(shader, 1, &src_ptr, &src_len);
            gl_compile_shader(shader);
        }
        let mut status: i32 = 0;
        unsafe { gl_get_shader_iv(shader, GL_COMPILE_STATUS, &mut status) };
        if status == 0 {
            let mut log_len: i32 = 0;
            unsafe { gl_get_shader_iv(shader, GL_INFO_LOG_LENGTH, &mut log_len) };
            if log_len > 0 {
                let mut log = vec![0u8; log_len as usize];
                let mut written: i32 = 0;
                unsafe {
                    gl_get_shader_info_log(
                        shader,
                        log_len,
                        &mut written,
                        log.as_mut_ptr() as *mut c_char,
                    );
                }
                let log_str = String::from_utf8_lossy(&log[..written.max(0) as usize]);
                eprintln!("compute shader compile failed: {log_str}");
            }
            return None;
        }

        let program = unsafe { gl_create_program() };
        unsafe {
            gl_attach_shader(program, shader);
            gl_link_program(program);
        }
        let mut link_status: i32 = 0;
        unsafe { gl_get_program_iv(program, GL_LINK_STATUS, &mut link_status) };
        if link_status == 0 {
            return None;
        }

        let initial: Vec<f32> = (0..elements).map(|i| i as f32 * 0.001).collect();
        let mut ssbo: c_uint = 0;
        unsafe {
            gl_gen_buffers(1, &mut ssbo);
            gl_bind_buffer(GL_SHADER_STORAGE_BUFFER, ssbo);
            gl_buffer_data(
                GL_SHADER_STORAGE_BUFFER,
                (initial.len() * core::mem::size_of::<f32>()) as isize,
                initial.as_ptr() as *const c_void,
                GL_DYNAMIC_COPY,
            );
            gl_bind_buffer_base(GL_SHADER_STORAGE_BUFFER, 0, ssbo);
        }

        Some(ComputeProgram {
            program,
            shader,
            ssbo,
            elements,
            local_size_x,
            gl_use_program,
            gl_bind_buffer,
            gl_bind_buffer_base,
            gl_dispatch_compute,
            gl_memory_barrier,
            gl_finish,
            gl_delete_buffers,
            gl_delete_program,
            gl_delete_shader,
        })
    }

    const GL_MAP_READ_BIT: c_uint = 0x0001;

    fn verify_gpu_executed(window: &NativeWindow, prog: &ComputeProgram) {
        let gl_map_buffer_range: GlMapBufferRange = unsafe { load(window, b"glMapBufferRange\0") };
        let gl_unmap_buffer: GlUnmapBuffer = unsafe { load(window, b"glUnmapBuffer\0") };

        let groups = prog.elements.div_ceil(prog.local_size_x);
        unsafe {
            (prog.gl_use_program)(prog.program);
            (prog.gl_bind_buffer)(GL_SHADER_STORAGE_BUFFER, prog.ssbo);
            (prog.gl_bind_buffer_base)(GL_SHADER_STORAGE_BUFFER, 0, prog.ssbo);
            (prog.gl_dispatch_compute)(groups, 1, 1);
            (prog.gl_memory_barrier)(GL_SHADER_STORAGE_BARRIER_BIT);
            (prog.gl_finish)();
        }

        let total_bytes = (prog.elements as usize * core::mem::size_of::<f32>()) as isize;
        let ptr = unsafe {
            gl_map_buffer_range(GL_SHADER_STORAGE_BUFFER, 0, total_bytes, GL_MAP_READ_BIT)
        };
        if ptr.is_null() {
            println!("-- GPU execution verification --");
            println!(
                "  glMapBufferRange returned NULL; cannot verify (driver may have rejected map)"
            );
            println!();
            return;
        }
        let slice =
            unsafe { core::slice::from_raw_parts(ptr as *const f32, prog.elements as usize) };

        let sample_count: usize = 8;
        let stride = (prog.elements as usize / sample_count).max(1);

        println!("-- GPU execution verification --");
        println!(
            "fresh SSBO of {} floats initialised CPU-side as v[i] = i * 0.001, then 1 compute dispatch of v[i] = v[i] * 2 + 1",
            prog.elements
        );
        let mut all_ok = true;
        for k in 0..sample_count {
            let i = k * stride;
            let initial = i as f32 * 0.001;
            let expected = initial * 2.0 + 1.0;
            let got = slice[i];
            let abs_err = (got - expected).abs();
            let ok = abs_err < 1e-4;
            if !ok {
                all_ok = false;
            }
            println!(
                "  v[{:>4}]  cpu_initial={:>10.6}  expected={:>10.6}  gpu_returned={:>10.6}  abs_err={:>10.2e}  {}",
                i,
                initial,
                expected,
                got,
                abs_err,
                if ok { "OK" } else { "MISMATCH" }
            );
        }
        unsafe {
            gl_unmap_buffer(GL_SHADER_STORAGE_BUFFER);
        }
        if all_ok {
            println!(
                "  >> GPU EXECUTION CONFIRMED: SSBO contains values transformed by the compute shader running on the GPU."
            );
        } else {
            println!("  >> GPU EXECUTION FAILED: SSBO does not match expected values.");
        }
        println!();
    }

    fn bench_compute_dispatch(prog: &ComputeProgram, label: &'static str) -> Sample {
        let groups = prog.elements.div_ceil(prog.local_size_x);
        let elements = prog.elements as u64;
        let program = prog.program;
        let ssbo = prog.ssbo;
        let gl_use_program = prog.gl_use_program;
        let gl_bind_buffer = prog.gl_bind_buffer;
        let gl_bind_buffer_base = prog.gl_bind_buffer_base;
        let gl_dispatch_compute = prog.gl_dispatch_compute;
        let gl_memory_barrier = prog.gl_memory_barrier;
        let gl_finish = prog.gl_finish;
        bench(label, elements, || unsafe {
            gl_use_program(program);
            gl_bind_buffer(GL_SHADER_STORAGE_BUFFER, ssbo);
            gl_bind_buffer_base(GL_SHADER_STORAGE_BUFFER, 0, ssbo);
            gl_dispatch_compute(groups, 1, 1);
            gl_memory_barrier(GL_SHADER_STORAGE_BARRIER_BIT);
            gl_finish();
        })
    }

    pub fn run() {
        println!("== enginerenderer android GPU micro-benchmarks ==");
        println!("warmup={WARMUP_ITERS}  measure={MEASURE_ITERS}");
        println!();

        let bootstrap = NativeWindow::open(256, 256, "bench-bootstrap").expect("open bootstrap");
        assert!(bootstrap.has_backend(), "EGL backend must be attached");
        assert!(bootstrap.make_current(), "eglMakeCurrent must succeed");
        let gl_get_string: GlGetString = unsafe { load(&bootstrap, b"glGetString\0") };
        let vendor = read_cstring(unsafe { gl_get_string(GL_VENDOR) });
        let renderer = read_cstring(unsafe { gl_get_string(GL_RENDERER) });
        let version = read_cstring(unsafe { gl_get_string(GL_VERSION) });
        println!("device: vendor={vendor:?} renderer={renderer:?} version={version:?}");
        println!(
            "surface: {}x{}  vsync_capacity={}",
            bootstrap.width, bootstrap.height, bootstrap.vsync_capacity
        );
        println!();

        let mut samples: Vec<Sample> = Vec::new();
        samples.push(bench_native_window_open_drop());

        let window = NativeWindow::open(256, 256, "bench-main").expect("open main");
        assert!(window.make_current());

        samples.push(bench_make_current(&window));
        samples.push(bench_gl_get_proc(&window));
        samples.push(bench_gl_get_string(&window));
        samples.push(bench_gl_get_error(&window));
        samples.push(bench_gl_clear(&window));
        samples.push(bench_gl_clear_color_depth(&window));
        samples.push(bench_swap_buffers(&window));
        samples.push(bench_full_frame(&window));

        let compute_small = build_compute_program(&window, 64 * 1024, MEMORY_BOUND_SRC, 64);
        let compute_large = build_compute_program(&window, 1024 * 1024, MEMORY_BOUND_SRC, 64);
        let mandel_64k = build_compute_program(&window, 64 * 1024, MANDELBROT_SRC, 64);
        let mandel_1m = build_compute_program(&window, 1024 * 1024, MANDELBROT_SRC, 64);
        let mandel_64k_4x4 = build_compute_program(&window, 64 * 1024, MANDELBROT_SRC_16, 128);
        let mandel_1m_4x4 = build_compute_program(&window, 1024 * 1024, MANDELBROT_SRC_16, 128);
        if let Some(p) = compute_small.as_ref() {
            samples.push(bench_compute_dispatch(p, "gpu_compute_dispatch_64k_floats"));
        } else {
            eprintln!("compute_small program failed to build (skipped)");
        }
        if let Some(p) = compute_large.as_ref() {
            samples.push(bench_compute_dispatch(p, "gpu_compute_dispatch_1m_floats"));
        } else {
            eprintln!("compute_large program failed to build (skipped)");
        }
        if let Some(p) = mandel_64k.as_ref() {
            samples.push(bench_compute_dispatch(
                p,
                "gpu_mandelbrot_64k_x_512iter_lsx64",
            ));
        }
        if let Some(p) = mandel_1m.as_ref() {
            samples.push(bench_compute_dispatch(
                p,
                "gpu_mandelbrot_1m_x_512iter_lsx64",
            ));
        }
        if let Some(p) = mandel_64k_4x4.as_ref() {
            samples.push(bench_compute_dispatch(
                p,
                "gpu_mandelbrot_64k_x_512iter_lsx128",
            ));
        }
        if let Some(p) = mandel_1m_4x4.as_ref() {
            samples.push(bench_compute_dispatch(
                p,
                "gpu_mandelbrot_1m_x_512iter_lsx128",
            ));
        }

        if let Some(p) = build_compute_program(&window, 4096, MEMORY_BOUND_SRC, 64) {
            verify_gpu_executed(&window, &p);
        }

        for s in &samples {
            s.report();
        }

        let total_ns: u128 = samples
            .iter()
            .map(|s| s.timings_ns.iter().sum::<u128>())
            .sum();
        println!();
        println!(
            "total measurement wall-clock: {:.2}s",
            Duration::from_nanos(total_ns as u64).as_secs_f64()
        );

        drop(window);
        drop(bootstrap);
    }
}
