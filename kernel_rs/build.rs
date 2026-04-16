// Build script for AI-OS kernel
// Links llama.cpp static library and sets up FFI bindings

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=libllama.a");
    
    // Get the output directory
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    // Link llama.cpp static library
    let llama_lib = PathBuf::from("libllama.a");
    
    if llama_lib.exists() {
        println!("cargo:rustc-link-search=native=.");
        println!("cargo:rustc-link-lib=static=llama");
        
        // Link C++ standard library (required by llama.cpp)
        println!("cargo:rustc-link-lib=dylib=stdc++");
        
        // Link math library
        println!("cargo:rustc-link-lib=dylib=m");
        
        println!("cargo:warning=llama.cpp library found and linked");
    } else {
        println!("cargo:warning=libllama.a not found - AI features will use stubs");
        println!("cargo:warning=Run ./build_llama.sh to build llama.cpp");
    }
    
    // Generate bindings (optional - for now we use manual FFI)
    // If bindgen is needed:
    // let bindings = bindgen::Builder::default()
    //     .header("llama.cpp/llama.h")
    //     .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    //     .generate()
    //     .expect("Unable to generate bindings");
    //
    // let out_path = out_dir.join("llama_bindings.rs");
    // bindings.write_to_file(out_path).expect("Couldn't write bindings!");
}
