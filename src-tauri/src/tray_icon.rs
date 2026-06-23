/// System tray icon — a 32×32 sticky-note icon generated at compile time.
/// No external file dependency; the icon is a procedural RGBA pixel array.
///
/// Design: a yellow sticky note with a subtle folded corner (top-right),
///        thin darker border, and a faint center-line hint.

/// Returns a 32×32 tray icon image as a hardcoded procedural RGBA bitmap.
pub fn tray_icon_image() -> tauri::image::Image<'static> {
    let mut rgba = vec![0u8; 32 * 32 * 4];
    let note_color: [u8; 3] = [255, 235, 120];   // soft yellow
    let shadow_color: [u8; 3] = [200, 175, 60];   // darker for fold/border
    let fold_color: [u8; 3] = [230, 210, 90];     // lighter fold
    let bg_alpha: u8 = 0;

    // Fill entire region transparent first
    for y in 0..32 {
        for x in 0..32 {
            let idx = (y * 32 + x) * 4;
            let inside = x >= 2 && x < 30 && y >= 1 && y < 30;
            // Fold corner mask (top-right triangle)
            let fold = x + y >= 33 && x >= 18;
            if !inside {
                rgba[idx] = 0;
                rgba[idx + 1] = 0;
                rgba[idx + 2] = 0;
                rgba[idx + 3] = bg_alpha;
                continue;
            }
            // Border (outer 1px of the note body, rounded-rect approximation)
            let border = x == 2 || x == 29 || y == 1 || y == 29;
            // Skip border on fold edge
            let skip_border = (x + y >= 30 && x >= 18) || (x + y >= 31 && x >= 19);

            if fold {
                rgba[idx] = fold_color[0];
                rgba[idx + 1] = fold_color[1];
                rgba[idx + 2] = fold_color[2];
                rgba[idx + 3] = 255;
            } else if border && !skip_border {
                rgba[idx] = shadow_color[0];
                rgba[idx + 1] = shadow_color[1];
                rgba[idx + 2] = shadow_color[2];
                rgba[idx + 3] = 255;
            } else {
                rgba[idx] = note_color[0];
                rgba[idx + 1] = note_color[1];
                rgba[idx + 2] = note_color[2];
                rgba[idx + 3] = 255;
            }
        }
    }

    // Draw a subtle horizontal line in the center to suggest ruled paper
    for x in 6..28 {
        let y = 16;
        let idx = (y * 32 + x) * 4;
        if x % 3 != 0 {
            // skip fold area
            let fold = x + y >= 33 && x >= 18;
            if !fold {
                rgba[idx] = shadow_color[0];
                rgba[idx + 1] = shadow_color[1];
                rgba[idx + 2] = shadow_color[2];
                rgba[idx + 3] = 80;
            }
        }
    }

    // Write a small "S" glyph (initials for Sticky) on the upper-left
    // Simple pixel font "S" at offset (8, 8), 5×7 glyph
    let s_glyph: [[bool; 5]; 7] = [
        [false, true, true, true, false],
        [true, false, false, false, false],
        [true, false, false, false, false],
        [false, true, true, false, false],
        [false, false, false, true, false],
        [false, false, false, true, false],
        [true, true, true, false, false],
    ];
    let glyph_x = 8;
    let glyph_y = 8;
    for gy in 0..7 {
        for gx in 0..5 {
            if s_glyph[gy][gx] {
                let px = glyph_x + gx;
                let py = glyph_y + gy;
                if px < 30 && py < 30 {
                    let idx = (py * 32 + px) * 4;
                    // Darker text on yellow
                    rgba[idx] = 140;
                    rgba[idx + 1] = 100;
                    rgba[idx + 2] = 30;
                    rgba[idx + 3] = 180;
                }
            }
        }
    }

    tauri::image::Image::new_owned(rgba, 32, 32)
}
