use bootloader::{BiosBoot, UefiBoot};
use std::{env, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("Failed to find out dir!"));
    let kernel = PathBuf::from(
        env::var_os("CARGO_BIN_FILE_ZEFIR_OS_KERNEL_zefir-os-kernel")
            .expect("Failed to find kernel artifact path!"),
    );

    let uefi_path = out_dir.join("uefi.img");
    UefiBoot::new(&kernel)
        .create_disk_image(&uefi_path)
        .expect("Failed to build UEFI boot image!");

    let bios_path = out_dir.join("bios.img");
    BiosBoot::new(&kernel)
        .create_disk_image(&bios_path)
        .expect("Failed to build BIOS boot image!");

    println!("cargo:rustc-env=UEFI_PATH={}", uefi_path.display());
    println!("cargo:rustc-env=BIOS_PATH={}", bios_path.display());
}
