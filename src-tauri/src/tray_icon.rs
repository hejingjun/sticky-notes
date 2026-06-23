/// System tray icon — a 32×32 procedural icon with no external file dependency.
///
/// Design: a bold rounded square with a white checkmark (✓), inspired by
/// the todo/sticky-note nature of the app. Vibrant orange background,
/// fully opaque, high contrast — readable at 16×16 system tray scale.

pub fn tray_icon_image() -> tauri::image::Image<'static> {
    const W: usize = 32;
    let mut rgba = vec![0u8; W * W * 4];

    // Background: rounded orange square (full opacity)
    let bg_r = 255u8;
    let bg_g = 155u8;
    let bg_b = 55u8;

    for y in 0..W {
        for x in 0..W {
            let idx = (y * W + x) * 4;

            // Rounded corners: clip corners with a radius of ~5px
            let in_rect = x >= 2 && x < W - 2 && y >= 1 && y < W - 1;
            // Corner radius: distance from corner center > 5 → outside
            let corner_ok = {
                let cx = if x < W / 2 { 4 } else { W - 5 } as i32;
                let cy = if y < W / 2 { 3 } else { W - 4 } as i32;
                let dx = x as i32 - cx;
                let dy = y as i32 - cy;
                let dist2 = dx * dx + dy * dy;
                // Only apply corner test near the four corners
                let near_corner = (x < 6 || x > W - 7) && (y < 5 || y > W - 6);
                if near_corner {
                    dist2 <= 16 // radius ~4px
                } else {
                    true
                }
            };

            if in_rect && corner_ok {
                rgba[idx] = bg_r;
                rgba[idx + 1] = bg_g;
                rgba[idx + 2] = bg_b;
                rgba[idx + 3] = 255;
            } else {
                // Fully transparent outside the rounded rect
                rgba[idx] = 0;
                rgba[idx + 1] = 0;
                rgba[idx + 2] = 0;
                rgba[idx + 3] = 0;
            }
        }
    }

    // White checkmark (✓) — bold pixel art, 12×12 area centered
    // Checkmark path designed on a 0..11 grid
    let check: &[(usize, usize)] = &[
        // stem going down-right
        (3, 3), (3, 4), (2, 5), (2, 6), (1, 7), (1, 8),
        // hook going up-right
        (2, 8), (3, 8), (4, 7), (5, 6), (6, 5),
        // thicken
        (4, 3), (4, 4),
        (3, 5),
        (2, 7),
        (3, 7),
        (4, 6), (5, 5),
        // more thickness
        (5, 3), (5, 4),
        (4, 5),
        (3, 6),
        (5, 7), (6, 6), (6, 7),
        // right side
        (6, 3), (6, 4), (7, 4),
        (7, 5), (8, 5),
        (7, 6),
    ];

    let cx = 10; // center x offset
    let cy = 10; // center y offset
    for &(px, py) in check.iter() {
        let x = cx + px;
        let y = cy + py;
        if x < W - 3 && y < W - 3 {
            let idx = (y * W + x) * 4;
            rgba[idx] = 255;
            rgba[idx + 1] = 255;
            rgba[idx + 2] = 255;
            rgba[idx + 3] = 255;
        }
    }

    tauri::image::Image::new_owned(rgba, W as u32, W as u32)
}
