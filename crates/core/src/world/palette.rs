use std::fmt::Debug;

pub type PaletteToI32<T> = fn(&T) -> Option<i32>;

#[derive(Clone, Debug)]
pub struct Palette<T: Clone + Debug> {
    items: Vec<T>,
}

impl<T: PartialEq + Clone + Debug + 'static> Palette<T> {
    pub fn empty() -> Self {
        Palette { items: vec![] }
    }

    pub fn new(init: Vec<T>) -> Self {
        Palette { items: init }
    }

    pub fn push(&mut self, item: T) -> usize {
        self.items.push(item);
        self.items.len() - 1
    }

    pub fn get_index(&self, target: &T) -> Option<usize> {
        self.items
            .iter()
            .enumerate()
            .find(|(_, item)| *item == target)
            .map(|(i, _)| i)
    }

    pub fn at(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }

    pub fn remove(&mut self, index: usize) -> usize {
        self.items.swap_remove(index);
        self.items.len()
    }

    pub fn calculate_bits_per_entry(&self, to_i32: fn(&T) -> Option<i32>) -> u32 {
        let count = self
            .items
            .iter()
            .map(to_i32)
            .filter(|item| item.is_some())
            .count();
        usize::BITS - count.leading_zeros()
    }

    pub fn build_direct_palette<'a, I>(
        &'a self,
        data_iterator: I,
        to_i32: PaletteToI32<T>,
        default: T,
    ) -> impl Iterator<Item = u64> + 'a
    where
        I: Iterator<Item = u16> + 'a,
    {
        let default_value = to_i32(&default).unwrap();
        data_iterator
            .map(move |value| to_i32(&self.items[value as usize]).unwrap_or(default_value) as u64)
    }

    pub fn build_indirect_palette<'a, I>(
        &'a self,
        data_iterator: I,
        to_i32: PaletteToI32<T>,
        default: T,
    ) -> (impl Iterator<Item = u64>, Vec<i32>)
    where
        I: Iterator<Item = u16> + 'a,
    {
        let default_value = to_i32(&default).unwrap() as u64;
        let mut palette_missing = 0;
        let modified_palette: Vec<i32> = {
            let mut section_palette: Vec<Option<i32>> = self.items.iter().map(to_i32).collect();
            let mut i = 0;
            while i < section_palette.len() - palette_missing {
                if section_palette[i].is_none() {
                    section_palette.remove(i);
                    section_palette.push(Some((i + palette_missing) as i32));
                    palette_missing += 1;
                } else {
                    i += 1;
                }
            }
            section_palette.iter().map(|value| value.unwrap()).collect()
        };
        let palette_len = modified_palette.len();
        let final_palette = modified_palette[..palette_len - palette_missing].to_owned();
        (
            data_iterator.map(move |value| {
                if modified_palette[palette_len - palette_missing..palette_len].contains(&(value as i32)) {
                    default_value
                } else {
                    let mut res = value;
                    for j in &modified_palette[palette_len - palette_missing..palette_len] {
                        if value > *j as u16 {
                            res -= 1
                        }
                    }
                    res as u64
                }
            }),
            final_palette,
        )
    }
}
