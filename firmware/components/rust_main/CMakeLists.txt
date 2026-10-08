# Based on esp-rs/esp-idf-template's official CMake mixed Rust/C template.
# The generated template is licensed MIT-0; see firmware/THIRD-PARTY.md.
set(RUST_DEPS pthread vfs fatfs spiffs esp_timer esp_driver_usb_serial_jtag lwip board_p4)
idf_component_register(SRCS "placeholder.c" INCLUDE_DIRS "include"
    PRIV_REQUIRES ${RUST_DEPS})
include(ExternalProject)

set(RUST_TARGET riscv32imafc-esp-espidf)
set(RUST_TOOLCHAIN nightly-2026-09-27 CACHE STRING "Pinned Rust toolchain")
set(CARGO_PROJECT_DIR "${CMAKE_CURRENT_LIST_DIR}")
set(CARGO_BUILD_DIR "${CMAKE_CURRENT_BINARY_DIR}")
set(CARGO_TARGET_DIR "${CARGO_BUILD_DIR}/target")
file(MAKE_DIRECTORY "${CARGO_TARGET_DIR}")
if(NOT EXISTS "${CARGO_TARGET_DIR}/CACHEDIR.TAG")
    file(WRITE "${CARGO_TARGET_DIR}/CACHEDIR.TAG"
        "Signature: 8a477f597d28d172789f06886806bc55\n# P4Desk owned Cargo build cache.\n")
endif()
set(RUST_STATIC_LIBRARY "${CARGO_TARGET_DIR}/${RUST_TARGET}/release/librust_main.a")
idf_build_get_property(sdkconfig SDKCONFIG)
idf_build_get_property(idf_path IDF_PATH)
file(SHA256 "${sdkconfig}" RUST_SDKCONFIG_HASH)
file(SHA256 "${PROJECT_DIR}/dependencies.lock" RUST_COMPONENTS_HASH)
string(SHA256 RUST_SHARED_ABI_HASH "${RUST_DEPS};${RUST_SDKCONFIG_HASH};${RUST_COMPONENTS_HASH};${idf_path};${CMAKE_C_COMPILER}")

ExternalProject_Add(project_rust_main
    PREFIX "${CARGO_BUILD_DIR}"
    DOWNLOAD_COMMAND ""
    # esp-idf-sys's CMake driver does not track CMake environment changes in
    # Cargo. Regenerate its bindings when the shared IDF configuration changes.
    CONFIGURE_COMMAND ${CMAKE_COMMAND} -E echo "Shared IDF ABI ${RUST_SHARED_ABI_HASH}"
    COMMAND cargo +${RUST_TOOLCHAIN} clean --manifest-path "${CARGO_PROJECT_DIR}/Cargo.toml"
        --target ${RUST_TARGET} --target-dir "${CARGO_TARGET_DIR}"
    USES_TERMINAL_BUILD true
    BUILD_COMMAND ${CMAKE_COMMAND} -E env
        "CARGO_CMAKE_BUILD_INCLUDES=$<TARGET_PROPERTY:${COMPONENT_LIB},INCLUDE_DIRECTORIES>"
        "CARGO_CMAKE_BUILD_LINK_LIBRARIES=$<TARGET_PROPERTY:${COMPONENT_LIB},LINK_LIBRARIES>"
        "CARGO_CMAKE_BUILD_SDKCONFIG=${sdkconfig}"
        "CARGO_CMAKE_BUILD_ESP_IDF=${idf_path}"
        "CARGO_CMAKE_BUILD_COMPILER=${CMAKE_C_COMPILER}"
        "CARGO_TARGET_DIR=${CARGO_TARGET_DIR}"
        # The cfg must also reach build-std's libc: IDF 6 uses picolibc's
        # O_CREAT/O_TRUNC/O_APPEND values, not newlib's values.
        "RUSTFLAGS=--cfg espidf_time64 --cfg espidf_picolibc"
        "MCU=esp32p4"
        cargo +${RUST_TOOLCHAIN} build --manifest-path "${CARGO_PROJECT_DIR}/Cargo.toml"
            --target ${RUST_TARGET} --target-dir "${CARGO_TARGET_DIR}" --release
            -Zbuild-std=std,panic_abort
    INSTALL_COMMAND ""
    BUILD_ALWAYS true
    TMP_DIR "${CARGO_BUILD_DIR}/tmp"
    STAMP_DIR "${CARGO_BUILD_DIR}/stamp"
    SOURCE_DIR "${CARGO_PROJECT_DIR}"
    BINARY_DIR "${CARGO_PROJECT_DIR}"
    BUILD_BYPRODUCTS "${RUST_STATIC_LIBRARY}")

add_prebuilt_library(library_rust_main "${RUST_STATIC_LIBRARY}" PRIV_REQUIRES ${RUST_DEPS})
add_dependencies(library_rust_main project_rust_main)
target_link_libraries(${COMPONENT_LIB} PRIVATE library_rust_main)
