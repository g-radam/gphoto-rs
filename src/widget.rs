use std::borrow::Cow;
use std::ffi::{ CString, CStr };
use std::str;
use std::marker::PhantomData;
use libc;

/// Structure containing information about a camera's widget.
///
/// ## Example
///
/// A `Widget` object can be used to set and get a camera widget:
///
/// ```no_run
/// let mut context = gphoto::Context::new().unwrap();
/// let mut camera = gphoto::Camera::autodetect(&mut context).unwrap();
///
/// let widgets = camera.widget(&mut context);
/// for widget in widgets.iter() {
///     println!("           name = {:?}", storage.name());
///     println!("          label = {:?}", storage.label());
///     println!("          value = {:?}", storage.value());
///     println!("       readonly = {:?}", storage.readonly());
///     println!("           path = {:?}", storage.path());
/// }
/// ```
#[repr(C)]
pub struct Widget {
    // Below pointers are required to dynamically get and set camera widgets.
    // Lifetime is needed because it borrows data owned by the Camera struct.
    pub context: *mut ::gphoto2::GPContext,
    pub camera: *mut ::gphoto2::Camera,
    pub window: *mut ::gphoto2::CameraWidget,
    pub inner: *mut ::gphoto2::CameraWidget,
}

impl Drop for Widget {
    fn drop(&mut self) {
        unsafe {
            println!("+++++++++++++++++ Dropping: {:?}", self.inner);
            ::gphoto2::gp_context_unref(self.context);
            ::gphoto2::gp_camera_unref(self.camera);
            ::gphoto2::gp_widget_unref(self.window);
            ::gphoto2::gp_widget_unref(self.inner);
        }
    }
}

impl Widget {

    // Construct widget from sys cameraWidget
    pub fn from_raw(context: *mut ::gphoto2::GPContext, camera: *mut ::gphoto2::Camera,window_ptr: *mut ::gphoto2::CameraWidget, ptr: *mut ::gphoto2::CameraWidget) -> ::Result<Self>
    {
        Ok(Widget {
            context: context,
            camera: camera,
            window: window_ptr,
            inner: ptr,
        })
    }

    // Widget name
    pub fn name(&self) -> &str {
        let mut name_ptr: *const libc::c_char = std::ptr::null_mut();
        unsafe { 
            assert_eq!(::gphoto2::GP_OK, ::gphoto2::gp_widget_get_name(self.inner, &mut name_ptr));
            CStr::from_ptr(name_ptr).to_str().unwrap()
        }

        /*let mut ptr: *const libc::c_char = std::ptr::null_mut(); 
        unsafe {
            assert_eq!(::gphoto2::GP_OK, ::gphoto2::gp_widget_get_name(self.inner, &mut ptr));
            String::from_utf8_lossy(CStr::from_ptr(ptr).to_bytes())
        }*/
    }

    // Widget label
    pub fn label(&self) -> &str {
        let mut label_ptr: *const libc::c_char = std::ptr::null_mut();
        unsafe { 
            assert_eq!(::gphoto2::GP_OK, ::gphoto2::gp_widget_get_label(self.inner, &mut label_ptr));
            CStr::from_ptr(label_ptr).to_str().unwrap()
        }
        /*let mut ptr: *const libc::c_char = std::ptr::null_mut(); 
        unsafe {
            assert_eq!(::gphoto2::GP_OK, ::gphoto2::gp_widget_get_label(self.inner, &mut ptr));
            String::from_utf8_lossy(CStr::from_ptr(ptr).to_bytes())
        }*/
    }

    // Widget readonly
    pub fn readonly(&self) -> bool {
        let mut readonly: libc::c_int = 0;
        unsafe { 
            assert_eq!(::gphoto2::GP_OK, ::gphoto2::gp_widget_get_readonly(self.inner, &mut readonly));
        }
        readonly != 0
    }
    
    // Widget type
    pub fn ty(&self) -> gphoto2_sys::CameraWidgetType {
        let mut ty = gphoto2_sys::CameraWidgetType::GP_WIDGET_WINDOW;
        unsafe { 
            assert_eq!(::gphoto2::GP_OK, ::gphoto2::gp_widget_get_type(self.inner, &mut ty));
        }
        ty

    }

    // Widget value
    pub fn value(&self) -> Option<WidgetValue> {
        use ::gphoto2::CameraWidgetType::*;
        let mut ty = GP_WIDGET_WINDOW; 
        unsafe {
            assert_eq!(::gphoto2::GP_OK, ::gphoto2::gp_widget_get_type(self.inner, &mut ty));
            match ty
            {
                // No value
                GP_WIDGET_BUTTON | GP_WIDGET_SECTION | GP_WIDGET_WINDOW => {
                    None
                },

                // Char*, Char*[]
                GP_WIDGET_MENU | GP_WIDGET_RADIO => {
                    let value: *mut libc::c_char = std::ptr::null_mut(); 
                    assert_eq!(::gphoto2::GP_OK, ::gphoto2::gp_widget_get_value(self.inner, std::mem::transmute(&value)));

                    // Get choices
                    let mut choices = vec!(); 
                    let choices_count = ::gphoto2::gp_widget_count_choices(self. inner);
                    for i in 0..choices_count {
                        let mut choice: *const libc::c_char = std::ptr::null_mut();
                        assert_eq!(::gphoto2::GP_OK, ::gphoto2::gp_widget_get_choice(self.inner, i, &mut choice));
                        choices.push(CStr::from_ptr(choice).to_str().unwrap());
                    }

                    Some(WidgetValue::Select(String::from_utf8_lossy(CStr::from_ptr(value).to_bytes()), Some(choices)))
                }
                
                // Char*
                GP_WIDGET_TEXT => {
                    let value: *mut libc::c_char = std::ptr::null_mut(); 
                    assert_eq!(::gphoto2::GP_OK, ::gphoto2::gp_widget_get_value(self.inner, std::mem::transmute(&value)));
                    Some(WidgetValue::Text(String::from_utf8_lossy(CStr::from_ptr(value).to_bytes()) ))
                },

                // Integer
                GP_WIDGET_TOGGLE | GP_WIDGET_DATE => {
                    let value: libc::c_int = 0; 
                    assert_eq!(::gphoto2::GP_OK, ::gphoto2::gp_widget_get_value(self.inner, std::mem::transmute(&value)));
                    Some(WidgetValue::Number(value))
                },

                // Float
                GP_WIDGET_RANGE => {
                    let value: libc::c_float = 0.0; 
                    let min: libc::c_float = 0.0; 
                    let max: libc::c_float = 0.0; 
                    let inc: libc::c_float = 0.0; 
                    assert_eq!(::gphoto2::GP_OK, ::gphoto2::gp_widget_get_value(self.inner, std::mem::transmute(&value)));
                    assert_eq!(::gphoto2::GP_OK, ::gphoto2::gp_widget_get_range(self.inner,
                        std::mem::transmute(&min),
                        std::mem::transmute(&max),
                        std::mem::transmute(&inc)));
                    Some(WidgetValue::Range(value, min, max, inc))
                },
            }
        }
    }

    // Widget value
    pub fn set_value_string(&self, value_string: &str) {
        use ::gphoto2::CameraWidgetType::*;

        // Exit if widget is readonly
        if self.readonly() {
            println!("Cannot set value: Widget is readonly");
            return;
        }

        // Updates widgets internal value
        let mut ty = GP_WIDGET_WINDOW; 
        unsafe {
            assert_eq!(::gphoto2::GP_OK, ::gphoto2::gp_widget_get_type(self.inner, &mut ty));
            match ty
            {
                // No value
                GP_WIDGET_BUTTON | GP_WIDGET_SECTION | GP_WIDGET_WINDOW => {
                },

                // Char*
                GP_WIDGET_MENU | GP_WIDGET_RADIO | GP_WIDGET_TEXT => {
                    let value = CString::new(value_string).expect("Invalid value string");
                    println!("Setting (GP_WIDGET_MENU | GP_WIDGET_RADIO | GP_WIDGET_TEXT) => {}", value_string);
                    assert_eq!(::gphoto2::GP_OK, ::gphoto2::gp_widget_set_value(self.inner, std::mem::transmute(value.as_ref().as_ptr())));
                }

                // Integer - Convert string to 0 or 1 (integer boolean) from textual version
                GP_WIDGET_TOGGLE => {
                    let value = match value_string {
                        "true"  | "yes" | "on"  | "1" => 1,
                        "false" | "no"  | "off" | "0" => 0,
                        _ => panic!("Unknown value_string value: {}", value_string)
                    };
                    println!("Setting (GP_WIDGET_TOGGLE) => {}", value);
                    assert_eq!(::gphoto2::GP_OK, ::gphoto2::gp_widget_get_value(self.inner, std::mem::transmute(&value)));
                },
                
                // Integer
                GP_WIDGET_DATE => {
                    // TODO: Handle "Now" string OR unix-time integer?
                    // Unix time
                    let value: libc::c_int = value_string.parse().expect("Invalid value string: Unable to parse into int");
                    println!("Setting (GP_WIDGET_DATE) => {}", value);
                    assert_eq!(::gphoto2::GP_OK, ::gphoto2::gp_widget_get_value(self.inner, std::mem::transmute(&value)));
                },

                // Float
                GP_WIDGET_RANGE => {
                    // TODO: check value is within range
                    let value: libc::c_float = value_string.parse().expect("Invalid value string: Unable to parse into float");
                    println!("Setting (GP_WIDGET_RANGE) => {}", value);
                    assert_eq!(::gphoto2::GP_OK, ::gphoto2::gp_widget_get_value(self.inner, std::mem::transmute(&value)));
                },
            }
        }

        // Sync widget back to camera
        
        // Extract all widgets
        unsafe { 
            ::gphoto2::gp_camera_set_config(self.camera, self.window, self.context);
        }

    }
    
}


/// Config WidgetValue
#[derive(Debug)]
pub enum WidgetValue<'a>
{
    Select(Cow<'a, str>, Option<Vec<&'a str>>),
    Range(f32, f32, f32, f32),
    Text(Cow<'a, str>),
    Number(i32),
}