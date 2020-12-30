use crate::plugins::physics::AxisAlignedBoundingBox;
use bevy::{
    math::Vec2,
    utils::{AHashExt, HashMap},
};
use tracing::instrument;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Entry(usize);

#[derive(Clone, Debug)]
pub(crate) enum QuadTreeNode<
    const MIN_ENTRIES: usize,
    const MAX_ENTRIES: usize,
    const MAX_DEPTH: usize,
> {
    Inner {
        bounds: AxisAlignedBoundingBox,
        children: Box<[Self; 4]>,
        entries: HashMap<Entry, AxisAlignedBoundingBox>,
        length: usize,
    },
    Leaf {
        bounds: AxisAlignedBoundingBox,
        entries: HashMap<Entry, AxisAlignedBoundingBox>,
    },
}

impl<const MIN_ENTRIES: usize, const MAX_ENTRIES: usize, const MAX_DEPTH: usize>
    QuadTreeNode<MIN_ENTRIES, MAX_ENTRIES, MAX_DEPTH>
{
    #[instrument(skip(self))]
    pub fn bounds(&self) -> AxisAlignedBoundingBox {
        match self {
            QuadTreeNode::Inner { bounds, .. } => *bounds,
            QuadTreeNode::Leaf { bounds, .. } => *bounds,
        }
    }

    #[instrument(skip(self))]
    pub fn len(&self) -> usize {
        match self {
            QuadTreeNode::Leaf { entries, .. } => entries.len(),
            QuadTreeNode::Inner { length, .. } => *length,
        }
    }

    #[instrument(skip(self, entry, bounds))]
    pub fn insert(&mut self, depth: usize, entry: Entry, bounds: AxisAlignedBoundingBox) {
        match self {
            QuadTreeNode::Inner {
                children,
                entries,
                length,
                ..
            } => {
                if let Some(child) = children
                    .iter_mut()
                    .filter(|child| child.bounds().contains(&bounds))
                    .nth(0)
                {
                    child.insert(depth + 1, entry, bounds);
                } else {
                    entries.insert(entry, bounds);
                }

                *length += 1;
            }
            QuadTreeNode::Leaf {
                entries,
                bounds: leaf_bounds,
            } => {
                if depth < MAX_DEPTH && entries.len() >= MAX_ENTRIES {
                    // Split the node
                    let child_size = leaf_bounds.size() / 2.0;
                    let mut new_node = QuadTreeNode::Inner {
                        bounds: *leaf_bounds,
                        entries: HashMap::default(),
                        length: 0,
                        children: Box::new([
                            QuadTreeNode::Leaf {
                                bounds: AxisAlignedBoundingBox::new(
                                    leaf_bounds.top_left(),
                                    child_size,
                                ),
                                entries: HashMap::default(),
                            },
                            QuadTreeNode::Leaf {
                                bounds: AxisAlignedBoundingBox::new(
                                    Vec2::new(leaf_bounds.center()[0], leaf_bounds.top()),
                                    child_size,
                                ),
                                entries: HashMap::default(),
                            },
                            QuadTreeNode::Leaf {
                                bounds: AxisAlignedBoundingBox::new(
                                    Vec2::new(leaf_bounds.left(), leaf_bounds.center()[1]),
                                    child_size,
                                ),
                                entries: HashMap::default(),
                            },
                            QuadTreeNode::Leaf {
                                bounds: AxisAlignedBoundingBox::new(
                                    leaf_bounds.center(),
                                    child_size,
                                ),
                                entries: HashMap::default(),
                            },
                        ]),
                    };

                    // Reinsert each entry
                    for (entry, bounds) in entries.drain() {
                        new_node.insert(depth, entry, bounds);
                    }

                    // Insert new entry
                    new_node.insert(depth, entry, bounds);

                    // Replace self
                    std::mem::swap(self, &mut new_node);
                } else {
                    // No need to split
                    entries.insert(entry, bounds);
                }
            }
        }
    }

    #[instrument(skip(self, entry))]
    pub fn remove(&mut self, entry: Entry) -> Option<AxisAlignedBoundingBox> {
        match self {
            QuadTreeNode::Leaf { entries, .. } => entries.remove(&entry),
            QuadTreeNode::Inner {
                entries,
                children,
                bounds: node_bounds,
                length,
            } => {
                let result = entries
                    .remove(&entry)
                    .or_else(|| children.iter_mut().find_map(|child| child.remove(entry)));

                // Merge if needed
                if result.is_some() {
                    *length -= 1;
                    if *length <= MIN_ENTRIES {
                        let mut new_node = QuadTreeNode::Leaf {
                            bounds: *node_bounds,
                            entries: HashMap::with_capacity(*length),
                        };

                        #[instrument(skip(source, dest))]
                        fn drain_into<
                            const MIN_ENTRIES: usize,
                            const MAX_ENTRIES: usize,
                            const MAX_DEPTH: usize,
                        >(
                            source: &mut QuadTreeNode<MIN_ENTRIES, MAX_ENTRIES, MAX_DEPTH>,
                            dest: &mut QuadTreeNode<MIN_ENTRIES, MAX_ENTRIES, MAX_DEPTH>,
                        ) {
                            match source {
                                QuadTreeNode::Leaf { entries, .. } => {
                                    for (entry, bounds) in entries.drain() {
                                        dest.insert(0, entry, bounds);
                                    }
                                }
                                QuadTreeNode::Inner {
                                    entries, children, ..
                                } => {
                                    for (entry, bounds) in entries.drain() {
                                        dest.insert(0, entry, bounds);
                                    }

                                    for child in children.iter_mut() {
                                        drain_into(child, dest);
                                    }
                                }
                            }
                        }

                        // Move entries from inner node
                        drain_into(self, &mut new_node);
                        std::mem::swap(self, &mut new_node);
                    }
                }

                result
            }
        }
    }

    #[instrument(skip(self, entry))]
    pub fn get_bounds(&self, entry: Entry) -> Option<AxisAlignedBoundingBox> {
        match self {
            QuadTreeNode::Leaf { entries, .. } => entries.get(&entry).copied(),
            QuadTreeNode::Inner {
                entries, children, ..
            } => entries
                .get(&entry)
                .copied()
                .or_else(|| children.iter().find_map(|child| child.get_bounds(entry))),
        }
    }
}

#[derive(Clone, Debug)]
pub struct QuadTree<T, const MIN_ENTRIES: usize, const MAX_ENTRIES: usize, const MAX_DEPTH: usize> {
    id: usize,
    pub(crate) entries: HashMap<Entry, T>,
    pub(crate) root: QuadTreeNode<MIN_ENTRIES, MAX_ENTRIES, MAX_DEPTH>,
    pub(crate) uncontained: HashMap<Entry, AxisAlignedBoundingBox>,
}

impl<T, const MIN_ENTRIES: usize, const MAX_ENTRIES: usize, const MAX_DEPTH: usize>
    QuadTree<T, MIN_ENTRIES, MAX_ENTRIES, MAX_DEPTH>
{
    #[instrument(skip(bounds))]
    pub fn new(bounds: AxisAlignedBoundingBox) -> Self {
        QuadTree {
            id: 0,
            entries: HashMap::new(),
            root: QuadTreeNode::Leaf {
                bounds,
                entries: HashMap::new(),
            },
            uncontained: HashMap::new(),
        }
    }

    #[instrument(skip(self, item, bounds))]
    pub fn insert(&mut self, item: T, bounds: AxisAlignedBoundingBox) -> Entry {
        // Get a new entry ID
        let entry = Entry(self.id);
        self.entries.insert(entry, item);
        self.id += 1;

        // Insert the entry
        self.insert_entry(entry, bounds);
        entry
    }

    #[instrument(skip(self, entry, bounds))]
    fn insert_entry(&mut self, entry: Entry, bounds: AxisAlignedBoundingBox) {
        if !self.root.bounds().contains(&bounds) {}

        if self.root.bounds().contains(&bounds) {
            self.root.insert(0, entry, bounds)
        } else {
            self.uncontained.insert(entry, bounds);
        }
    }

    #[instrument(skip(self, entry))]
    pub fn remove(&mut self, entry: Entry) -> Option<(T, AxisAlignedBoundingBox)> {
        self.remove_entry(entry)
            .map(|bounds| {
                self.entries
                    .remove(&entry)
                    .map(move |value| (value, bounds))
            })
            .flatten()
    }

    #[instrument(skip(self, entry))]
    fn remove_entry(&mut self, entry: Entry) -> Option<AxisAlignedBoundingBox> {
        self.uncontained
            .remove(&entry)
            .or_else(|| self.root.remove(entry))
    }

    #[instrument(skip(self, entry))]
    pub fn get(&self, entry: Entry) -> Option<&T> {
        self.entries.get(&entry)
    }

    #[instrument(skip(self, entry))]
    pub fn get_mut(&mut self, entry: Entry) -> Option<&mut T> {
        self.entries.get_mut(&entry)
    }

    #[instrument(skip(self, entry))]
    pub fn get_bounds(&self, entry: Entry) -> Option<AxisAlignedBoundingBox> {
        self.uncontained
            .get(&entry)
            .copied()
            .or_else(|| self.root.get_bounds(entry))
    }

    #[instrument(skip(self, entry, bounds))]
    pub fn set_bounds(
        &mut self,
        entry: Entry,
        bounds: AxisAlignedBoundingBox,
    ) -> Option<AxisAlignedBoundingBox> {
        match self.remove_entry(entry) {
            Some(old_bounds) => {
                self.insert_entry(entry, bounds);
                Some(old_bounds)
            }
            None => None,
        }
    }

    #[instrument(skip(self, point))]
    pub fn query_point(&self, point: Vec2) -> QueryPoint<'_, MIN_ENTRIES, MAX_ENTRIES, MAX_DEPTH> {
        let root_entries = match &self.root {
            QuadTreeNode::Inner { entries, .. } | QuadTreeNode::Leaf { entries, .. } => {
                entries.keys()
            }
        };

        QueryPoint {
            node: &self.root,
            buffer: self
                .uncontained
                .keys()
                .chain(root_entries)
                .copied()
                .collect(),
            point,
        }
    }

    #[instrument(skip(self, bounds))]
    pub fn query_bounds(
        &self,
        bounds: AxisAlignedBoundingBox,
    ) -> QueryBounds<'_, MIN_ENTRIES, MAX_ENTRIES, MAX_DEPTH> {
        QueryBounds {
            nodes: vec![&self.root],
            buffer: self.uncontained.keys().copied().collect(),
            bounds,
        }
    }
}

pub struct QueryPoint<
    'a,
    const MIN_ENTRIES: usize,
    const MAX_ENTRIES: usize,
    const MAX_DEPTH: usize,
> {
    node: &'a QuadTreeNode<MIN_ENTRIES, MAX_ENTRIES, MAX_DEPTH>,
    buffer: Vec<Entry>,
    point: Vec2,
}

impl<'a, const MIN_ENTRIES: usize, const MAX_ENTRIES: usize, const MAX_DEPTH: usize> Iterator
    for QueryPoint<'a, MIN_ENTRIES, MAX_ENTRIES, MAX_DEPTH>
{
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        while self.buffer.is_empty() {
            // Focus on next child
            if let QuadTreeNode::Inner { children, .. } = self.node {
                self.node = children
                    .iter()
                    .find(|child| child.bounds().contains_point(self.point))?;

                // Get entries stored in that node
                self.buffer.extend(match self.node {
                    QuadTreeNode::Leaf { entries, .. } => entries.keys().copied(),
                    QuadTreeNode::Inner { entries, .. } => entries.keys().copied(),
                });
            } else {
                break;
            }
        }

        self.buffer.pop()
    }
}

pub struct QueryBounds<
    'a,
    const MIN_ENTRIES: usize,
    const MAX_ENTRIES: usize,
    const MAX_DEPTH: usize,
> {
    nodes: Vec<&'a QuadTreeNode<MIN_ENTRIES, MAX_ENTRIES, MAX_DEPTH>>,
    buffer: Vec<Entry>,
    bounds: AxisAlignedBoundingBox,
}

impl<'a, const MIN_ENTRIES: usize, const MAX_ENTRIES: usize, const MAX_DEPTH: usize> Iterator
    for QueryBounds<'a, MIN_ENTRIES, MAX_ENTRIES, MAX_DEPTH>
{
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        while self.buffer.is_empty() {
            let node = self.nodes.pop()?;
            match node {
                QuadTreeNode::Leaf { entries, .. } => {
                    self.buffer.extend(entries.keys().copied());
                }
                QuadTreeNode::Inner {
                    entries, children, ..
                } => {
                    let QueryBounds {
                        ref mut buffer,
                        ref mut nodes,
                        bounds,
                    } = self;
                    buffer.extend(entries.keys().copied());
                    nodes.extend(
                        children
                            .iter()
                            .filter(|child| child.bounds().intersects(bounds)),
                    );
                }
            }
        }

        self.buffer.pop()
    }
}
