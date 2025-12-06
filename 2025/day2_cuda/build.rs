fn main() {
    println!("cargo:rerun-if-changed=cuda_src/gpu_proc.cu");

    cc::Build::new()
        .cuda(true)
        .ccbin(true)
        .cudart("shared")
        .flag("-t0")
        .file("cuda_src/gpu_proc.cu")
        .compile("libgpu_proc.a");

    println!("cargo:rustc-link-search=native=/opt/cuda/lib64");
    println!("cargo:rustc-link-lib=cudart");

    println!("cargo:rustc-link-search=native=/opt/cuda/lib64/stub");
    println!("cargo:rustc-link-lib=cuda");
}
