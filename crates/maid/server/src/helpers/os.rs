use std::process::{Command,Stdio};

pub fn uptime() -> u64 {
    let uptime: u64;
    #[cfg(target_os = "linux")]
    {
        let mut info = std::mem::MaybeUninit::uninit();
        let res = unsafe { libc::sysinfo(info.as_mut_ptr()) };
        uptime = if res == 0 {
            let info = unsafe { info.assume_init() };
            info.uptime as u64
        } else {
            0
        }
    }

    #[cfg(any(target_vendor = "apple", target_os = "freebsd", target_os = "openbsd"))]
    {
        use std::mem;
        use std::time::Duration;
        use std::time::SystemTime;
        let mut request = [libc::CTL_KERN, libc::KERN_BOOTTIME];
        let mut boottime: libc::timeval = unsafe { mem::zeroed() };
        let mut size: libc::size_t = mem::size_of_val(&boottime) as libc::size_t;
        let res = unsafe { libc::sysctl(&mut request[0], 2, &mut boottime as *mut libc::timeval as *mut libc::c_void, &mut size, std::ptr::null_mut(), 0) };
        uptime = if res == 0 {
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| (d - Duration::new(boottime.tv_sec as u64, boottime.tv_usec as u32 * 1000)).as_secs())
                .unwrap_or_default()
        } else {
            0
        }
    }

    #[cfg(target_family = "windows")]
    unsafe {
        uptime = winapi::um::sysinfoapi::GetTickCount64() / 1000;
    }

    uptime
}

pub fn release() -> String {
    #[cfg(target_os = "linux")]
    {
        match std::fs::read_to_string("/proc/sys/kernel/osrelease") {
            Ok(mut s) => {
                s.pop();
                s
            }
            _ => String::from(""),
        }
    }
    #[cfg(target_vendor = "apple")]
    {
        let mut s = [0u8; 256];
        let mut mib = [libc::CTL_KERN, libc::KERN_OSRELEASE];
        let mut len = s.len();
        if unsafe { libc::sysctl(mib.as_mut_ptr(), mib.len() as _, s.as_mut_ptr() as _, &mut len, std::ptr::null_mut(), 0) } == -1 {
            return String::from("Unknown");
        }

        return String::from_utf8_lossy(&s[..len - 1]).to_string();
    }
    #[cfg(target_family = "windows")]
    {
        use ntapi::ntrtl::RtlGetVersion;
        use winapi::shared::ntdef::NT_SUCCESS;
        use winapi::um::winnt::RTL_OSVERSIONINFOEXW;

        let mut version_info = std::mem::MaybeUninit::<RTL_OSVERSIONINFOEXW>::uninit();
        unsafe {
            (*version_info.as_mut_ptr()).dwOSVersionInfoSize = std::mem::size_of::<RTL_OSVERSIONINFOEXW>() as u32;
        }
        if !NT_SUCCESS(unsafe { RtlGetVersion(version_info.as_mut_ptr() as *mut _) }) {
            String::from("")
        } else {
            let version_info = unsafe { version_info.assume_init() };
            format!("{}.{}.{}", version_info.dwMajorVersion, version_info.dwMinorVersion, version_info.dwBuildNumber)
        }
    }
}

pub fn health() -> bool {
    let status: bool;
    
    #[cfg(any(target_os = "linux", target_vendor = "apple", target_os = "freebsd", target_os = "openbsd"))]
    {
        let cmd = Command::new("bash")
            .stdout(Stdio::null())
            .arg("-c")
            .arg("docker --version")
            .status()
            .expect("Failed to execute command");
            
        status = cmd.success();
    }
    
    #[cfg(target_family = "windows")]
    {
        let cmd = Command::new("cmd")
            .stdout(Stdio::null())
            .args(&["/C", "docker --version"])
            .status()
            .expect("Failed to execute command");
            
        status = cmd.success();
    }
    
    status
}