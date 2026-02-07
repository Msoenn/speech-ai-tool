use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconId},
    webview::WebviewWindowBuilder,
    App, AppHandle, Manager,
};

const TRAY_ID: &str = "main-tray";
const ICON_SIZE: u32 = 32;

pub fn setup_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .tooltip("Speech AI Tool")
        .icon(make_circle_icon(&[100, 100, 100, 255]))
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    // Hide to tray on window close instead of quitting
    if let Some(window) = app.get_webview_window("main") {
        let win = window.clone();
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = win.hide();
            }
        });
    }

    Ok(())
}

fn make_circle_icon(color: &[u8; 4]) -> Image<'static> {
    let mut rgba = vec![0u8; (ICON_SIZE * ICON_SIZE * 4) as usize];
    let center = ICON_SIZE as f32 / 2.0;
    let radius = center - 2.0;

    for y in 0..ICON_SIZE {
        for x in 0..ICON_SIZE {
            let dx = x as f32 - center + 0.5;
            let dy = y as f32 - center + 0.5;
            let dist = (dx * dx + dy * dy).sqrt();

            let idx = ((y * ICON_SIZE + x) * 4) as usize;
            if dist <= radius - 1.0 {
                // Fully inside circle
                rgba[idx..idx + 4].copy_from_slice(color);
            } else if dist <= radius {
                // Anti-alias edge
                let alpha = ((radius - dist) * color[3] as f32) as u8;
                rgba[idx] = color[0];
                rgba[idx + 1] = color[1];
                rgba[idx + 2] = color[2];
                rgba[idx + 3] = alpha;
            }
            // else transparent (already 0)
        }
    }

    Image::new_owned(rgba, ICON_SIZE, ICON_SIZE)
}

pub fn set_tray_status(app: &AppHandle, status: &str) {
    let tray_id = TrayIconId::new(TRAY_ID);
    let Some(tray) = app.tray_by_id(&tray_id) else {
        eprintln!("Tray icon not found");
        return;
    };

    let (color, tooltip) = match status {
        "recording" => ([220, 38, 38, 255], "Speech AI Tool - Recording..."),
        "processing" => ([234, 179, 8, 255], "Speech AI Tool - Processing..."),
        "done" => ([34, 197, 94, 255], "Speech AI Tool - Done"),
        _ => ([100, 100, 100, 255], "Speech AI Tool"),
    };

    let _ = tray.set_icon(Some(make_circle_icon(&color)));
    let _ = tray.set_tooltip(Some(tooltip));
}

const OVERLAY_W: f64 = 36.0;
const OVERLAY_H: f64 = 36.0;

pub fn show_overlay(app: &AppHandle) {
    // If the overlay window already exists, just show it
    if let Some(window) = app.get_webview_window("status-overlay") {
        let _ = window.show();
        return;
    }

    let url = tauri::WebviewUrl::App("index.html?window=overlay".into());
    let size = tauri::LogicalSize::new(OVERLAY_W, OVERLAY_H);

    let builder = WebviewWindowBuilder::new(app, "status-overlay", url)
        .title("Status")
        .inner_size(OVERLAY_W, OVERLAY_H)
        .min_inner_size(OVERLAY_W, OVERLAY_H)
        .max_inner_size(OVERLAY_W, OVERLAY_H)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false)
        .resizable(false);

    // Position top-right of primary monitor
    let builder = if let Some(monitor) = app
        .get_webview_window("main")
        .and_then(|w| w.primary_monitor().ok().flatten())
    {
        let scale = monitor.scale_factor();
        let logical_w = monitor.size().width as f64 / scale;
        let x = logical_w - OVERLAY_W - 20.0;
        builder.position(x, 20.0)
    } else {
        builder.position(1200.0, 20.0)
    };

    match builder.build() {
        Ok(window) => {
            // webkitgtk enforces a large minimum widget size by default.
            // Override it via the GTK API so the window can actually be small.
            #[cfg(target_os = "linux")]
            {
                let w = OVERLAY_W as i32;
                let h = OVERLAY_H as i32;
                let _ = window.with_webview(move |wv| {
                    use gtk::prelude::{Cast, GtkWindowExt, WidgetExt};
                    let webview = wv.inner();
                    webview.set_size_request(w, h);
                    // Set type hint so WM treats this as a notification overlay,
                    // not a regular window (prevents snap/tile behavior).
                    if let Some(toplevel) = webview.toplevel() {
                        if let Ok(gtk_win) = toplevel.downcast::<gtk::Window>() {
                            gtk_win.set_type_hint(gdk::WindowTypeHint::Utility);
                        }
                    }
                });
            }

            let _ = window.set_size(size);
            let _ = window.set_min_size(Some(size));
            let _ = window.set_max_size(Some(size));

            let win = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = win.hide();
                }
            });
        }
        Err(e) => eprintln!("Failed to create overlay window: {}", e),
    }
}

pub fn hide_overlay(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("status-overlay") {
        let _ = window.hide();
    }
}
