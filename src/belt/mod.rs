use crate::util::{Id, IdMapper, NonMaxUsize};
use std::{collections::VecDeque, ops::Range};

pub struct BeltLineLocationRange {
    start: u32,
    end: u32,
}
impl BeltLineLocationRange {
    pub const fn new(Range { start, end }: Range<u32>) -> Self {
        Self { start, end }
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BeltPositionIndex {
    position: u32,
    index: usize,
}
impl BeltPositionIndex {
    const ZERO: BeltPositionIndex = BeltPositionIndex {
        position: 0,
        index: 0,
    };
}
#[derive(Debug, Clone)]
pub struct BeltLine<T> {
    /// Element 0 is the front of the belt.
    elements: VecDeque<BeltElement<T>>,
    /// Index of the front-most uncompressed item
    front_uncompressed: Option<NonMaxUsize>,
    /// Index of the next over-compressed item, after the front-most over-compressed item
    next_overcompressed: Option<NonMaxUsize>,
    position_index_cache: Option<BeltPositionIndex>,
    properties: BeltLineProperties,
}

impl<T> BeltLine<T> {
    pub const fn new(properties: BeltLineProperties) -> Self {
        Self {
            elements: VecDeque::new(),
            front_uncompressed: None,
            next_overcompressed: None,
            position_index_cache: None,
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

    pub fn insert_at_position(&mut self, position: u32, item: T) {
        let hint = self.get_hint_for_position(position);

        self.insert_at_position_with_hint(position, item, hint);
    }

    fn get_hint_for_index_with_hint(
        &self,
        index: usize,
        hint: BeltPositionIndex,
    ) -> Option<BeltPositionIndex> {
        todo!()
    }
    fn get_hint_for_index(&self, index: usize) -> Option<BeltPositionIndex> {
        if self.elements.len() > index {
            let position = self
                .elements
                .range(0..index)
                .map(|elem| elem.front_gap)
                .sum();

            Some(BeltPositionIndex { position, index })
        } else {
            None
        }
    }

    fn get_hint_for_position(&self, position: u32) -> BeltPositionIndex {
        let index = {
            let mut pos = 0;
            self.elements.iter().enumerate().find_map(|(index, elem)| {
                pos += elem.front_gap;
                if pos < position { None } else { Some(index) }
            })
        }
        .unwrap_or(0);
        BeltPositionIndex { position, index }
    }

    fn insert_at_position_with_hint(&mut self, position: u32, item: T, hint: BeltPositionIndex) {
        let (index, item_position) = self
            .iter_positions_from_hint(hint)
            .into_iter()
            .enumerate()
            .find(|(_index, (item_position, _))| *item_position > position)
            .map_or(
                (hint.index, hint.position),
                |(index, (item_position, _))| (index, item_position),
            );

        let ahead_position = self
            .elements
            .get(index)
            .map_or(0, |element| item_position.strict_sub(element.front_gap));

        let gap_ahead = position - ahead_position;

        self.insert_ahead(index, gap_ahead, item);
    }

    pub fn take_at_index(&mut self, index: usize) -> Option<T> {
        let elem = self.elements.remove(index);
        if let Some(taken) = &elem
            && let Some(behind) = self.elements.get_mut(index)
        {
            behind.front_gap += taken.front_gap;
        }

        elem.map(
            |BeltElement {
                 front_gap: _,
                 contents,
             }| contents,
        )
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.elements.get(index).map(|x| &x.contents)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.elements.get_mut(index).map(|x| &mut x.contents)
    }

    fn insert_ahead(&mut self, index: usize, gap_ahead: u32, item: T) {
        if let Some(behind) = self.elements.get_mut(index) {
            behind.front_gap = behind.front_gap.strict_sub(gap_ahead);
        }

        if let Some(cache) = &mut self.position_index_cache {
            // Cache reset
            if cache.index > index {
                cache.index += 1
            } else if cache.index == index {
                self.position_index_cache = None;
            }
        }

        self.elements.insert(
            index,
            BeltElement {
                front_gap: gap_ahead,
                contents: item,
            },
        );
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
            if let Some(cache) = &mut self.position_index_cache {
                if cache.index == index {
                    cache.position += decompress_amount;
                }
            }

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
        self.iter_positions_from_hint(BeltPositionIndex {
            position: self.elements.get(0).map_or(0, |elem| elem.front_gap),
            index: 0,
        })
    }

    fn iter_positions_from_hint(
        &self,
        hint: BeltPositionIndex,
    ) -> impl IntoIterator<Item = (u32, &T)> {
        let mut position = hint.position;
        self.elements.iter().skip(hint.index).map(move |x| {
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

    pub fn slice_location(&self, start: u32, end: u32) -> BeltLineView<'_, T> {
        let hint = self.get_hint_for_position(start);
        BeltLineView {
            beltline: self,
            hint: hint,
            range: BeltLineLocationRange {
                start,
                end: end.max(start),
            },
        }
    }
    pub fn slice_location_mut(&mut self, start: u32, end: u32) -> BeltLineViewMut<'_, T> {
        let hint = self.get_hint_for_position(start);
        BeltLineViewMut {
            beltline: self,
            hint: hint,
            range: BeltLineLocationRange {
                start,
                end: end.max(start),
            },
        }
    }
    pub fn slice_index(&self, start: usize, end: usize) -> BeltLineView<'_, T> {
        if let Some(hint) = self.get_hint_for_index(start) {
            let end_pos = if let Some(end_hint) = self.get_hint_for_index(end) {
                end_hint.position
            } else {
                // Does not panic: There's definitely an element in the vecdeque
                self.get_hint_for_index_with_hint(self.elements.len() - 1, hint)
                    .expect("It's not empty")
                    .position
                    + 1
            };
            BeltLineView {
                beltline: self,
                hint,
                range: BeltLineLocationRange::new(hint.position..end_pos),
            }
        } else {
            BeltLineView {
                beltline: self,
                hint: BeltPositionIndex::ZERO,
                range: BeltLineLocationRange::new(0..0),
            }
        }
    }
    pub fn slice_index_mut(&mut self, start: usize, end: usize) -> BeltLineViewMut<'_, T> {
        if let Some(hint) = self.get_hint_for_index(start) {
            let end_pos = if let Some(end_hint) = self.get_hint_for_index(end) {
                end_hint.position
            } else {
                // Does not panic: There's definitely an element in the vecdeque
                self.get_hint_for_index_with_hint(self.elements.len() - 1, hint)
                    .expect("It's not empty")
                    .position
                    + 1
            };
            BeltLineViewMut {
                beltline: self,
                hint,
                range: BeltLineLocationRange::new(hint.position..end_pos),
            }
        } else {
            BeltLineViewMut {
                beltline: self,
                hint: BeltPositionIndex::ZERO,
                range: BeltLineLocationRange::new(0..0),
            }
        }
    }
}

pub struct BeltLineView<'a, T> {
    beltline: &'a BeltLine<T>,
    /// Some position before the first element in the location range
    hint: BeltPositionIndex,
    range: BeltLineLocationRange,
}
pub struct BeltLineViewMut<'a, T> {
    beltline: &'a mut BeltLine<T>,
    /// Some position before the first element in the location range
    hint: BeltPositionIndex,
    range: BeltLineLocationRange,
}

struct RollOntoInfo {
    to: Id,
    to_point: u32,
    is_priority: bool,
}
struct PriorityRollFromInfo {
    from: Id,
}

struct BeltLineInfo<T> {
    beltline: BeltLine<T>,
    /// Maybe it doesn't roll onto anything
    roll_onto: Option<RollOntoInfo>,
    roll_from: Option<PriorityRollFromInfo>,
}

pub struct BeltLineSystem<T> {
    beltlines: IdMapper<BeltLineInfo<T>>,
}

impl<T> BeltLineSystem<T> {
    fn sort(&mut self) {
        todo!()
    }
    fn disconnect_belt(&mut self, id: Id) {
        if let Some(beltline_info) = self.beltlines.get_mut(id) {
            beltline_info.roll_onto = None;
            // Reordering is not necessary
        }
    }

    fn connect_belt(&mut self, from_id: Id, to_id: Id, is_priority: bool) {
        match is_priority {
            true => self.end_extend_belt(from_id, to_id),
            false => todo!(),
        }
    }
    fn end_extend_belt(&mut self, from_id: Id, to_id: Id) {}
    fn sideload_belt(&mut self, from_id: Id, to_id: Id) {}
    fn tick(&mut self) {}
}
