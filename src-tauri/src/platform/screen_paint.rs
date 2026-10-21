use dictation_core::defaults;
use windows::{
    core::w,
    Win32::{
        Foundation::{COLORREF, HWND, POINT, RECT, SIZE},
        Graphics::Gdi::*,
        UI::WindowsAndMessaging::{UpdateLayeredWindow, ULW_ALPHA},
    },
};

#[derive(Clone, Copy)]
pub(super) enum Shape {
    Top,
    Bottom,
    Left,
    Right,
    Label,
}

struct Bitmap {
    dc: HDC,
    bitmap: HBITMAP,
    previous: HGDIOBJ,
    bits: *mut u32,
    width: i32,
    height: i32,
    length: usize,
}

impl Bitmap {
    unsafe fn new(width: i32, height: i32) -> Result<Self, String> {
        let length = usize::try_from(width)
            .ok()
            .zip(usize::try_from(height).ok())
            .and_then(|(w, h)| w.checked_mul(h))
            .filter(|length| *length > 0 && *length <= isize::MAX as usize / 4)
            .ok_or("Invalid screen overlay dimensions.")?;
        let dc = CreateCompatibleDC(None);
        if dc.is_invalid() {
            return Err("Cannot allocate the overlay drawing context.".into());
        }
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits = std::ptr::null_mut();
        let bitmap = match CreateDIBSection(Some(dc), &info, DIB_RGB_COLORS, &mut bits, None, 0) {
            Ok(bitmap) => bitmap,
            Err(_) => {
                let _ = DeleteDC(dc);
                return Err("Cannot allocate the overlay bitmap.".into());
            }
        };
        let previous = SelectObject(dc, bitmap.into());
        if previous.is_invalid() {
            let _ = DeleteObject(bitmap.into());
            let _ = DeleteDC(dc);
            return Err("Cannot initialize the overlay bitmap.".into());
        }
        if bits.is_null() {
            SelectObject(dc, previous);
            let _ = DeleteObject(bitmap.into());
            let _ = DeleteDC(dc);
            return Err("Cannot access the overlay bitmap.".into());
        }
        std::ptr::write_bytes(bits.cast::<u8>(), 0, length * 4);
        Ok(Self {
            dc,
            bitmap,
            previous,
            bits: bits.cast(),
            width,
            height,
            length,
        })
    }

    unsafe fn text(&self, label: &str, font_size: i32) -> Result<(), String> {
        let font = CreateFontW(
            -font_size,
            0,
            0,
            0,
            500,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            ANTIALIASED_QUALITY,
            0,
            w!("Segoe UI"),
        );
        if font.is_invalid() {
            return Err("Cannot create the screen overlay font.".into());
        }
        let previous = SelectObject(self.dc, font.into());
        if previous.is_invalid() {
            let _ = DeleteObject(font.into());
            return Err("Cannot select the screen overlay font.".into());
        }
        SetBkMode(self.dc, TRANSPARENT);
        SetTextColor(self.dc, COLORREF(0x00ffffff));
        let mut bounds = RECT {
            left: self.height / 2,
            top: 0,
            right: self.width - self.height / 2,
            bottom: self.height,
        };
        let mut text: Vec<_> = label.encode_utf16().collect();
        let drawn = DrawTextW(
            self.dc,
            &mut text,
            &mut bounds,
            DT_SINGLELINE | DT_CENTER | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX,
        );
        SelectObject(self.dc, previous);
        let _ = DeleteObject(font.into());
        // GDI must finish writing the DIB before its pixels are read for alpha composition.
        let flushed = GdiFlush().as_bool();
        (drawn != 0 && flushed)
            .then_some(())
            .ok_or("Cannot draw the screen overlay label.".into())
    }
}

impl Drop for Bitmap {
    fn drop(&mut self) {
        // SAFETY: these resources belong to this thread; restore the selected bitmap before deletion.
        unsafe {
            SelectObject(self.dc, self.previous);
            let _ = DeleteObject(self.bitmap.into());
            let _ = DeleteDC(self.dc);
        }
    }
}

fn pixel(color: [u8; 3], alpha: u32) -> u32 {
    let [red, green, blue] = color.map(|channel| channel as u32 * alpha / 255);
    alpha << 24 | red << 16 | green << 8 | blue
}

fn label_pixel(mask: u32, x: i32, y: i32, width: i32, height: i32, color: [u8; 3]) -> u32 {
    let radius = height as f32 / 2.0;
    let center_x = (x as f32 + 0.5).clamp(radius, (width as f32 - radius).max(radius));
    let distance = ((x as f32 + 0.5 - center_x).powi(2) + (y as f32 + 0.5 - radius).powi(2)).sqrt();
    let coverage = (radius - distance).clamp(0.0, 1.0);
    let alpha = (coverage * defaults::SCREEN_LABEL_ALPHA as f32) as u32;
    let background = if distance > radius - 1.5 {
        color
    } else {
        defaults::SCREEN_LABEL_BACKGROUND
    };
    let base = pixel(background, alpha);
    let text_alpha = (mask & 0xff).min((coverage * 255.0) as u32);
    let blend = |shift: u32| -> u32 {
        let channel = (base >> shift) & 0xff;
        text_alpha + channel * (255 - text_alpha) / 255
    };
    let alpha = text_alpha + alpha * (255 - text_alpha) / 255;
    alpha << 24 | blend(16) << 16 | blend(8) << 8 | blend(0)
}

pub(super) unsafe fn draw(
    window: HWND,
    bounds: RECT,
    shape: Shape,
    color: [u8; 3],
    label: &str,
    font_size: i32,
) -> Result<(), String> {
    // SAFETY: the checked top-down 32-bit DIB is exclusively accessed on its owning thread.
    let width = bounds.right - bounds.left;
    let height = bounds.bottom - bounds.top;
    let bitmap = Bitmap::new(width, height)?;
    if matches!(shape, Shape::Label) {
        bitmap.text(label, font_size)?;
    }
    let pixels = std::slice::from_raw_parts_mut(bitmap.bits, bitmap.length);
    for (index, target) in pixels.iter_mut().enumerate() {
        let x = (index % width as usize) as i32;
        let y = (index / width as usize) as i32;
        let (depth, thickness) = match shape {
            Shape::Top => (y.min(x).min(width - 1 - x), height),
            Shape::Bottom => ((height - 1 - y).min(x).min(width - 1 - x), height),
            Shape::Left => (x, width),
            Shape::Right => (width - 1 - x, width),
            Shape::Label => {
                *target = label_pixel(*target, x, y, width, height, color);
                continue;
            }
        };
        let strength = 1.0 - depth as f32 / thickness as f32;
        *target = pixel(
            color,
            (strength * strength * defaults::SCREEN_GLOW_ALPHA as f32) as u32,
        );
    }
    let blend = BLENDFUNCTION {
        BlendOp: AC_SRC_OVER as u8,
        SourceConstantAlpha: 255,
        AlphaFormat: AC_SRC_ALPHA as u8,
        ..Default::default()
    };
    UpdateLayeredWindow(
        window,
        None,
        Some(&POINT { x: bounds.left, y: bounds.top }),
        Some(&SIZE { cx: width, cy: height }),
        Some(bitmap.dc),
        Some(&POINT::default()),
        COLORREF(0),
        Some(&blend),
        ULW_ALPHA,
    )
    .map_err(|_| "Cannot compose the screen overlay.".into())
}
