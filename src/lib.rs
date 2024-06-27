//! An abstraction over the OpenHarmony [XComponent]
//!
//! ## Example
//! ```
//! # use ohos_sys::ace::xcomponent::native_interface_xcomponent::OH_NativeXComponent;
//! # use core::ffi::c_void;
//! pub extern "C" fn on_surface_created_cb(xcomponent: *mut OH_NativeXComponent, window: *mut c_void) {
//!     let xc = xcomponent::XComponent::new(xcomponent, window).expect("Invalid XC");
//!     let size = xc.size();
//!     // do something with the xcomponent ...
//! }
//!
//! pub extern "C" fn on_dispatch_touch_event_cb(
//!     component: *mut OH_NativeXComponent,
//!     window: *mut c_void,
//! ) {
//!      let xc = xcomponent::XComponent::new(component, window).unwrap();
//!      let touch_event = xc.get_touch_event().unwrap();
//!      // Handle the touch event ....
//! }
//! ```
//!
//! ## Features
//!
//! * log: Outputs error and diagnostic messages via the `log` crate if enabled.
//!
//! [XComponent]: https://gitee.com/openharmony/docs/blob/master/zh-cn/application-dev/ui/napi-xcomponent-guidelines.md

use core::{ffi::c_void, marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

use crate::log::error;
use ohos_sys::ace::xcomponent::native_interface_xcomponent::OH_NativeXComponent_GetXComponentSize;
use ohos_sys::{
    ace::xcomponent::native_interface_xcomponent::{
        OH_NativeXComponent, OH_NativeXComponent_Callback, OH_NativeXComponent_GetTouchEvent,
        OH_NativeXComponent_RegisterCallback, OH_NativeXComponent_TouchEvent,
    },
    native_window::OHNativeWindow,
};

mod log;

pub struct Size {
    pub width: u64,
    pub height: u64,
    _opaque: [u64; 0],
}

pub struct XComponent<'a> {
    xcomponent: NonNull<OH_NativeXComponent>,
    window: NonNull<OHNativeWindow>,
    phantom: PhantomData<&'a mut OH_NativeXComponent>,
}

impl<'a> XComponent<'a> {
    pub fn new(
        xcomponent: *mut OH_NativeXComponent,
        window: *mut c_void,
    ) -> Option<XComponent<'a>> {
        Some(XComponent {
            xcomponent: NonNull::new(xcomponent)?,
            window: NonNull::new(window.cast())?,
            phantom: PhantomData,
        })
    }

    pub fn get_touch_event(&self) -> Result<OH_NativeXComponent_TouchEvent, i32> {
        let touch_event = unsafe {
            let mut touch_event: MaybeUninit<OH_NativeXComponent_TouchEvent> =
                MaybeUninit::uninit();
            let res = OH_NativeXComponent_GetTouchEvent(
                self.xcomponent.as_ptr(),
                self.window.as_ptr().cast(),
                touch_event.as_mut_ptr(),
            );
            if res != 0 {
                error!("OH_NativeXComponent_GetTouchEvent failed with {res}");
                return Err(res);
            }
            touch_event.assume_init()
        };

        Ok(touch_event)
    }

    pub fn register_callback(
        &mut self,
        callbacks: OH_NativeXComponent_Callback,
    ) -> Result<(), i32> {
        // Fixme: Leaking the box fixes a crash in release mode.
        // I would expect `OH_NativeXComponent_RegisterCallback` to copy the callbacks
        // synchronously, so the struct shouldn't need to be leaked, and we should be able to
        // use a reference to the stack.
        // This should be investigated at a later point in time, to avoid the leaking.
        let boxed = Box::new(callbacks);
        unsafe {
            let res = OH_NativeXComponent_RegisterCallback(
                self.xcomponent.as_ptr(),
                Box::leak(boxed) as *mut _,
            );
            if res != 0 {
                return Err(res);
            }
        }
        Ok(())
    }

    /// Returns the size of the XComponent
    pub fn size(&self) -> Size {
        let mut width: u64 = 0;
        let mut height: u64 = 0;
        let res = unsafe {
            OH_NativeXComponent_GetXComponentSize(
                self.xcomponent.as_ptr(),
                self.window.as_ptr() as *const c_void,
                &mut width as *mut _,
                &mut height as *mut _,
            )
        };
        assert_eq!(res, 0, "OH_NativeXComponent_GetXComponentSize failed");
        Size {
            width,
            height,
            _opaque: [],
        }
    }
}
