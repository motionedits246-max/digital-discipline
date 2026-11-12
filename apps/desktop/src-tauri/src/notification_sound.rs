#[cfg(windows)]
pub fn play() {
    use std::ffi::c_void;

    #[link(name = "winmm")]
    extern "system" {
        fn PlaySoundW(sound: *const u16, module: *mut c_void, flags: u32) -> i32;
    }

    const ASYNC: u32 = 0x0001;
    const NO_DEFAULT: u32 = 0x0002;
    const MEMORY: u32 = 0x0004;
    // Windows reads the waveform asynchronously, so its bytes must stay alive.
    static SOUND: &[u8] = include_bytes!("../assets/notification.wav");

    unsafe {
        PlaySoundW(SOUND.as_ptr().cast(), std::ptr::null_mut(), ASYNC | NO_DEFAULT | MEMORY);
    }
}

#[cfg(not(windows))]
pub fn play() {}
