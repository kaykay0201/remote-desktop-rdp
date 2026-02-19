#[cfg(windows)]
mod job {
    use std::os::windows::io::RawHandle;
    use std::sync::OnceLock;

    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
        SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };

    struct JobObject(HANDLE);

    unsafe impl Send for JobObject {}
    unsafe impl Sync for JobObject {}

    impl Drop for JobObject {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }

    static JOB: OnceLock<JobObject> = OnceLock::new();

    fn get_or_create_job() -> &'static JobObject {
        JOB.get_or_init(|| {
            unsafe {
                let handle =
                    CreateJobObjectW(std::ptr::null::<SECURITY_ATTRIBUTES>(), std::ptr::null());
                assert!(handle != 0, "CreateJobObjectW failed");

                let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
                info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

                let ret = SetInformationJobObject(
                    handle,
                    JobObjectExtendedLimitInformation,
                    (&raw const info).cast(),
                    std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                );
                assert!(ret != 0, "SetInformationJobObject failed");

                JobObject(handle)
            }
        })
    }

    pub fn assign_child_to_job(raw: RawHandle) {
        let job = get_or_create_job();
        unsafe {
            AssignProcessToJobObject(job.0, raw as HANDLE);
        }
    }
}

#[cfg(windows)]
pub use job::assign_child_to_job;
