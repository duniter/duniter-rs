//  Copyright (C) 2017-2019  The AXIOM TEAM Association.
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

// ! Paging input methods

pub use crate::schema::Paging;

use crate::constants::*;
use std::ops::Range;

#[derive(Debug, PartialEq)]
pub struct FilledPaging {
    pub page_number: isize,
    pub page_size: usize,
}

impl Default for FilledPaging {
    fn default() -> Self {
        FilledPaging {
            page_number: DEFAULT_PAGE_NUMBER,
            page_size: DEFAULT_PAGE_SIZE,
        }
    }
}

impl From<Option<Paging>> for FilledPaging {
    fn from(paging_opt: Option<Paging>) -> Self {
        if let Some(paging) = paging_opt {
            FilledPaging {
                page_number: paging.page_number.unwrap_or(DEFAULT_PAGE_NUMBER_I32) as isize,
                page_size: if let Some(page_size) = paging.page_size {
                    if page_size < MIN_PAGE_SIZE {
                        MIN_PAGE_SIZE as usize
                    } else if page_size > MAX_PAGE_SIZE {
                        MAX_PAGE_SIZE as usize
                    } else {
                        page_size as usize
                    }
                } else {
                    DEFAULT_PAGE_SIZE
                },
            }
        } else {
            FilledPaging::default()
        }
    }
}

impl FilledPaging {
    pub(crate) fn get_page_range(&self, count_elems: usize, step: usize) -> (Range<usize>, usize) {
        let page_extended_size = self.page_size * step;
        let mut count_pages = count_elems / page_extended_size;
        if count_elems % page_extended_size > 0 {
            count_pages += 1;
        }
        let page_number = if self.page_number.is_negative() {
            std::cmp::max(0, count_pages as isize - self.page_number.abs()) as usize
        } else {
            self.page_number as usize
        };

        (
            Range {
                start: std::cmp::min(count_elems, page_number * page_extended_size),
                end: std::cmp::min(count_elems, (page_number + 1) * page_extended_size),
            },
            count_pages,
        )
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_default() {
        assert_eq!(
            FilledPaging {
                page_number: DEFAULT_PAGE_NUMBER,
                page_size: DEFAULT_PAGE_SIZE,
            },
            FilledPaging::default(),
        )
    }

    #[test]
    fn test_from_none_paging() {
        assert_eq!(
            FilledPaging {
                page_number: DEFAULT_PAGE_NUMBER,
                page_size: DEFAULT_PAGE_SIZE,
            },
            FilledPaging::from(None),
        )
    }

    #[test]
    fn test_from_some_paging() {
        assert_eq!(
            FilledPaging {
                page_number: 0,
                page_size: 10,
            },
            FilledPaging::from(Some(Paging {
                page_number: None,
                page_size: Some(10)
            })),
        );
        assert_eq!(
            FilledPaging {
                page_number: 1,
                page_size: 50,
            },
            FilledPaging::from(Some(Paging {
                page_number: Some(1),
                page_size: None
            })),
        );
        assert_eq!(
            FilledPaging {
                page_number: 1,
                page_size: 10,
            },
            FilledPaging::from(Some(Paging {
                page_number: Some(1),
                page_size: Some(10)
            })),
        )
    }

    #[test]
    fn test_get_page_range() {
        assert_eq!(
            (Range { start: 10, end: 20 }, 500),
            FilledPaging {
                page_number: 1,
                page_size: 10,
            }
            .get_page_range(5_000, 1),
        );
        assert_eq!(
            (
                Range {
                    start: 4_980,
                    end: 4_990
                },
                500
            ),
            FilledPaging {
                page_number: -2,
                page_size: 10,
            }
            .get_page_range(5_000, 1),
        );
        assert_eq!(
            (Range { start: 10, end: 15 }, 2),
            FilledPaging {
                page_number: 1,
                page_size: 10,
            }
            .get_page_range(15, 1),
        );
        assert_eq!(
            (Range { start: 15, end: 15 }, 2),
            FilledPaging {
                page_number: 2,
                page_size: 10,
            }
            .get_page_range(15, 1),
        );
        assert_eq!(
            (Range { start: 20, end: 40 }, 250),
            FilledPaging {
                page_number: 1,
                page_size: 10,
            }
            .get_page_range(5_000, 2),
        );
        assert_eq!(
            (
                Range {
                    start: 4_980,
                    end: 5_000
                },
                250
            ),
            FilledPaging {
                page_number: -1,
                page_size: 10,
            }
            .get_page_range(5_000, 2),
        );
        assert_eq!(
            (Range { start: 0, end: 400 }, 1),
            FilledPaging {
                page_number: -1,
                page_size: 500,
            }
            .get_page_range(400, 2),
        );
        assert_eq!(
            (
                Range {
                    start: 0,
                    end: 1_000
                },
                1
            ),
            FilledPaging {
                page_number: -3,
                page_size: 400,
            }
            .get_page_range(1_000, 5),
        );
        assert_eq!(
            (
                Range {
                    start: 2_000,
                    end: 3_000
                },
                2
            ),
            FilledPaging {
                page_number: -1,
                page_size: 400,
            }
            .get_page_range(3_000, 5),
        );
        assert_eq!(
            (Range { start: 40, end: 80 }, 3),
            FilledPaging {
                page_number: -2,
                page_size: 40,
            }
            .get_page_range(100, 1),
        );
    }
}
