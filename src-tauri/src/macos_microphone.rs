//! Microphone (TCC) permission handling on macOS via AVCaptureDevice.
//!
//! cpal touching CoreAudio implicitly triggers the microphone permission
//! prompt, so the UI must check/request the permission explicitly *before*
//! enumerating or opening input devices. The request must run off the main
//! thread: if the main run loop is blocked while the prompt is up, the dialog
//! can never deliver its result and keeps re-presenting.

use block2::RcBlock;
use objc2::runtime::{AnyObject, Bool};
use objc2::{class, msg_send};
use std::sync::Mutex;

#[link(name = "AVFoundation", kind = "framework")]
extern "C" {
    /// NSString* AVMediaTypeAudio
    static AVMediaTypeAudio: *const AnyObject;
}

// AVAuthorizationStatus values.
const AV_AUTHORIZATION_STATUS_NOT_DETERMINED: isize = 0;
const AV_AUTHORIZATION_STATUS_RESTRICTED: isize = 1;
const AV_AUTHORIZATION_STATUS_AUTHORIZED: isize = 3;

/// Current microphone permission status. Never triggers the system prompt.
pub fn microphone_permission_status() -> &'static str {
    unsafe {
        let status: isize = msg_send![
            class!(AVCaptureDevice),
            authorizationStatusForMediaType: AVMediaTypeAudio
        ];
        match status {
            AV_AUTHORIZATION_STATUS_NOT_DETERMINED => "notdetermined",
            AV_AUTHORIZATION_STATUS_RESTRICTED => "restricted",
            AV_AUTHORIZATION_STATUS_AUTHORIZED => "granted",
            _ => "denied",
        }
    }
}

/// Fire the system microphone prompt and resolve once the user answers.
/// If permission was already decided, the completion handler fires
/// immediately with the stored answer.
pub async fn request_microphone_permission() -> bool {
    // All ObjC values stay inside the sync helper: the returned future must be
    // Send (Tauri requirement), and RcBlock is not.
    spawn_request().await.unwrap_or(false)
}

fn spawn_request() -> tokio::sync::oneshot::Receiver<bool> {
    let (tx, rx) = tokio::sync::oneshot::channel::<bool>();
    // The completion block is `Fn`, but the oneshot sender must be consumed.
    let tx = Mutex::new(Some(tx));
    let block = RcBlock::new(move |granted: Bool| {
        if let Some(tx) = tx.lock().unwrap().take() {
            let _ = tx.send(granted.as_bool());
        }
    });
    // AVFoundation copies the (refcounted) block for the async callback, so
    // dropping our reference when this fn returns is fine.
    unsafe {
        let _: () = msg_send![
            class!(AVCaptureDevice),
            requestAccessForMediaType: AVMediaTypeAudio,
            completionHandler: &*block
        ];
    }
    rx
}
