//  Copyright (C) 2019  Éloïs SANCHEZ
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Common test tools for collections

use std::collections::HashMap;
use std::hash::Hash;

/// Returns true if both slices contain the same elements but not necessarily in the same order
pub fn slice_same_elems<T: Hash + Eq + Clone>(a: &[T], b: &[T]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut last_pos_elements: HashMap<T, usize> = HashMap::with_capacity(a.len());

    for e in a {
        let last_pos: &mut usize = last_pos_elements.entry(e.clone()).or_insert(0);
        let begin = if *last_pos > 0 { 1 } else { 0 };
        if let Some(pos) = find_element_in_slice(&b[(*last_pos + begin)..], e) {
            *last_pos += pos + begin;
        } else {
            return false;
        }
    }

    true
}

fn find_element_in_slice<T: Hash + Eq>(s: &[T], e: &T) -> Option<usize> {
    for i in 0..s.len() {
        if s[i] == *e {
            return Some(i);
        }
    }

    None
}

#[cfg(test)]
mod test {
    
    use super::*;

    #[test]
    fn test_find_element_in_slice() {
        let s1 = [0, 1, 2, 3];

        assert_eq!(Some(2), find_element_in_slice(&s1, &2));

        let s2 = [0, 1, 2, 1, 1, 1, 2];

        assert_eq!(Some(3), find_element_in_slice(&s2[3..], &2));
    }

    #[test]
    fn test_slice_same_elems() {
        assert!(slice_same_elems(&[1, 2, 2, 3, 3, 3], &[3, 2, 1, 2, 3, 3]));
        assert!(slice_same_elems(&[4, 1, 4, 4, 4], &[1, 4, 4, 4, 4]));
        assert!(slice_same_elems(&[1, 4, 4, 4, 4], &[4, 4, 4, 4, 1]));
    }
}
