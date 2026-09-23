#[derive(Debug)]
pub(crate) struct IdMapper<T> {
    permutation_mapper: Vec<Option<PermutationElement>>,
    items: Vec<(usize, T)>,
    free_ids: FreeIdManager,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Id {
    permutation_index: usize,
    generation: usize,
}
#[derive(Debug, Clone, Copy)]
pub(crate) struct PermutationElement {
    item_index: usize,
    generation: usize,
}
#[derive(Debug)]
struct FreeIdManager {
    known_free_ids: Vec<Id>,
    trailing_free_ids: usize,
}

impl FreeIdManager {
    fn get_new_id(&mut self) -> Id {
        self.known_free_ids.pop().unwrap_or_else(|| {
            let id = self.trailing_free_ids;
            self.trailing_free_ids += 1;
            Id {
                permutation_index: id,
                generation: 0,
            }
        })
    }
    fn free_id(&mut self, id: Id) {
        self.known_free_ids.push(Id {
            permutation_index: id.permutation_index,
            generation: id.generation + 1,
        });
    }
}

impl<T> IdMapper<T> {
    pub(crate) fn new() -> Self {
        Self::with_capacity(0)
    }
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        IdMapper {
            permutation_mapper: Vec::with_capacity(capacity),
            items: Vec::with_capacity(capacity),
            free_ids: FreeIdManager {
                known_free_ids: Vec::with_capacity(capacity),
                trailing_free_ids: 0,
            },
        }
    }
    pub fn push(&mut self, item: T) -> Id {
        let id = self.free_ids.get_new_id();
        let permutation_map_index = id.permutation_index;
        let permutation = Some(PermutationElement {
            item_index: self.items.len(),
            generation: id.generation,
        });
        match self.permutation_mapper.len().cmp(&permutation_map_index) {
            std::cmp::Ordering::Less => unreachable!("Logical invariant of the type"),
            std::cmp::Ordering::Equal => self.permutation_mapper.push(permutation),
            std::cmp::Ordering::Greater => {
                self.permutation_mapper[id.permutation_index] = permutation
            }
        }
        self.items.push((id.permutation_index, item));
        id
    }

    pub(crate) fn take(&mut self, id: Id) -> Option<T> {
        let permutation = self.permutation_mapper.get(id.permutation_index)?.clone()?;
        if permutation.generation == id.generation {
            let modify_permutation_index = self.items.last().expect("There has to be something in the item list as there's something in the permutation mapper").0;
            let (permutation_index, out) = self.items.swap_remove(permutation.item_index);
            self.permutation_mapper[permutation_index] = None;
            let took_last_element = self.items.len() == permutation.item_index;

            debug_assert_eq!(
                permutation_index, id.permutation_index,
                "They should point at each other (logical invariant in this type)"
            );

            if !took_last_element {
                let perm_element =
                    self.permutation_mapper[modify_permutation_index].expect("Logical invariant");
                self.permutation_mapper[modify_permutation_index] = Some(PermutationElement {
                    item_index: permutation.item_index,
                    generation: perm_element.generation,
                });
            }

            Some(out)
        } else {
            None
        }
    }

    pub(crate) fn get(&self, id: Id) -> Option<&T> {
        let permutation = self.permutation_mapper[id.permutation_index]?;
        if id.generation == permutation.generation {
            self.items.get(permutation.item_index).map(|(_, x)| x)
        } else {
            None
        }
    }
    pub(crate) fn get_mut(&mut self, id: Id) -> Option<&mut T> {
        let permutation = self.permutation_mapper[id.permutation_index]?;
        if id.generation == permutation.generation {
            self.items.get_mut(permutation.item_index).map(|(_, x)| x)
        } else {
            None
        }
    }
    pub(crate) fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter().map(|(_, t)| t)
    }
    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.iter_mut().map(|(_, t)| t)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn stab_insert() {
        let mut container = IdMapper::<u8>::with_capacity(8);

        let a_val = 0;
        let a = container.push(a_val);
        let b_val = 2;
        let b = container.push(b_val);

        let a_out = container.take(a);
        assert_eq!(Some(a_val), a_out);

        let c_val = 26;
        let c = container.push(c_val);

        assert_eq!(container.take(a), None);

        let c_out = container.take(c);
        assert_eq!(c_out, Some(c_val));

        let b_out = container.take(b);
        assert_eq!(b_out, Some(b_val));
    }
}
