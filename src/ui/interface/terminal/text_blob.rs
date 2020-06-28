use std::iter::{once, Iterator};

use crate::helper::split_exhaustive;
use crate::ui::TextStyle;

pub struct TextBlob {
    pub text: String,
    pub style: TextStyle,
    pub break_points: Vec<BreakPoint>,
}

impl TextBlob {
    pub fn from_string(text: &str, style: TextStyle) -> Vec<TextBlob> {
        split_exhaustive(&text, '\n')
            .map(|v| TextBlob {
                text: v.to_owned(),
                style: style.clone(),
                break_points: Vec::new(),
            })
            .collect()
    }
}

/// Set the "wrap points" in text blobs, which are used by the
/// `print_blob` method to determine where to wrap lines.
pub fn wrap_blobs(blobs: &mut [TextBlob], width: usize, mut offset: usize) {
    let mut last_possible_breakpoint: Option<(usize, BreakPoint)> = None;
    for blob in blobs.iter_mut() {
        blob.break_points.clear();
    }
    for i in 0..blobs.len() {
        let blob = &blobs[i];
        if blob.text == "\n" {
            offset = 0;
            last_possible_breakpoint = None;
            continue;
        }
        let break_points: Vec<usize> = once(0)
            .chain(
                blob.text
                    .match_indices(' ')
                    .map(|x| vec![x.0, x.0 + x.1.len()].into_iter())
                    .flatten()
                    .chain(once(blob.text.len())),
            )
            .collect();
        for point in break_points.windows(2) {
            let start = point[0];
            let end = point[1];

            let len = blobs[i].text[start..end].chars().count();
            if offset + len <= width {
                offset += len;
            } else if let Some((blob_index, breakpoint)) = &last_possible_breakpoint {
                let len = if i == *blob_index {
                    blobs[i].text[breakpoint.byte_index..end].chars().count()
                } else {
                    blobs[*blob_index].text[breakpoint.byte_index..]
                        .chars()
                        .count()
                        + blobs[*blob_index..i]
                            .iter()
                            .skip(1)
                            .fold(0, |acc, cur| acc + cur.text.chars().count())
                        + blobs[i].text[..end].chars().count()
                };
                blobs[*blob_index].break_points.push(breakpoint.clone());
                last_possible_breakpoint = None;
                if len <= width {
                    offset = len;
                } else {
                    //TODO
                }
            }
            if &blobs[i].text[start..end] == " " {
                last_possible_breakpoint = Some((i, BreakPoint { byte_index: start }));
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct BreakPoint {
    pub byte_index: usize,
}
