/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use euclid::{Point2D, Rect, Size2D};
use std::borrow::Cow;

pub fn rgba8_get_rect(pixels: &[u8], size: Size2D<u32>, rect: Rect<u32>) -> Cow<[u8]> {
    assert!(!rect.is_empty());
    assert!(Rect::from_size(size).contains_rect(&rect));
    assert_eq!(pixels.len() % 4, 0);
    assert_eq!(size.area() as usize, pixels.len() / 4);
    let area = rect.size.area() as usize;
    let first_column_start = rect.origin.x as usize * 4;
    let row_length = size.width as usize * 4;
    let first_row_start = rect.origin.y as usize * row_length;
    if rect.origin.x == 0 && rect.size.width == size.width || rect.size.height == 1 {
        let start = first_column_start + first_row_start;
        return Cow::Borrowed(&pixels[start..start + area * 4]);
    }
    let mut data = Vec::with_capacity(area * 4);
    for row in pixels[first_row_start..]
        .chunks(row_length)
        .take(rect.size.height as usize)
    {
        data.extend_from_slice(&row[first_column_start..][..rect.size.width as usize * 4]);
    }
    data.into()
}

// TODO(pcwalton): Speed up with SIMD, or better yet, find some way to not do this.
pub fn rgba8_byte_swap_colors_inplace(pixels: &mut [u8]) {
    assert!(pixels.len() % 4 == 0);
    for rgba in pixels.chunks_mut(4) {
        let b = rgba[0];
        rgba[0] = rgba[2];
        rgba[2] = b;
    }
}

pub fn rgba8_byte_swap_and_premultiply_inplace(pixels: &mut [u8]) {
    assert!(pixels.len() % 4 == 0);
    for rgba in pixels.chunks_mut(4) {
        let b = rgba[0];
        rgba[0] = multiply_u8_color(rgba[2], rgba[3]);
        rgba[1] = multiply_u8_color(rgba[1], rgba[3]);
        rgba[2] = multiply_u8_color(b, rgba[3]);
    }
}

/// Returns true if the pixels were found to be completely opaque.
pub fn rgba8_premultiply_inplace(pixels: &mut [u8]) -> bool {
    assert!(pixels.len() % 4 == 0);
    let mut is_opaque = true;
    for rgba in pixels.chunks_mut(4) {
        rgba[0] = multiply_u8_color(rgba[0], rgba[3]);
        rgba[1] = multiply_u8_color(rgba[1], rgba[3]);
        rgba[2] = multiply_u8_color(rgba[2], rgba[3]);
        is_opaque = is_opaque && rgba[3] == 255;
    }
    is_opaque
}

pub fn multiply_u8_color(a: u8, b: u8) -> u8 {
    return (a as u32 * b as u32 / 255) as u8;
}

pub fn clip(
    mut origin: Point2D<i32>,
    mut size: Size2D<u32>,
    surface: Size2D<u32>,
) -> Option<Rect<u32>> {
    if origin.x < 0 {
        size.width = size.width.saturating_sub(-origin.x as u32);
        origin.x = 0;
    }
    if origin.y < 0 {
        size.height = size.height.saturating_sub(-origin.y as u32);
        origin.y = 0;
    }
    Rect::new(origin.to_u32(), size)
        .intersection(&Rect::from_size(surface))
        .filter(|rect| !rect.is_empty())
}