#![doc = include_str!("../README.md")]

/// Exports an AUv2 entrypoint named `GetPluginFactoryAUV2` that wraps a global CLAP entrypoint (`clap_entry`) exported from the resulting shared library.
/// Currently it only reexports the first plugin available through the CLAP entrypoint, but this limitation might be lifted in the future.
///
/// Failure to export a CLAP entrypoint might result in linker errors or missing symbols.
#[macro_export]
macro_rules! export_vst3 {
    () => {
        #[allow(unused_imports)]
        pub use $crate::vst3::*;
    };
}

/// Exports a VST3 entrypoint named `GetPluginFactory` that wraps a global CLAP entrypoint (`clap_entry`) exported from the resulting shared library.
///
/// Failure to export a CLAP entrypoint might result in linker errors or missing symbols.
#[macro_export]
macro_rules! export_auv2 {
    () => {
        #[allow(unused_imports)]
        pub use $crate::auv2::*;
    };
}

#[cfg(all(target_os = "macos", feature = "auv2"))]
#[doc(hidden)]
pub mod auv2 {
    use core::ffi::c_void;

    #[link(name = "clap_wrapper_auv2")]
    unsafe extern "C" {
        pub unsafe fn wrapAsAUV2_inst0Factory(desc: *const c_void) -> *mut c_void;
        pub unsafe fn wrapAsAUV2_inst1Factory(desc: *const c_void) -> *mut c_void;
        pub unsafe fn wrapAsAUV2_inst2Factory(desc: *const c_void) -> *mut c_void;
        pub unsafe fn wrapAsAUV2_inst3Factory(desc: *const c_void) -> *mut c_void;
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn GetPluginFactoryAUV2(desc: *const c_void) -> *mut c_void {
        unsafe { wrapAsAUV2_inst0Factory(desc) }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn GetPluginFactoryAUV2_0(desc: *const c_void) -> *mut c_void {
        unsafe { wrapAsAUV2_inst0Factory(desc) }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn GetPluginFactoryAUV2_1(desc: *const c_void) -> *mut c_void {
        unsafe { wrapAsAUV2_inst1Factory(desc) }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn GetPluginFactoryAUV2_2(desc: *const c_void) -> *mut c_void {
        unsafe { wrapAsAUV2_inst2Factory(desc) }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn GetPluginFactoryAUV2_3(desc: *const c_void) -> *mut c_void {
        unsafe { wrapAsAUV2_inst3Factory(desc) }
    }
}

#[cfg(not(all(target_os = "macos", feature = "auv2")))]
#[doc(hidden)]
pub mod auv2 {}

#[cfg(feature = "vst3")]
#[doc(hidden)]
pub mod vst3 {
    use core::ffi::c_void;

    #[link(name = "clap_wrapper_vst3")]
    unsafe extern "system" {
        unsafe fn clap_wrapper_GetPluginFactory() -> *mut c_void;
    }

    #[link(name = "clap_wrapper_vst3")]
    #[cfg(all(target_family = "unix", not(target_os = "macos")))]
    unsafe extern "C" {
        unsafe fn clap_wrapper_ModuleEntry(lib_module: *mut c_void) -> bool;
        unsafe fn clap_wrapper_ModuleExit() -> bool;
    }

    #[link(name = "clap_wrapper_vst3")]
    #[cfg(target_os = "macos")]
    unsafe extern "C" {
        unsafe fn clap_wrapper_bundleEntry(lib_module: *mut c_void) -> bool;
        unsafe fn clap_wrapper_bundleExit() -> bool;
    }

    #[link(name = "clap_wrapper_vst3")]
    #[cfg(target_os = "windows")]
    unsafe extern "system" {
        unsafe fn clap_wrapper_InitDll() -> bool;
        unsafe fn clap_wrapper_ExitDll() -> bool;
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "system" fn GetPluginFactory() -> *mut c_void {
        unsafe { clap_wrapper_GetPluginFactory() }
    }

    #[unsafe(no_mangle)]
    #[cfg(all(target_family = "unix", not(target_os = "macos")))]
    pub unsafe extern "C" fn ModuleEntry(lib_module: *mut c_void) -> bool {
        unsafe { clap_wrapper_ModuleEntry(lib_module) }
    }

    #[unsafe(no_mangle)]
    #[cfg(all(target_family = "unix", not(target_os = "macos")))]
    pub unsafe extern "C" fn ModuleExit() -> bool {
        unsafe { clap_wrapper_ModuleExit() }
    }

    #[unsafe(no_mangle)]
    #[cfg(target_os = "macos")]
    pub unsafe extern "C" fn bundleEntry(lib_module: *mut c_void) -> bool {
        unsafe { clap_wrapper_bundleEntry(lib_module) }
    }

    #[unsafe(no_mangle)]
    #[cfg(target_os = "macos")]
    pub unsafe extern "C" fn bundleExit() -> bool {
        unsafe { clap_wrapper_bundleExit() }
    }

    #[unsafe(no_mangle)]
    #[cfg(target_os = "windows")]
    pub unsafe extern "system" fn InitDll() -> bool {
        unsafe { clap_wrapper_InitDll() }
    }

    #[unsafe(no_mangle)]
    #[cfg(target_os = "windows")]
    pub unsafe extern "system" fn ExitDll() -> bool {
        unsafe { clap_wrapper_ExitDll() }
    }
}

#[cfg(not(feature = "vst3"))]
#[doc(hidden)]
pub mod vst3 {}
