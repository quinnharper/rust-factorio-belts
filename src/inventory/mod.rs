use crate::polyfill::option::OptionPolyfill;
use std::num::NonZeroUsize;

struct ItemAmount<T> {
    item: T,
    amount: NonZeroUsize,
}

trait ItemMatcher<T> {
    fn matches(&self, checked_item: &T) -> bool;
}

struct ExactItemMatch<T>(T);
impl<T> ItemMatcher<T> for ExactItemMatch<T>
where
    for<'a> &'a T: PartialEq,
{
    fn matches(&self, checked_item: &T) -> bool {
        &self.0 == checked_item
    }
}

impl<F, T> ItemMatcher<T> for F
where
    F: Fn(&T) -> bool,
{
    fn matches(&self, checked_item: &T) -> bool {
        self(checked_item)
    }
}

trait Slot<T>
where
    T: PartialEq,
{
    /// Try to add items to this slot. If the slot can't accept all the items, it will return `Err` holding the remaining items given to the slot.
    fn try_add_items(
        &mut self,
        item_amount: ItemAmount<T>,
        max_stack: NonZeroUsize,
    ) -> Result<(), ItemAmount<T>>;

    /// Try to take items of a matching type from this slot. If the slot can
    fn try_take_items<P>(
        &mut self,
        item_matcher: P,
        max_take_amount: NonZeroUsize,
    ) -> Option<ItemAmount<T>>
    where
        P: ItemMatcher<T>;
}

struct FilledSlot<T>(ItemAmount<T>);

enum ItemSlot<T> {
    Empty,
    Filled(FilledSlot<T>),
}

impl<T> Slot<T> for ItemSlot<T>
where
    T: PartialEq + Clone,
{
    fn try_add_items(
        &mut self,
        item_amount: ItemAmount<T>,
        max_stack: NonZeroUsize,
    ) -> Result<(), ItemAmount<T>> {
        let take_amount = match self {
            ItemSlot::Empty => max_stack,
            ItemSlot::Filled(filled_slot) => {
                if &filled_slot.0.item == &item_amount.item {
                    match max_stack
                        .get()
                        .checked_sub(filled_slot.0.amount.get())
                        .and_then(NonZeroUsize::new)
                    {
                        Some(take_amount) => take_amount,
                        None => return Err(item_amount),
                    }
                } else {
                    return Err(item_amount);
                }
            }
        };

        match self {
            slot @ ItemSlot::Empty => {
                *slot = ItemSlot::Filled(FilledSlot(ItemAmount {
                    item: item_amount.item.clone(),
                    amount: take_amount,
                }));
            }
            ItemSlot::Filled(filled_slot) => {
                filled_slot.0.amount = filled_slot
                    .0
                    .amount
                    .get()
                    .checked_add(take_amount.get())
                    .and_then(NonZeroUsize::new)
                    .expect("Should be impossible to overflow because the amount we're adding on is at most just enough to top up to max_stack, which is also a usize");
            }
        }

        item_amount
            .amount
            .get()
            .checked_sub(take_amount.get())
            .and_then(NonZeroUsize::new)
            .map(|amount| ItemAmount {
                item: item_amount.item,
                amount,
            })
            .err_or(())
    }

    fn try_take_items<P>(
        &mut self,
        item_matcher: P,
        max_take_amount: NonZeroUsize,
    ) -> Option<ItemAmount<T>>
    where
        P: ItemMatcher<T>,
    {
        match self {
            ItemSlot::Empty => None,
            ItemSlot::Filled(filled_slot) => {
                if item_matcher.matches(&filled_slot.0.item) {
                    let take_amount = max_take_amount.min(filled_slot.0.amount);
                    let remaining_amount = filled_slot
                        .0
                        .amount
                        .get()
                        .checked_sub(take_amount.get())
                        .and_then(NonZeroUsize::new);

                    match remaining_amount {
                        Some(remaining) => todo!(),
                        None => todo!(),
                    }
                    todo!()
                } else {
                    None
                }
            }
        }
    }
}

enum FilteredItemSlot<T> {
    Empty(T),
    Filled(FilledSlot<T>),
}
