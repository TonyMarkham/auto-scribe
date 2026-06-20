use std::sync::{Arc, LazyLock};

#[cfg(target_os = "linux")]
use std::{env, fs, io, path::PathBuf};

#[cfg(target_os = "linux")]
use crate::error::{AppError, AppResult, ResultContext};
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage, imageops::FilterType};

pub(crate) const APP_ID: &str = "dev.gpui.AutoScribe";

const ICON_SIZE: u32 = 64;
#[cfg(target_os = "linux")]
const DESKTOP_ICON_SIZE: u32 = 512;
#[cfg(target_os = "linux")]
const DESKTOP_FILE_NAME: &str = "dev.gpui.AutoScribe.desktop";
#[cfg(target_os = "linux")]
const DESKTOP_ENTRY: &str = include_str!("../data/dev.gpui.AutoScribe.desktop");
#[cfg(target_os = "linux")]
const PNG_ICON_FILE_NAME: &str = "dev.gpui.AutoScribe.png";

static WINDOW_ICON: LazyLock<Arc<RgbaImage>> = LazyLock::new(|| Arc::new(build_window_icon()));

pub(crate) fn window_icon() -> Arc<RgbaImage> {
    WINDOW_ICON.clone()
}

#[cfg(target_os = "linux")]
pub(crate) fn ensure_desktop_entry() -> AppResult<PathBuf> {
    let icon_path = ensure_desktop_icon()?;

    let applications_dir = user_applications_dir()?;
    let desktop_entry_path = applications_dir.join(DESKTOP_FILE_NAME);
    let desktop_entry = DESKTOP_ENTRY
        .replace(
            "Exec=auto-scribe",
            &format!("Exec={}", desktop_exec_value()?),
        )
        .replace(
            "Icon=dev.gpui.AutoScribe",
            &format!("Icon={}", icon_path.to_string_lossy()),
        );

    match fs::read_to_string(&desktop_entry_path) {
        Ok(existing_entry) if existing_entry == desktop_entry => return Ok(desktop_entry_path),
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(AppError::with_context(
                format!("read desktop entry {}", desktop_entry_path.display()),
                error,
            ));
        }
    }

    fs::create_dir_all(&applications_dir)
        .with_context(|| format!("create {}", applications_dir.display()))?;
    fs::write(&desktop_entry_path, desktop_entry)
        .with_context(|| format!("write {}", desktop_entry_path.display()))?;

    Ok(desktop_entry_path)
}

#[cfg(target_os = "linux")]
fn ensure_desktop_icon() -> AppResult<PathBuf> {
    let icons_dir = user_data_dir()?.join("icons/hicolor/512x512/apps");
    let icon_path = icons_dir.join(PNG_ICON_FILE_NAME);
    let icon_png = desktop_icon_png()?;

    match fs::read(&icon_path) {
        Ok(existing_icon) if existing_icon == icon_png => return Ok(icon_path),
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(AppError::with_context(
                format!("read icon {}", icon_path.display()),
                error,
            ));
        }
    }

    fs::create_dir_all(&icons_dir).with_context(|| format!("create {}", icons_dir.display()))?;
    fs::write(&icon_path, icon_png).with_context(|| format!("write {}", icon_path.display()))?;

    Ok(icon_path)
}

#[cfg(target_os = "linux")]
fn desktop_icon_png() -> AppResult<Vec<u8>> {
    let icon = DynamicImage::ImageRgba8(build_window_icon()).resize_exact(
        DESKTOP_ICON_SIZE,
        DESKTOP_ICON_SIZE,
        FilterType::Lanczos3,
    );
    let mut bytes = std::io::Cursor::new(Vec::new());

    icon.write_to(&mut bytes, ImageFormat::Png)
        .context("encode desktop icon PNG")?;

    Ok(bytes.into_inner())
}

#[cfg(target_os = "linux")]
fn user_applications_dir() -> AppResult<PathBuf> {
    Ok(user_data_dir()?.join("applications"))
}

#[cfg(target_os = "linux")]
fn user_data_dir() -> AppResult<PathBuf> {
    if let Some(data_home) = env::var_os("XDG_DATA_HOME")
        && !data_home.as_os_str().is_empty()
    {
        return Ok(PathBuf::from(data_home));
    }

    let Some(home) = env::var_os("HOME") else {
        return Err(AppError::operation(
            "resolve HOME for XDG desktop file installation",
        ));
    };
    Ok(PathBuf::from(home).join(".local/share"))
}

#[cfg(target_os = "linux")]
fn desktop_exec_value() -> AppResult<String> {
    let executable =
        env::current_exe().context("resolve current executable for XDG desktop entry")?;
    let executable = executable.to_string_lossy();
    let escaped = executable.replace('\\', "\\\\").replace('"', "\\\"");

    Ok(format!("\"{escaped}\""))
}

fn build_window_icon() -> RgbaImage {
    let mut icon = RgbaImage::from_pixel(ICON_SIZE, ICON_SIZE, Rgba([0, 0, 0, 0]));

    draw_background(&mut icon);
    draw_keycap(&mut icon, 12, 17, 17, 17, Rgba([232, 249, 255, 255]));
    draw_keycap(&mut icon, 35, 17, 17, 17, Rgba([214, 245, 255, 255]));
    draw_keycap(&mut icon, 14, 38, 36, 10, Rgba([241, 252, 255, 255]));
    draw_plus(&mut icon, 20, 25, Rgba([12, 74, 110, 255]));
    draw_dot(&mut icon, 43, 25, 3, Rgba([8, 145, 178, 255]));
    draw_hold_bar(&mut icon, 21, 42, 22, Rgba([14, 116, 144, 255]));

    icon
}

fn draw_background(icon: &mut RgbaImage) {
    for y in 0..ICON_SIZE {
        for x in 0..ICON_SIZE {
            if !inside_rounded_rect(x, y, 3, 3, 58, 58, 14) {
                continue;
            }

            let progress = y as f32 / (ICON_SIZE - 1) as f32;
            let r = lerp(30, 15, progress);
            let g = lerp(64, 23, progress);
            let b = lerp(175, 42, progress);
            icon.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }

    stroke_rounded_rect(icon, 3, 3, 58, 58, 14, Rgba([103, 232, 249, 180]));
}

fn draw_keycap(icon: &mut RgbaImage, x: u32, y: u32, width: u32, height: u32, color: Rgba<u8>) {
    fill_rounded_rect(icon, x + 1, y + 2, width, height, 5, Rgba([8, 23, 42, 90]));
    fill_rounded_rect(icon, x, y, width, height, 5, color);
    stroke_rounded_rect(icon, x, y, width, height, 5, Rgba([14, 116, 144, 160]));
}

fn draw_plus(icon: &mut RgbaImage, center_x: u32, center_y: u32, color: Rgba<u8>) {
    fill_rect(icon, center_x - 1, center_y - 5, 3, 11, color);
    fill_rect(icon, center_x - 5, center_y - 1, 11, 3, color);
}

fn draw_dot(icon: &mut RgbaImage, center_x: u32, center_y: u32, radius: u32, color: Rgba<u8>) {
    let radius_squared = (radius * radius) as i64;
    for y in center_y - radius..=center_y + radius {
        for x in center_x - radius..=center_x + radius {
            let dx = x as i64 - center_x as i64;
            let dy = y as i64 - center_y as i64;
            if dx * dx + dy * dy <= radius_squared {
                icon.put_pixel(x, y, color);
            }
        }
    }
}

fn draw_hold_bar(icon: &mut RgbaImage, x: u32, y: u32, width: u32, color: Rgba<u8>) {
    fill_rounded_rect(icon, x, y, width, 3, 2, color);
}

fn fill_rounded_rect(
    icon: &mut RgbaImage,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    radius: u32,
    color: Rgba<u8>,
) {
    for py in y..y + height {
        for px in x..x + width {
            if inside_rounded_rect(px, py, x, y, width, height, radius) {
                icon.put_pixel(px, py, color);
            }
        }
    }
}

fn stroke_rounded_rect(
    icon: &mut RgbaImage,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    radius: u32,
    color: Rgba<u8>,
) {
    for px in x..x + width {
        if inside_rounded_rect(px, y, x, y, width, height, radius) {
            icon.put_pixel(px, y, color);
        }
        if inside_rounded_rect(px, y + height - 1, x, y, width, height, radius) {
            icon.put_pixel(px, y + height - 1, color);
        }
    }

    for py in y..y + height {
        if inside_rounded_rect(x, py, x, y, width, height, radius) {
            icon.put_pixel(x, py, color);
        }
        if inside_rounded_rect(x + width - 1, py, x, y, width, height, radius) {
            icon.put_pixel(x + width - 1, py, color);
        }
    }
}

fn fill_rect(icon: &mut RgbaImage, x: u32, y: u32, width: u32, height: u32, color: Rgba<u8>) {
    for py in y..y + height {
        for px in x..x + width {
            icon.put_pixel(px, py, color);
        }
    }
}

fn inside_rounded_rect(
    px: u32,
    py: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    radius: u32,
) -> bool {
    let left = x + radius;
    let right = x + width - radius - 1;
    let top = y + radius;
    let bottom = y + height - radius - 1;

    if (left..=right).contains(&px) || (top..=bottom).contains(&py) {
        return true;
    }

    let corner_x = if px < left { left } else { right };
    let corner_y = if py < top { top } else { bottom };
    let dx = px as i64 - corner_x as i64;
    let dy = py as i64 - corner_y as i64;

    dx * dx + dy * dy <= (radius as i64 * radius as i64)
}

fn lerp(start: u8, end: u8, progress: f32) -> u8 {
    (start as f32 + (end as f32 - start as f32) * progress).round() as u8
}
