use std::ffi::{CString};
use std::mem;
use std::path::Path;

use std::os::unix::prelude::*;


/// A trait for types that can store media.
pub trait Media {
    #[doc(hidden)]
    unsafe fn as_mut_ptr(&mut self) -> *mut ::gphoto2::CameraFile;
}


/// Media stored as a local file.
pub struct FileMedia {
    file: *mut ::gphoto2::CameraFile,
}

impl Drop for FileMedia {
    fn drop(&mut self) {
        unsafe {
            ::gphoto2::gp_file_unref(self.file);
        }
    }
}

impl FileMedia {
    /// Creates a new file that stores media.
    ///
    /// This function creates a new file on disk. The file will start out empty.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the file can not be created:
    ///
    /// * `FileExists` if the file already exists.
    pub fn create(path: &Path) -> ::Result<Self> {
        use ::libc::{O_CREAT,O_EXCL,O_RDWR};

        let cstr = match CString::new(path.as_os_str().as_bytes()) {
            Ok(s) => s,
            Err(_) => return Err(::error::from_libgphoto2(::gphoto2::GP_ERROR_BAD_PARAMETERS))
        };

        let fd = unsafe { ::libc::open(cstr.as_ptr(), O_CREAT|O_EXCL|O_RDWR, 0o644) };
        if fd < 0 {
            return Err(::error::from_libgphoto2(::gphoto2::GP_ERROR_FILE_EXISTS));
        }

        let mut ptr = unsafe { mem::MaybeUninit::uninit().assume_init() };

        match unsafe { ::gphoto2::gp_file_new_from_fd(&mut ptr, fd) } {
            ::gphoto2::GP_OK => {
                Ok(FileMedia { file: ptr })
            },
            err => {
                unsafe {
                    ::libc::close(fd);
                }

                Err(::error::from_libgphoto2(err))
            }
        }
    }

    /// Creates an in-memory file that stores media.
    ///
    /// This function creates an empty FileMedia for in-memory data retrieval
    ///
    /// ## Errors
    ///
    /// This function returns an error if unable to create file media object
    pub fn create_mem() -> ::Result<Self> {
        let mut ptr = std::ptr::null_mut();

        match unsafe { ::gphoto2::gp_file_new(&mut ptr) } {
            ::gphoto2::GP_OK => Ok(FileMedia { file: ptr }),
            err => Err(::error::from_libgphoto2(err)),
        }
    }

    /// Returns media data as a array of bytes
    pub fn as_bytes(&mut self) -> Vec<u8> {
        let mut ptr: *const i8 = std::ptr::null();
        let mut len: ::libc::c_ulong = 0;

        unsafe {
            ::gphoto2::gp_file_get_data_and_size(self.file, &mut ptr, &mut len);
            std::slice::from_raw_parts(ptr as *const u8, len as usize).to_vec() 
        }
    }
}

impl Media for FileMedia {
    #[doc(hidden)]
    unsafe fn as_mut_ptr(&mut self) -> *mut ::gphoto2::CameraFile {
        self.file
    }
}
