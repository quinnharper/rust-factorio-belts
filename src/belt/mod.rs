use crate::util::NonMaxUsize;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy)]
pub struct BeltLineProperties {
    speed: u32,
    item_separation: u32,
}
impl BeltLineProperties {
    pub const fn new(speed: u32, item_separation: u32) -> Self {
        Self {
            speed,
            item_separation,
        }
    }
    pub const fn speed(&self) -> u32 {
        self.speed
    }
    pub const fn item_separation(&self) -> u32 {
        self.item_separation
    }
}

impl Default for BeltLineProperties {
    fn default() -> Self {
        Self {
            speed: 1,
            item_separation: 4,
        }
    }
}

#[derive(Debug, Clone)]
struct BeltElement<T> {
    /// The distance between the back-most point of the item and the next thing in front of it
    front_gap: u32,
    contents: T,
}

#[derive(Debug, Clone)]
pub struct BeltLine<T> {
    /// Element 0 is the front of the belt.
    elements: VecDeque<BeltElement<T>>,
    /// Index of the front-most uncompressed item
    front_uncompressed: Option<NonMaxUsize>,
    /// Index of the next over-compressed item, after the front-most over-compressed item
    next_overcompressed: Option<NonMaxUsize>,
    properties: BeltLineProperties,
}

impl<T> BeltLine<T> {
    pub const fn new(properties: BeltLineProperties) -> Self {
        Self {
            elements: VecDeque::new(),
            front_uncompressed: None,
            next_overcompressed: None,
            properties,
        }
    }

    pub fn tick(&mut self) {
        let mut movement_left = self.properties.speed();

        while movement_left > 0 {
            let movement_amount = self.move_element(&mut movement_left);

            let Some(next_overcompressed) = self.next_overcompressed else {
                continue;
            };
            self.decompress(next_overcompressed, movement_amount);
        }
    }

    pub fn append_item(&mut self, distance: u32, item: T) {
        self.elements.push_back(BeltElement {
            front_gap: distance,
            contents: item,
        });

        if distance < self.properties.item_separation() {
            let index = self.elements.len();
            if self.front_uncompressed.is_some() {
                match &mut self.next_overcompressed {
                    Some(_) => (),
                    overcompressed @ None => {
                        *overcompressed = NonMaxUsize::new(index);
                    }
                }
            }
        } else if distance > self.properties.item_separation() {
            let index = self.elements.len();

            match &mut self.front_uncompressed {
                Some(_) => (),
                uncompressed @ None => {
                    *uncompressed = NonMaxUsize::new(index);
                }
            }
        }
    }

    pub fn can_sleep(&self) -> bool {
        self.front_uncompressed.is_none()
    }

    fn move_from(&mut self, index: &mut Option<NonMaxUsize>, movement_left: &mut u32) -> u32 {
        let front_gap = self.properties.item_separation();

        let Some(front_uncompressed) = index else {
            return 0;
        };

        let to_move = &mut self.elements[front_uncompressed.get()];
        let movement_amount;

        if let Some(element_movement_available) = to_move.front_gap.checked_sub(front_gap)
            && element_movement_available != 0
        {
            movement_amount = (*movement_left).min(element_movement_available);
            *movement_left -= movement_amount;
            to_move.front_gap -= movement_amount;
        } else {
            movement_amount = 0;
        }
        *index = (*index).and_then(|x| {
            let next = x.get() + 1;
            if next >= self.elements.len() {
                None
            } else {
                NonMaxUsize::new(next)
            }
        });
        return movement_amount;
    }

    fn move_element(&mut self, movement_left: &mut u32) -> u32 {
        // Some shuffling so that we don't get aliasing pointers
        let mut front = self.front_uncompressed;
        let movement_amount = self.move_from(&mut front, movement_left);
        self.front_uncompressed = front;

        movement_amount
    }

    fn decompress(&mut self, start_index: NonMaxUsize, mut decompress_budget: u32) {
        for index in start_index.get()..self.elements.len() {
            self.next_overcompressed = NonMaxUsize::new(index);
            // Return if we run out of decompress

            let Some(compress_amount) = self
                .properties
                .item_separation()
                .checked_sub(self.elements[index].front_gap)
            else {
                // This element is properly compressed
                continue;
            };
            let decompress_amount = compress_amount.min(decompress_budget);
            self.elements[index].front_gap += decompress_amount;
            decompress_budget -= decompress_amount;

            self.move_from(
                &mut NonMaxUsize::new(index + 1),
                &mut decompress_amount.clone(),
            );

            if decompress_budget == 0 {
                return;
            }
        }
        // We didn't run out of decompression budget, but we did run out of elements to decompress
        // So all elements starting at `start_index` are no longer overcompressed
        self.next_overcompressed = None;
    }

    pub fn iter_positions(&self) -> impl IntoIterator<Item = (u32, &T)> {
        let mut position = 0;
        self.elements.iter().map(move |x| {
            position += x.front_gap;
            (position, &x.contents)
        })
    }

    pub fn ascii_art_display_to(&self, out: &mut String) {
        out.clear();
        let cap = self
            .iter_positions()
            .into_iter()
            .last()
            .map_or(0, |(pos, _)| pos)
            + self.properties.item_separation();
        out.reserve(cap as usize);
        let item_marker = if self.properties.item_separation() > 1 {
            "["
        } else {
            "#"
        }
        .chars()
        .into_iter()
        .chain(std::iter::repeat_n(
            '=',
            self.properties.item_separation() as usize - 2,
        ))
        .chain(if self.properties.item_separation() > 1 {
            Some(']')
        } else {
            None
        });

        for (position, _) in self.iter_positions() {
            out.truncate(position as usize);
            out.extend(std::iter::repeat_n(' ', position as usize - out.len()));
            out.extend(item_marker.clone());
        }
    }

    pub fn ascii_art_display(&self) -> String {
        let mut string = String::new();
        self.ascii_art_display_to(&mut string);
        string
    }
}
