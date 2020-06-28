use std::iter::from_fn;

/// Split a string, including the separator as an element in the result.
pub fn split_exhaustive<'a>(input: &'a str, separator: char) -> impl Iterator<Item = &'a str> {
    let mut indices = input.match_indices(|x| x == '\n' || x == ' ');
    let mut last = 0;
    from_fn(move || {
        let mut results = Vec::with_capacity(2);
        if let Some((index, item)) = indices.next() {
            if last < index {
                results.push(&input[last..index])
            }
            results.push(item);
            last = index + 1;
        } else if last < input.len() {
            results.push(&input[last..]);
            last = input.len();
        }
        if results.is_empty() {
            None
        } else {
            Some(results.into_iter())
        }
    })
    .flatten()
}
