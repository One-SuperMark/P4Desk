fn main() {
    println!("cargo:rustc-check-cfg=cfg(espidf_time64)");
    println!("cargo:rerun-if-changed=../../../include/p4desk_hal.h");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("espidf") {
        embuild::espidf::sysenv::output();
    }
}
