#![allow(clippy::uninlined_format_args)]

extern crate bindgen;

use cmake::Config;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

fn main() {
    let target = env::var("TARGET").unwrap();
    // Link C++ standard library
    if let Some(cpp_stdlib) = get_cpp_link_stdlib(&target) {
        println!("cargo:rustc-link-lib=dylib={}", cpp_stdlib);
    }
    // Link macOS Accelerate framework for matrix calculations
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=framework=Accelerate");
        #[cfg(feature = "coreml")]
        {
            println!("cargo:rustc-link-lib=framework=Foundation");
            println!("cargo:rustc-link-lib=framework=CoreML");
        }
        #[cfg(feature = "metal")]
        {
            println!("cargo:rustc-link-lib=framework=Foundation");
            println!("cargo:rustc-link-lib=framework=Metal");
            println!("cargo:rustc-link-lib=framework=MetalKit");
        }
    }

    #[cfg(feature = "coreml")]
    println!("cargo:rustc-link-lib=static=whisper.coreml");

    #[cfg(feature = "openblas")]
    {
        if let Ok(openblas_path) = env::var("OPENBLAS_PATH") {
            println!(
                "cargo::rustc-link-search={}",
                PathBuf::from(openblas_path).join("lib").display()
            );
        }
        if cfg!(windows) {
            println!("cargo:rustc-link-lib=libopenblas");
        } else {
            println!("cargo:rustc-link-lib=openblas");
        }
    }
    #[cfg(feature = "cuda")]
    {
        println!("cargo:rustc-link-lib=cublas");
        println!("cargo:rustc-link-lib=cudart");
        println!("cargo:rustc-link-lib=cublasLt");
        println!("cargo:rustc-link-lib=cuda");
        cfg_if::cfg_if! {
            if #[cfg(target_os = "windows")] {
                let cuda_path = PathBuf::from(env::var("CUDA_PATH").unwrap()).join("lib/x64");
                println!("cargo:rustc-link-search={}", cuda_path.display());
            } else {
                println!("cargo:rustc-link-lib=culibos");
                println!("cargo:rustc-link-search=/usr/local/cuda/lib64");
                println!("cargo:rustc-link-search=/usr/local/cuda/lib64/stubs");
                println!("cargo:rustc-link-search=/opt/cuda/lib64");
                println!("cargo:rustc-link-search=/opt/cuda/lib64/stubs");
            }
        }
    }
    #[cfg(feature = "hipblas")]
    {
        println!("cargo:rustc-link-lib=hipblas");
        println!("cargo:rustc-link-lib=rocblas");
        println!("cargo:rustc-link-lib=amdhip64");

        cfg_if::cfg_if! {
            if #[cfg(target_os = "windows")] {
                panic!("Due to a problem with the last revision of the ROCm 5.7 library, it is not possible to compile the library for the windows environment.\nSee https://github.com/ggerganov/whisper.cpp/issues/2202 for more details.")
            } else {
                println!("cargo:rerun-if-env-changed=HIP_PATH");

                let hip_path = match env::var("HIP_PATH") {
                    Ok(path) =>PathBuf::from(path),
                    Err(_) => PathBuf::from("/opt/rocm"),
                };
                let hip_lib_path = hip_path.join("lib");

                println!("cargo:rustc-link-search={}",hip_lib_path.display());
            }
        }
    }

    #[cfg(feature = "openmp")]
    {
        if target.contains("gnu") {
            println!("cargo:rustc-link-lib=gomp");
        } else if target.contains("apple") {
            println!("cargo:rustc-link-lib=omp");
            println!("cargo:rustc-link-search=/opt/homebrew/opt/libomp/lib");
        }
    }

    println!("cargo:rerun-if-changed=wrapper.h");

    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let whisper_root = out.join("whisper.cpp/");

    if !whisper_root.exists() {
        std::fs::create_dir_all(&whisper_root).unwrap();
        fs_extra::dir::copy("./whisper.cpp", &out, &Default::default()).unwrap_or_else(|e| {
            panic!(
                "Failed to copy whisper sources into {}: {}",
                whisper_root.display(),
                e
            )
        });
    }

    if env::var("WHISPER_DONT_GENERATE_BINDINGS").is_ok() {
        let bindings_path = out.join("bindings.rs");
        let _: u64 =
            std::fs::copy("src/bindings.rs", &bindings_path).expect("Failed to copy bindings.rs");
        patch_generated_bindings(&bindings_path).expect("Unable to patch bindings.rs");
    } else {
        let bindings = bindgen::Builder::default().header("wrapper.h");

        #[cfg(feature = "metal")]
        let bindings = bindings.header("whisper.cpp/ggml/include/ggml-metal.h");

        let bindings = bindings
            .clang_arg("-I./whisper.cpp/")
            .clang_arg("-I./whisper.cpp/include")
            .clang_arg("-I./whisper.cpp/ggml/include")
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .generate();

        match bindings {
            Ok(b) => {
                let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
                let bindings_path = out_path.join("bindings.rs");
                b.write_to_file(&bindings_path)
                    .expect("Couldn't write bindings!");
                patch_generated_bindings(&bindings_path).expect("Unable to patch bindings.rs");
            }
            Err(e) => {
                println!("cargo:warning=Unable to generate bindings: {}", e);
                println!("cargo:warning=Using bundled bindings.rs, which may be out of date");
                // copy src/bindings.rs to OUT_DIR
                let bindings_path = out.join("bindings.rs");
                std::fs::copy("src/bindings.rs", &bindings_path)
                    .expect("Unable to copy bindings.rs");
                patch_generated_bindings(&bindings_path).expect("Unable to patch bindings.rs");
            }
        }
    };

    // stop if we're on docs.rs
    if env::var("DOCS_RS").is_ok() {
        return;
    }

    let mut config = Config::new(&whisper_root);

    config
        .profile("Release")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("WHISPER_ALL_WARNINGS", "OFF")
        .define("WHISPER_ALL_WARNINGS_3RD_PARTY", "OFF")
        .define("WHISPER_BUILD_TESTS", "OFF")
        .define("WHISPER_BUILD_EXAMPLES", "OFF")
        .very_verbose(true)
        .pic(true);

    if cfg!(feature = "coreml") {
        config.define("WHISPER_COREML", "ON");
        config.define("WHISPER_COREML_ALLOW_FALLBACK", "1");
    }

    if cfg!(feature = "cuda") {
        config.define("GGML_CUDA", "ON");
    }

    if cfg!(feature = "hipblas") {
        config.define("GGML_HIPBLAS", "ON");
        config.define("CMAKE_C_COMPILER", "hipcc");
        config.define("CMAKE_CXX_COMPILER", "hipcc");
        println!("cargo:rerun-if-env-changed=AMDGPU_TARGETS");
        if let Ok(gpu_targets) = env::var("AMDGPU_TARGETS") {
            config.define("AMDGPU_TARGETS", gpu_targets);
        }
    }

    if cfg!(feature = "vulkan") {
        config.define("GGML_VULKAN", "ON");
        if cfg!(windows) {
            println!("cargo:rerun-if-env-changed=VULKAN_SDK");
            println!("cargo:rustc-link-lib=vulkan-1");
            let vulkan_path = match env::var("VULKAN_SDK") {
                Ok(path) => PathBuf::from(path),
                Err(_) => panic!(
                    "Please install Vulkan SDK and ensure that VULKAN_SDK env variable is set"
                ),
            };
            let vulkan_lib_path = vulkan_path.join("Lib");
            println!("cargo:rustc-link-search={}", vulkan_lib_path.display());
        } else if cfg!(target_os = "macos") {
            println!("cargo:rerun-if-env-changed=VULKAN_SDK");
            println!("cargo:rustc-link-lib=vulkan");
            let vulkan_path = match env::var("VULKAN_SDK") {
                Ok(path) => PathBuf::from(path),
                Err(_) => panic!(
                    "Please install Vulkan SDK and ensure that VULKAN_SDK env variable is set"
                ),
            };
            let vulkan_lib_path = vulkan_path.join("lib");
            println!("cargo:rustc-link-search={}", vulkan_lib_path.display());
        } else {
            println!("cargo:rustc-link-lib=vulkan");
        }
    }

    if cfg!(feature = "openblas") {
        config.define("GGML_BLAS", "ON");
    }

    if cfg!(feature = "metal") {
        config.define("GGML_METAL", "ON");
        config.define("GGML_METAL_NDEBUG", "ON");
        config.define("GGML_METAL_EMBED_LIBRARY", "ON");
    } else {
        // Metal is enabled by default, so we need to explicitly disable it
        config.define("GGML_METAL", "OFF");
    }

    if cfg!(debug_assertions) || cfg!(feature = "force-debug") {
        // debug builds are too slow to even remotely be usable,
        // so we build with optimizations even in debug mode
        config.define("CMAKE_BUILD_TYPE", "RelWithDebInfo");
        config.cxxflag("-DWHISPER_DEBUG");
    }

    // Allow passing any WHISPER or CMAKE compile flags
    for (key, value) in env::vars() {
        let is_whisper_flag =
            key.starts_with("WHISPER_") && key != "WHISPER_DONT_GENERATE_BINDINGS";
        let is_cmake_flag = key.starts_with("CMAKE_");
        if is_whisper_flag || is_cmake_flag {
            config.define(&key, &value);
        }
    }

    if cfg!(not(feature = "openmp")) {
        config.define("GGML_OPENMP", "OFF");
    }

    let destination = config.build();

    add_link_search_path(&out.join("build")).unwrap();

    println!("cargo:rustc-link-search=native={}", destination.display());
    println!("cargo:rustc-link-lib=static=whisper");
    println!("cargo:rustc-link-lib=static=ggml");

    println!(
        "cargo:WHISPER_CPP_VERSION={}",
        get_whisper_cpp_version(&whisper_root)
            .expect("Failed to read whisper.cpp CMake config")
            .expect("Could not find whisper.cpp version declaration"),
    );

    // for whatever reason this file is generated during build and triggers cargo complaining
    _ = std::fs::remove_file("bindings/javascript/package.json");
}

// From https://github.com/alexcrichton/cc-rs/blob/fba7feded71ee4f63cfe885673ead6d7b4f2f454/src/lib.rs#L2462
fn get_cpp_link_stdlib(target: &str) -> Option<&'static str> {
    if target.contains("msvc") {
        None
    } else if target.contains("apple") || target.contains("freebsd") || target.contains("openbsd") {
        Some("c++")
    } else if target.contains("android") {
        Some("c++_shared")
    } else {
        Some("stdc++")
    }
}

fn add_link_search_path(dir: &std::path::Path) -> std::io::Result<()> {
    if dir.is_dir() {
        println!("cargo:rustc-link-search={}", dir.display());
        for entry in std::fs::read_dir(dir)? {
            add_link_search_path(&entry?.path())?;
        }
    }
    Ok(())
}

fn patch_generated_bindings(path: &Path) -> std::io::Result<()> {
    let bindings = std::fs::read_to_string(path)?;
    let patched = patch_opaque_whisper_full_params(bindings);
    std::fs::write(path, patched)
}

fn patch_opaque_whisper_full_params(bindings: String) -> String {
    let line_ending = if bindings.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let start_marker = format!(
        "#[repr(C)]{line_ending}#[derive(Debug, Copy, Clone)]{line_ending}pub struct whisper_full_params {{{line_ending}    pub _address: u8,{line_ending}}}{line_ending}"
    );
    let Some(start) = bindings.find(&start_marker) else {
        return bindings;
    };

    let end_marker =
        format!("extern \"C\" {{{line_ending}    pub fn whisper_context_default_params_by_ref");
    let Some(relative_end) = bindings[start..].find(&end_marker) else {
        return bindings;
    };
    let end = start + relative_end;

    let mut patched = String::with_capacity(bindings.len() + WHISPER_FULL_PARAMS_PATCH.len());
    patched.push_str(&bindings[..start]);
    patched.push_str(&WHISPER_FULL_PARAMS_PATCH.replace('\n', line_ending));
    patched.push_str(&bindings[end..]);
    patched
}

const WHISPER_FULL_PARAMS_PATCH: &str = r#"#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct whisper_full_params {
    pub strategy: whisper_sampling_strategy,
    pub n_threads: ::std::os::raw::c_int,
    pub n_max_text_ctx: ::std::os::raw::c_int,
    pub offset_ms: ::std::os::raw::c_int,
    pub duration_ms: ::std::os::raw::c_int,
    pub translate: bool,
    pub no_context: bool,
    pub no_timestamps: bool,
    pub single_segment: bool,
    pub print_special: bool,
    pub print_progress: bool,
    pub print_realtime: bool,
    pub print_timestamps: bool,
    pub token_timestamps: bool,
    pub thold_pt: f32,
    pub thold_ptsum: f32,
    pub max_len: ::std::os::raw::c_int,
    pub split_on_word: bool,
    pub max_tokens: ::std::os::raw::c_int,
    pub debug_mode: bool,
    pub audio_ctx: ::std::os::raw::c_int,
    pub tdrz_enable: bool,
    pub suppress_regex: *const ::std::os::raw::c_char,
    pub initial_prompt: *const ::std::os::raw::c_char,
    pub prompt_tokens: *const whisper_token,
    pub prompt_n_tokens: ::std::os::raw::c_int,
    pub language: *const ::std::os::raw::c_char,
    pub detect_language: bool,
    pub suppress_blank: bool,
    pub suppress_non_speech_tokens: bool,
    pub temperature: f32,
    pub max_initial_ts: f32,
    pub length_penalty: f32,
    pub temperature_inc: f32,
    pub entropy_thold: f32,
    pub logprob_thold: f32,
    pub no_speech_thold: f32,
    pub greedy: whisper_full_params__bindgen_ty_1,
    pub beam_search: whisper_full_params__bindgen_ty_2,
    pub new_segment_callback: whisper_new_segment_callback,
    pub new_segment_callback_user_data: *mut ::std::os::raw::c_void,
    pub progress_callback: whisper_progress_callback,
    pub progress_callback_user_data: *mut ::std::os::raw::c_void,
    pub encoder_begin_callback: whisper_encoder_begin_callback,
    pub encoder_begin_callback_user_data: *mut ::std::os::raw::c_void,
    pub abort_callback: ggml_abort_callback,
    pub abort_callback_user_data: *mut ::std::os::raw::c_void,
    pub logits_filter_callback: whisper_logits_filter_callback,
    pub logits_filter_callback_user_data: *mut ::std::os::raw::c_void,
    pub grammar_rules: *mut *const whisper_grammar_element,
    pub n_grammar_rules: usize,
    pub i_start_rule: usize,
    pub grammar_penalty: f32,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct whisper_full_params__bindgen_ty_1 {
    pub best_of: ::std::os::raw::c_int,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct whisper_full_params__bindgen_ty_2 {
    pub beam_size: ::std::os::raw::c_int,
    pub patience: f32,
}
"#;

fn get_whisper_cpp_version(whisper_root: &std::path::Path) -> std::io::Result<Option<String>> {
    let cmake_lists = BufReader::new(File::open(whisper_root.join("CMakeLists.txt"))?);

    for line in cmake_lists.lines() {
        let line = line?;

        if let Some(suffix) = line.strip_prefix(r#"project("whisper.cpp" VERSION "#) {
            let whisper_cpp_version = suffix.trim_end_matches(')');
            return Ok(Some(whisper_cpp_version.into()));
        }
    }

    Ok(None)
}
