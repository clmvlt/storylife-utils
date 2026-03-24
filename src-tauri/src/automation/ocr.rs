use image::{ImageBuffer, Rgba};
use windows::core::HSTRING;
use windows::Graphics::Imaging::BitmapPixelFormat;
use windows::Media::Ocr::OcrEngine;

#[derive(Debug, Clone)]
pub struct OcrMatch {
    pub text: String,
    pub confidence: f64,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

// ── Image preprocessing ──

fn invert(img: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut out = img.clone();
    for pixel in out.pixels_mut() {
        pixel[0] = 255 - pixel[0];
        pixel[1] = 255 - pixel[1];
        pixel[2] = 255 - pixel[2];
    }
    out
}

fn adjust_contrast(
    img: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    factor: f64,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut out = img.clone();
    for pixel in out.pixels_mut() {
        for i in 0..3 {
            let val = ((pixel[i] as f64 - 128.0) * factor + 128.0).clamp(0.0, 255.0);
            pixel[i] = val as u8;
        }
    }
    out
}

/// Crop image to a sub-region
fn crop_region(
    img: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let (iw, ih) = img.dimensions();
    let x2 = (x + w).min(iw);
    let y2 = (y + h).min(ih);
    let cw = x2 - x;
    let ch = y2 - y;
    let mut out = ImageBuffer::new(cw, ch);
    for dy in 0..ch {
        for dx in 0..cw {
            out.put_pixel(dx, dy, *img.get_pixel(x + dx, y + dy));
        }
    }
    out
}

// ── Windows OCR ──

/// Result from a single OCR word match
#[derive(Debug, Clone)]
struct WordMatch {
    pub word: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Run OCR and return all individual WORDS with their bounding boxes
fn run_ocr_words(
    img: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    language: &str,
) -> Result<Vec<WordMatch>, String> {
    let (width, height) = img.dimensions();

    let engine = {
        let lang_tag = HSTRING::from(language);
        let lang = windows::Globalization::Language::CreateLanguage(&lang_tag)
            .map_err(|e| format!("Language creation failed: {}", e))?;

        if OcrEngine::IsLanguageSupported(&lang).unwrap_or(false) {
            OcrEngine::TryCreateFromLanguage(&lang)
                .map_err(|e| format!("OCR engine creation failed: {}", e))?
        } else {
            OcrEngine::TryCreateFromUserProfileLanguages()
                .map_err(|e| format!("OCR engine fallback failed: {}", e))?
        }
    };

    let mut bgra_data = Vec::with_capacity((width * height * 4) as usize);
    for pixel in img.pixels() {
        bgra_data.push(pixel[2]);
        bgra_data.push(pixel[1]);
        bgra_data.push(pixel[0]);
        bgra_data.push(pixel[3]);
    }

    let bitmap = windows::Graphics::Imaging::SoftwareBitmap::Create(
        BitmapPixelFormat::Bgra8,
        width as i32,
        height as i32,
    )
    .map_err(|e| format!("SoftwareBitmap creation failed: {}", e))?;

    {
        let buffer = bitmap
            .LockBuffer(windows::Graphics::Imaging::BitmapBufferAccessMode::Write)
            .map_err(|e| format!("LockBuffer failed: {}", e))?;
        let reference = buffer
            .CreateReference()
            .map_err(|e| format!("CreateReference failed: {}", e))?;
        let interop: windows::Win32::System::WinRT::IMemoryBufferByteAccess =
            windows::core::Interface::cast(&reference)
                .map_err(|e| format!("Cast failed: {}", e))?;
        unsafe {
            let mut ptr = std::ptr::null_mut();
            let mut capacity = 0u32;
            interop
                .GetBuffer(&mut ptr, &mut capacity)
                .map_err(|e| format!("GetBuffer failed: {}", e))?;
            let dest = std::slice::from_raw_parts_mut(ptr, capacity as usize);
            let copy_len = dest.len().min(bgra_data.len());
            dest[..copy_len].copy_from_slice(&bgra_data[..copy_len]);
        }
    }

    let result = engine
        .RecognizeAsync(&bitmap)
        .map_err(|e| format!("RecognizeAsync failed: {}", e))?
        .get()
        .map_err(|e| format!("OCR get result failed: {}", e))?;

    let mut words = Vec::new();

    if let Ok(lines) = result.Lines() {
        for line in &lines {
            if let Ok(line_words) = line.Words() {
                for word in &line_words {
                    let text = match word.Text() {
                        Ok(t) => t.to_string(),
                        Err(_) => continue,
                    };
                    if let Ok(rect) = word.BoundingRect() {
                        words.push(WordMatch {
                            word: text,
                            x: rect.X as f64,
                            y: rect.Y as f64,
                            width: rect.Width as f64,
                            height: rect.Height as f64,
                        });
                    }
                }
            }
        }
    }

    Ok(words)
}

// ── Fuzzy matching ──

fn normalize(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn word_similarity(a: &str, b: &str) -> f64 {
    let a = a.to_lowercase();
    let b = b.to_lowercase();
    if a == b {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let len_diff = (a.len() as i32 - b.len() as i32).abs();
    if len_diff > 2 {
        return 0.0;
    }
    // Character-level Jaccard on bigrams
    let ca: Vec<char> = a.chars().collect();
    let cb: Vec<char> = b.chars().collect();
    let ba: std::collections::HashSet<(char, char)> =
        ca.windows(2).map(|w| (w[0], w[1])).collect();
    let bb: std::collections::HashSet<(char, char)> =
        cb.windows(2).map(|w| (w[0], w[1])).collect();
    if ba.is_empty() && bb.is_empty() {
        return if a == b { 1.0 } else { 0.0 };
    }
    let inter = ba.intersection(&bb).count() as f64;
    let union = ba.union(&bb).count() as f64;
    if union == 0.0 { 0.0 } else { inter / union }
}

/// Find consecutive OCR words that match the character name words.
/// Returns the combined bounding box of the matched word sequence.
fn find_name_in_words(
    ocr_words: &[WordMatch],
    name_words: &[&str],
) -> Option<(f64, OcrMatch)> {
    if name_words.is_empty() || ocr_words.is_empty() {
        return None;
    }

    let n = name_words.len();

    // Sliding window: try to find n consecutive OCR words matching the name words
    for start in 0..ocr_words.len() {
        if start + n > ocr_words.len() {
            break;
        }

        let mut total_score = 0.0;
        let mut all_match = true;

        for i in 0..n {
            let sim = word_similarity(&ocr_words[start + i].word, name_words[i]);
            if sim < 0.4 {
                all_match = false;
                break;
            }
            total_score += sim;
        }

        if all_match {
            let avg_score = total_score / n as f64;

            // Compute combined bounding box from the matched words
            let matched = &ocr_words[start..start + n];
            let min_x = matched.iter().map(|w| w.x).fold(f64::MAX, f64::min);
            let min_y = matched.iter().map(|w| w.y).fold(f64::MAX, f64::min);
            let max_x = matched
                .iter()
                .map(|w| w.x + w.width)
                .fold(0.0f64, f64::max);
            let max_y = matched
                .iter()
                .map(|w| w.y + w.height)
                .fold(0.0f64, f64::max);

            let text = matched
                .iter()
                .map(|w| w.word.as_str())
                .collect::<Vec<_>>()
                .join(" ");

            return Some((
                avg_score,
                OcrMatch {
                    text,
                    confidence: avg_score,
                    x: min_x,
                    y: min_y,
                    width: max_x - min_x,
                    height: max_y - min_y,
                },
            ));
        }
    }

    None
}

/// Run one OCR pass on an image and try to find the character name.
fn try_ocr_pass(
    img: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    name_words: &[&str],
    label: &str,
) -> Option<(f64, OcrMatch)> {
    let languages = ["fr-FR", "en-US"];
    let mut best: Option<(f64, OcrMatch)> = None;

    for lang in &languages {
        match run_ocr_words(img, lang) {
            Ok(words) => {
                log::debug!(
                    "OCR [{}][{}] found {} words: [{}]",
                    label,
                    lang,
                    words.len(),
                    words.iter().map(|w| w.word.as_str()).collect::<Vec<_>>().join(", ")
                );

                if let Some((score, m)) = find_name_in_words(&words, name_words) {
                    log::debug!(
                        "OCR [{}][{}] matched '{}' score={:.2} at ({:.0},{:.0})",
                        label,
                        lang,
                        m.text,
                        score,
                        m.x,
                        m.y
                    );
                    if best.as_ref().map_or(true, |(b, _)| score > *b) {
                        best = Some((score, m));
                    }
                }
            }
            Err(e) => {
                log::warn!("OCR [{}][{}] failed: {}", label, lang, e);
            }
        }
    }

    best
}

fn median(values: &mut [f64]) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = values.len();
    if n == 0 {
        return 0.0;
    }
    if n % 2 == 0 {
        (values[n / 2 - 1] + values[n / 2]) / 2.0
    } else {
        values[n / 2]
    }
}

/// Main OCR function.
///
/// Strategy:
/// 1. Crop the image to the character slot area (left 30%, top 35%)
///    — the selection UI is always top-left in FiveM
/// 2. Use WORD-level matching (not line-level) for precise bounding boxes
/// 3. Try original image first, then inverted (white text on dark = game typical)
/// 4. Stabilize with 5 passes + median position
pub fn find_character_in_image(
    img: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    character_name: &str,
) -> Option<OcrMatch> {
    let normalized_name = normalize(character_name);
    let name_words: Vec<&str> = normalized_name.split_whitespace().collect();

    if name_words.is_empty() {
        return None;
    }

    let (full_w, full_h) = img.dimensions();

    // Crop to left 30% and top 35% — where the character selection slots are
    let crop_w = (full_w as f64 * 0.30) as u32;
    let crop_h = (full_h as f64 * 0.35) as u32;
    let cropped = crop_region(img, 0, 0, crop_w, crop_h);

    log::info!(
        "OCR: cropped to {}x{} (from {}x{}), searching for {:?}",
        crop_w,
        crop_h,
        full_w,
        full_h,
        name_words
    );

    // Build image variants to try (in order of preference)
    let variants: Vec<(&str, ImageBuffer<Rgba<u8>, Vec<u8>>)> = vec![
        ("original", cropped.clone()),
        ("inverted", invert(&cropped)),
        ("contrast_2x", adjust_contrast(&cropped, 2.0)),
        ("inverted_contrast", adjust_contrast(&invert(&cropped), 2.0)),
    ];

    // ── Step 1: Find which variant detects the name ──
    let mut winning_variant: Option<(usize, f64, OcrMatch)> = None;

    for (idx, (name, variant_img)) in variants.iter().enumerate() {
        if let Some((score, m)) = try_ocr_pass(variant_img, &name_words, name) {
            if score >= 0.5 {
                log::info!(
                    "OCR found via '{}': '{}' score={:.2} at ({:.0},{:.0})",
                    name,
                    m.text,
                    score,
                    m.x,
                    m.y
                );
                winning_variant = Some((idx, score, m));
                break; // Use first variant that works well
            }
        }
    }

    let (variant_idx, _score, _first_match) = winning_variant?;

    // ── Step 2: Stabilize position with 5 passes on the SAME variant ──
    let ref_img = &variants[variant_idx].1;
    let ref_name = variants[variant_idx].0;
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    let mut ws = Vec::new();
    let mut hs = Vec::new();
    let mut best_text = String::new();

    for pass in 0..5 {
        if let Some((s, m)) = try_ocr_pass(ref_img, &name_words, &format!("{}_p{}", ref_name, pass))
        {
            if s >= 0.4 {
                if best_text.is_empty() {
                    best_text = m.text.clone();
                }
                xs.push(m.x);
                ys.push(m.y);
                ws.push(m.width);
                hs.push(m.height);
            }
        }
    }

    if xs.is_empty() {
        log::warn!("OCR stabilization: no consistent results");
        return None;
    }

    // Coordinates are relative to the CROP — no offset needed since we cropped from (0,0)
    let result = OcrMatch {
        text: best_text,
        confidence: 1.0,
        x: median(&mut xs),
        y: median(&mut ys),
        width: median(&mut ws),
        height: median(&mut hs),
    };

    log::info!(
        "OCR final: '{}' at ({:.0},{:.0}) {}x{:.0} — {} samples, crop offset (0,0)",
        result.text,
        result.x,
        result.y,
        result.width as i32,
        result.height,
        xs.len()
    );

    Some(result)
}
