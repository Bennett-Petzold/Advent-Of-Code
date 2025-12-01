use std::{
    cell::{Ref, RefCell},
    hash::Hash,
    rc::Rc,
};

use crate::grid::Pos2D;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RevLinkedNodeInternal<T = Pos2D> {
    value: T,
    next: RevLinkedNode<T>,
}

pub type RevLinkedNode<T = Pos2D> = Option<Rc<RevLinkedNodeInternal<T>>>;

impl<T> RevLinkedNodeInternal<T> {
    pub fn new() -> RevLinkedNode<T> {
        None
    }

    pub fn push(this: RevLinkedNode<T>, new_pos: T) -> RevLinkedNode<T> {
        Some(Rc::new(Self {
            value: new_pos,
            next: this,
        }))
    }

    pub fn iter(this: RevLinkedNode<T>) -> RevLinkedNodeIter<T> {
        RevLinkedNodeIter { node: this }
    }
}

// Iterates in reverse order
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RevLinkedNodeIter<T = Pos2D> {
    node: RevLinkedNode<T>,
}

impl<T: Clone> Iterator for RevLinkedNodeIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let prev_node = std::mem::take(&mut self.node)?;
        let value = prev_node.value.clone();
        self.node = prev_node.next.clone();

        Some(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PointerSequenceInternal<T = Pos2D> {
    Value(RefCell<T>),
    Next(PointerSequence<T>),
}

impl<T: Hash> Hash for PointerSequenceInternal<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Value(x) => x.borrow().hash(state),
            Self::Next(n) => n.hash(state),
        }
    }
}

pub type PointerSequence<T = Pos2D> = Rc<PointerSequenceInternal<T>>;

impl<T> PointerSequenceInternal<T> {
    pub fn new(value: T) -> PointerSequence<T> {
        Rc::new(PointerSequenceInternal::Value(value.into()))
    }

    pub fn point(this: PointerSequence<T>) -> PointerSequence<T> {
        Rc::new(PointerSequenceInternal::Next(this))
    }

    pub fn resolve(this: &PointerSequence<T>) -> Ref<'_, T> {
        match this.as_ref() {
            PointerSequenceInternal::Value(x) => x.borrow(),
            PointerSequenceInternal::Next(n) => Self::resolve(n),
        }
    }
}
