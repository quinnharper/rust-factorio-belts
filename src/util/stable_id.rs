#[derive(Debug)]
pub(crate) struct IdMapper<T> {
    permutation_mapper: Vec<Option<PermutationElement>>,
    items: Vec<(usize, T)>,
    free_ids: FreeIdManager,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Id {
    index: usize,
    generation: usize,
}
#[derive(Debug, Clone, Copy)]
pub(crate) struct PermutationElement {
    position: usize,
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
                index: id,
                generation: 0,
            }
        })
    }
    fn free_id(&mut self, id: Id) {
        self.known_free_ids.push(Id {
            index: id.index,
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
        self.permutation_mapper[id.index] = Some(PermutationElement {
            position: self.items.len(),
            generation: id.generation,
        });
        self.items.push((id.index, item));
        id
    }

    pub(crate) fn take(&mut self, id: Id) -> Option<T> {
        let index = self.permutation_mapper.get(id.index)?.clone()?;
        if index.generation == id.generation {
            let (permutation_index, out) = self.items.swap_remove(index.position);
            self.permutation_mapper[permutation_index] = None;

            debug_assert_eq!(
                permutation_index, id.index,
                "They should point at each other (logical invariant in this type)"
            );

            let modify_permutation_index = self.items[index.position].0;

            let perm_element =
                self.permutation_mapper[modify_permutation_index].expect("Logical invariant");
            self.permutation_mapper[modify_permutation_index] = Some(PermutationElement {
                position: index.position,
                generation: perm_element.generation,
            });

            Some(out)
        } else {
            None
        }
    }

    pub(crate) fn get(&self, id: Id) -> Option<&T> {
        let permutation = self.permutation_mapper[id.index]?;
        if id.generation == permutation.generation {
            self.items.get(permutation.position).map(|(_, x)| x)
        } else {
            None
        }
    }
    pub(crate) fn get_mut(&mut self, id: Id) -> Option<&mut T> {
        let permutation = self.permutation_mapper[id.index]?;
        if id.generation == permutation.generation {
            self.items.get_mut(permutation.position).map(|(_, x)| x)
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
