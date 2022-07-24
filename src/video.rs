use ::context::Context;
use ::camera::Camera;
use ::handle::prelude::*;


/// Capture a video 
pub struct Video
{
    fifo_path: String,
    fifo_fd: i32,
    camfile: *mut ::gphoto2::CameraFile,
}

/*
/// Video output type
pub enum OutputType
{
    /// No output
    none,

    /// Pipe to Stdout
    stdout,

    /// Pipe to fifio
    fifo(path, fd),

    /// Write to file
    file(path, fd),
}*/

impl Drop for Video
{
    fn drop(&mut self) {
        unsafe {
            println!("+++++++++++++++++ Dropping Video: {:?}", self.camfile);
            // TODO: check if null before unref
            ::gphoto2::gp_file_unref(self.camfile);
        }
    }
}

impl Video
{
    pub fn new_fifo(fifo_path: &str) -> Self
    {
        Video {
            fifo_path: fifo_path.to_owned(),
            fifo_fd: -1,
            camfile: std::ptr::null_mut()
        }
    }

    /// Start video capture
    pub fn start(&mut self)
    {
        println!("Camera stream starting...");
        unsafe {

            // Stdout
            //let fd = unsafe { ::libc::STDOUT_FILENO };

            // Create fifo
            let filename = std::ffi::CString::new(self.fifo_path.clone()).unwrap();
            libc::unlink(filename.as_ptr());
            let ret = libc::mkfifo(filename.as_ptr(), 0o644);
            if ret < 0 {
                println!("Failed to mkfifo: {ret}");
            }

            // Open via libc to retrieve the fd
            self.fifo_fd = ::libc::open(filename.as_ptr(), libc::O_RDWR, 0o644); // | O_NONBLOCK
            if self.fifo_fd < 0 {
                println!("Failed to open fifo")
                //return Err(::error::from_libgphoto2(::gphoto2::GP_ERROR_FILE_EXISTS));
            }

            // Bind fifo_fd to camfile
            match ::gphoto2::gp_file_new_from_fd(&mut self.camfile as *mut _, self.fifo_fd) {
                ::gphoto2::GP_OK => {},
                err => {
                    println!("gp_file_new_from_fd error: {err}");
                    //return Err(::error::from_libgphoto2(err))
                }
            }
        }   
        
        println!("Camera stream started");
    }

    /// Stop video
    pub fn stop(&mut self)
    {
        unsafe {
            // Delete fifo
            let filename = std::ffi::CString::new(self.fifo_path.clone()).unwrap();
            libc::unlink(filename.as_ptr());

            // Unref camfile
            ::gphoto2::gp_file_unref(self.camfile);
        }
    }

    /// Poll camera for next frame
    /// Pipes frame directly to output
    /// 
    /// Note: This command will block if the output is a FIFO, and no consumer
    /// is attached to the fifo (such as ffmpeg).
    pub fn poll(&mut self, camera: &mut Camera, context: &mut Context) -> ::Result<()>
    {
        Ok(try_unsafe!(::gphoto2::gp_camera_capture_preview(camera.as_mut_ptr(),
            self.camfile, 
            context.as_mut_ptr())))
    }

}