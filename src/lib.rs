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
//! * register: Add `register_xcomponent_callbacks` function to register XComponent callbacks.
//!
//! [XComponent]: https://gitee.com/openharmony/docs/blob/master/zh-cn/application-dev/ui/napi-xcomponent-guidelines.md

use crate::log::error;
use core::{ffi::c_void, marker::PhantomData, mem::MaybeUninit, ptr::NonNull};
use ohos_sys::ace::xcomponent::native_interface_xcomponent::OH_NativeXComponent_GetXComponentSize;
use ohos_sys::{
    ace::xcomponent::native_interface_xcomponent::{
        OH_NativeXComponent, OH_NativeXComponent_GetTouchEvent, OH_NativeXComponent_TouchEvent,
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

#[cfg(feature = "register")]
#[cfg_attr(docsrs, doc(cfg(feature = "register")))]
#[derive(Debug)]
pub enum RegisterCallbackError {
    XcomponentPropertyMissing(String),
    UnwrapXComponentFailed(i32),
    RegisterCallbackFailed(i32),
}

#[cfg(feature = "register")]
#[cfg_attr(docsrs, doc(cfg(feature = "register")))]
impl Into<String> for RegisterCallbackError {
    fn into(self) -> String {
        format!("{:?}", self)
    }
}

/// Register callbacks for the XComponent
///
/// This function is intended to be called from the module init function (See Example below).
/// We currently require the `callbacks` parameter to have a static lifetime, since despite
/// contrary documentation `OH_NativeXComponent_RegisterCallback` seems to use the address of
/// `callback` after it has returned.
///
///
///
///
/// ## Example:
///
/// ```
/// # use core::ffi::c_void;
/// # use log::info;
/// # use ohos_sys::ace::xcomponent::native_interface_xcomponent::{OH_NativeXComponent, OH_NativeXComponent_Callback};
/// // use napi_derive_ohos::module_exports;
/// // #[module_exports]
/// fn init(exports: napi_ohos::JsObject, env: napi_ohos::Env) -> napi_ohos::Result<()> {
///     xcomponent::register_xcomponent_callbacks(&exports, &env, &XC_CALLBACKS)
///         .expect("Registering Callback failed.");
///     Ok(())
/// }
///
/// static XC_CALLBACKS: OH_NativeXComponent_Callback = OH_NativeXComponent_Callback {
///     OnSurfaceCreated: Some(on_surface_created_cb),
///     OnSurfaceChanged: Some(on_surface_changed_cb),
///     OnSurfaceDestroyed: Some(on_surface_destroyed_cb),
///     DispatchTouchEvent: Some(on_dispatch_touch_event_cb),
/// };
///
/// // Note: `pub` attribute or `#[no_mangle]` are NOT required, since we just register the function
/// // pointer.
/// extern "C" fn on_surface_created_cb(xcomponent: *mut OH_NativeXComponent, window: *mut c_void) {
///     info!("on_surface_created_cb");
/// }
/// extern "C" fn on_surface_changed_cb(xcomponent: *mut OH_NativeXComponent, window: *mut c_void) {
///     info!("on_surface_changed_cb");
/// }
/// extern "C" fn on_surface_destroyed_cb(xcomponent: *mut OH_NativeXComponent, window: *mut c_void) {
///     info!("on_surface_destroyed_cb");
/// }
/// extern "C" fn on_dispatch_touch_event_cb(xcomponent: *mut OH_NativeXComponent, window: *mut c_void) {
///     info!("on_dispatch_touch_event_cb");
/// }
/// ```
#[cfg(feature = "register")]
#[cfg_attr(docsrs, doc(cfg(feature = "register")))]
pub fn register_xcomponent_callbacks(
    exports: &napi_ohos::JsObject,
    env: &napi_ohos::Env,
    callbacks: &'static ohos_sys::ace::xcomponent::native_interface_xcomponent::OH_NativeXComponent_Callback,
) -> Result<(), RegisterCallbackError> {
    use napi_ohos::NapiRaw;
    use ohos_sys::ace::xcomponent::native_interface_xcomponent::OH_NativeXComponent_RegisterCallback;

    let xcomponent_js_object = exports
        .get_named_property::<napi_ohos::JsObject>("__NATIVE_XCOMPONENT_OBJ__")
        .map_err(|e| RegisterCallbackError::XcomponentPropertyMissing(e.to_string()))?;
    let raw = unsafe { xcomponent_js_object.raw() };
    let raw_env = env.raw();
    let mut native_xcomponent: *mut OH_NativeXComponent = core::ptr::null_mut();
    let res = unsafe {
        napi_ohos::sys::napi_unwrap(
            raw_env,
            raw,
            &mut native_xcomponent as *mut *mut OH_NativeXComponent as *mut *mut c_void,
        )
    };
    if res != 0 {
        return Err(RegisterCallbackError::UnwrapXComponentFailed(res));
    }
    let res =
        // Note: The register function seems to offload the work to some other thread and return early.
        // so the CBs need to live longer than this function ....
        // SAFETY: `OH_NativeXComponent_RegisterCallback` will not mutate `callbacks`.
        unsafe { OH_NativeXComponent_RegisterCallback(native_xcomponent, callbacks as *const _ as *mut _) };
    if res != 0 {
        return Err(RegisterCallbackError::RegisterCallbackFailed(res));
    }
    Ok(())
}
