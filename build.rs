fn main() {
    let egaroucid_dir = std::path::PathBuf::from("Egaroucid");

    if egaroucid_dir.join("src/lib/egaroucid_c_api.cpp").exists() {
        println!("cargo:rerun-if-changed=Egaroucid/src");
        println!("cargo:rerun-if-changed=Egaroucid/include");

        let mut build = cc::Build::new();
        build
            .cpp(true)
            .flag_if_supported("-std=c++20")
            .flag_if_supported("-O2")
            .flag_if_supported("-pthread")
            .define("EGAROUCID_BUILDING_DLL", None)
            .define("EGAROUCID_STATIC", None)
            .include("Egaroucid/include")
            .include("Egaroucid/src")
            .file("Egaroucid/src/lib/egaroucid_c_api.cpp");

        #[cfg(target_arch = "aarch64")]
        {
            build.define("HAS_ARM_PROCESSOR", None);
            build.define("HAS_NO_AVX2", None);
        }

        #[cfg(target_arch = "x86_64")]
        {
            build.flag_if_supported("-mavx2");
            build.flag_if_supported("-mfma");
            build.flag_if_supported("-mbmi");
            build.flag_if_supported("-mbmi2");
            build.flag_if_supported("-mpopcnt");
            build.flag_if_supported("-lzcnt");
            build.flag_if_supported("/arch:AVX2");
        }

        build.compile("egaroucid");

        #[cfg(target_os = "macos")]
        println!("cargo:rustc-link-lib=c++");

        #[cfg(target_os = "linux")]
        println!("cargo:rustc-link-lib=stdc++");
    }
}
