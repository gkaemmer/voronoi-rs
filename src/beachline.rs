extern crate slab;
use std::ops::{Index, IndexMut};
use std::cmp::Ordering;
use crate::Site;
use crate::treeprint::print;

use slab::Slab;

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
enum Color {
    RED,
    BLACK
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
struct Pointer(usize);
impl Pointer {
    fn null() -> Pointer {
        Pointer(!0)
    }

    pub fn is_null(&self) -> bool {
        *self == Pointer::null()
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
pub struct BeachSegmentHandle(Pointer);
impl BeachSegmentHandle {
    pub fn is_null(&self) -> bool {
        self.0 == Pointer::null()
    }
}

pub struct BeachLine {
    nodes: Slab<Node>,
    root: Pointer
}

// Note: this is only public because of the implementation of the `Index` trait
// which is useful for private use here. There's no way to have the
// implementation of that trait be private to this module.
pub struct Node {
    color: Color,
    parent: Pointer,
    left: Pointer,
    right: Pointer,
    value: Site
}

// Just for convenience, so that we can type `self[i]` instead of `self.slab[i]`.
impl IndexMut<Pointer> for BeachLine {
    fn index_mut(&mut self, index: Pointer) -> &mut Node {
        &mut self.nodes[index.0]
    }
}
impl Index<Pointer> for BeachLine {
    type Output = Node;

    fn index(&self, index: Pointer) -> &Node {
        &self.nodes[index.0]
    }
}

impl BeachLine {
    pub fn new() -> BeachLine {
        BeachLine {
            nodes: Slab::new(),
            root: Pointer::null()
        }
    }

    #[allow(dead_code)]
    pub fn with_capacity(n: usize) -> BeachLine {
        BeachLine {
            nodes: Slab::with_capacity(n),
            root: Pointer::null()
        }
    }

    pub fn init(&mut self, value: Site) {
        if !self.root.is_null() {
            panic!("Tried initializing a non-empty beachline");
        }
        self.root = Pointer(self.nodes.insert(Node {
            color: Color::BLACK,
            parent: Pointer::null(),
            left: Pointer::null(),
            right: Pointer::null(),
            value: value
        }));
        self.insert_repair(self.root);
    }

    pub fn get(&self, handle: BeachSegmentHandle) -> &Site {
        &self[handle.0].value
    }

    pub fn insert_after(&mut self, handle: BeachSegmentHandle, value: Site) -> BeachSegmentHandle {
        let at = handle.0;
        if self[at].right.is_null() {
            let ptr = Pointer(self.nodes.insert(BeachLine::create_node(value, at)));
            self[at].right = ptr;
            self.insert_repair(self[at].right);
            BeachSegmentHandle(ptr)
        } else {
            let successor = self.successor_inner(at);
            self.insert_before(BeachSegmentHandle(successor), value)
        }
    }

    pub fn insert_before(&mut self, handle: BeachSegmentHandle, value: Site) -> BeachSegmentHandle {
        let at = handle.0;
        if self[at].left.is_null() {
            let ptr = Pointer(self.nodes.insert(BeachLine::create_node(value, at)));
            self[at].left = ptr;
            self.insert_repair(self[at].left);
            BeachSegmentHandle(ptr)
        } else {
            let predecessor = self.predecessor_inner(at);
            self.insert_after(BeachSegmentHandle(predecessor), value)
        }
    }

    pub fn search<F>(&self, comparator: F) -> BeachSegmentHandle where F: Fn(BeachSegmentHandle) -> Ordering {
        let mut current_node = self.root;
        while !current_node.is_null() {
            let result = comparator(BeachSegmentHandle(current_node));
            match result {
                Ordering::Less => {
                    // Go left
                    current_node = self[current_node].left;
                }
                Ordering::Greater => {
                    // Go right
                    current_node = self[current_node].right;
                }
                Ordering::Equal => {
                    // We found it
                    return BeachSegmentHandle(current_node);
                }
            }
        }
        return BeachSegmentHandle(Pointer::null());
    }

    pub fn delete(&mut self, handle: BeachSegmentHandle) -> Option<Site> {
        let at = handle.0;
        if at.is_null() { return None; }

        if !self[at].left.is_null() && !self[at].right.is_null() {
            // Node has two children,
            // Replace this node with its predecessor and delete its predecessor
            let predecessor = self.predecessor_inner(at);
            self.swap(predecessor, at);
            return self.delete(handle);
        } else if self[at].left.is_null() && self[at].right.is_null() {
            // Node has no children
            let parent = self[at].parent;
            if !parent.is_null() {
                // Node is not root, so properly update its parent
                if self[at].color == Color::BLACK {
                    // We're gonna end up invalidating the RB tree, repair it
                    // Note that if either parent or node are red, we end up with
                    // just a black node which is a valid replacement
                    self.delete_repair(at);
                }

                let parent = self[at].parent;
                if self[parent].left == at {
                    self[parent].left = Pointer::null();
                } else {
                    self[parent].right = Pointer::null();
                }
            } else {
                self.root = Pointer::null();
            }
            let node = self.nodes.remove(at.0);
            return Some(node.value);
        }

        // Node has exactly one child

        let child = if self[at].left.is_null() { self[at].right } else { self[at].left };
        // Replace node with parent
        let parent = self[at].parent;
        self[child].parent = parent;
        if parent.is_null() {
            self.root = child;
        } else if self[parent].left == at {
            self[parent].left = child;
        } else {
            self[parent].right = child;
        }

        // Repair at child
        let node = self.nodes.remove(at.0);
        if node.color == Color::RED {
            // Nothing to repair, tree still valid
        } else {
            if self[child].color == Color::RED {
                // Just set the child color to black and we're still good
                self[child].color = Color::BLACK;
            } else {
                // Node and child were both black
                // This is actually impossible. It would mean that the path from
                // the "leaf" on one side of this node would have fewer black
                // nodes than the path from the leaves on the other side.
                panic!("Impossible case: deleting black node with one black child");
            }
        }

        return Some(node.value);
    }

    pub fn predecessor(&self, handle: BeachSegmentHandle) -> BeachSegmentHandle {
        return BeachSegmentHandle(self.predecessor_inner(handle.0));
    }

    pub fn successor(&self, handle: BeachSegmentHandle) -> BeachSegmentHandle {
        return BeachSegmentHandle(self.successor_inner(handle.0));
    }

    fn predecessor_inner(&self, at: Pointer) -> Pointer {
        if at.is_null() {
            return Pointer::null();
        }

        if self[at].left.is_null() {
            let mut parent = self[at].parent;
            let mut child = at;
            if parent.is_null() {
                // Node is root and has no left children
                return Pointer::null();
            }
            while self[parent].left == child {
                if self[parent].parent.is_null() {
                    // Node only has parents to the right
                    return Pointer::null();
                }
                child = parent;
                parent = self[parent].parent;
            }
            return parent;
        } else {
            let mut child = self[at].left;
            while !self[child].right.is_null() {
                child = self[child].right;
            }
            return child;
        }
    }

    fn successor_inner(&self, at: Pointer) -> Pointer {
        if at.is_null() {
            return Pointer::null();
        }

        if self[at].right.is_null() {
            let mut parent = self[at].parent;
            let mut child = at;
            if parent.is_null() {
                // Node is root and has no right children
                return Pointer::null();
            }
            while self[parent].right == child {
                if self[parent].parent.is_null() {
                    // Node only has parents to the left
                    return Pointer::null();
                }
                child = parent;
                parent = self[parent].parent;
            }
            return parent;
        } else {
            let mut child = self[at].right;
            while !self[child].left.is_null() {
                child = self[child].left;
            }
            return child;
        }
    }

    // Replaces all references to OLD with references to NEW and copy references
    // from OLD to NEW, and vice versa
    // This is necessary so that pointers always point to the same value, even
    // after a swap
    // swap(a, d):
    //       a           d
    //      / \         / \
    //     b   c  =>   b   c
    //    /           /
    //   d           a
    fn swap(&mut self, old: Pointer, new: Pointer) {
        let old_parent = self[old].parent;
        let old_left = self[old].left;
        let old_right = self[old].right;
        let old_color = self[old].color;

        let new_parent = self[new].parent;
        let new_left = self[new].left;
        let new_right = self[new].right;
        let new_color = self[new].color;

        // Swap pointers (takes into consideration that old and new might be directly related)
        self[old].parent = if new_parent == old { new } else { new_parent };
        self[old].left = if new_left == old { new } else { new_left };
        self[old].right = if new_right == old { new } else { new_right };
        self[old].color = new_color;
        self[new].parent = if old_parent == new { old } else { old_parent };
        self[new].left = if old_left == new { old } else { old_left };
        self[new].right = if old_right == new { old } else { old_right };
        self[new].color = old_color;

        // Change references from OLD to NEW
        if self[new].parent.is_null() {
            // Node is root
            self.root = new;
        } else {
            if self[self[new].parent].right == old {
                let other = self[new].parent;
                self[other].right = new;
            } else {
                let other = self[new].parent;
                self[other].left = new;
            }
        }

        // Replace children's references
        if !self[new].left.is_null() {
            let other = self[new].left;
            self[other].parent = new;
        }
        if !self[new].right.is_null() {
            let other = self[new].right;
            self[other].parent = new;
        }

        // Change references from NEW to OLD
        if self[old].parent.is_null() {
            // Node is root
            self.root = old;
        } else {
            if self[self[old].parent].right == new {
                let other = self[old].parent;
                self[other].right = old;
            } else {
                let other = self[old].parent;
                self[other].left = old;
            }
        }

        // Replace children's references
        if !self[old].left.is_null() {
            let other = self[old].left;
            self[other].parent = old;
        }
        if !self[old].right.is_null() {
            let other = self[old].right;
            self[other].parent = old;
        }
    }

    fn create_node(value: Site, parent: Pointer) -> Node {
        Node {
            color: Color::RED,
            parent: parent,
            left: Pointer::null(),
            right: Pointer::null(),
            value: value
        }
    }

    fn insert_repair(&mut self, at: Pointer) {
        let uncle = self.uncle(at);

        if self[at].parent.is_null() {
            // Repair case 1
            self[at].color = Color::BLACK;
        } else if self[self[at].parent].color == Color::BLACK {
            // Nothing to do, we're fine
        } else if uncle != Pointer::null() && self[uncle].color == Color::RED {
            // Change uncle and parent to black
            let parent = self[at].parent;
            let grandparent = self[parent].parent;
            self[uncle].color = Color::BLACK;
            self[parent].color = Color::BLACK;
            self[grandparent].color = Color::RED;
            self.insert_repair(grandparent)
        } else {
            // Note: grandparent must exist because otherwise parent would be black
            let mut new_at = at;
            let parent = self[at].parent;
            let grandparent = self[parent].parent;

            // step 1
            if at == self[parent].right && parent == self[grandparent].left {
                self.rotate_left(parent);
                new_at = self[at].left;
            } else if at == self[parent].left && parent == self[grandparent].right {
                self.rotate_right(parent);
                new_at = self[at].right;
            }

            // step 2
            let parent = self[new_at].parent;
            let grandparent = self[parent].parent;
            if new_at == self[parent].left {
                self.rotate_right(grandparent);
            } else {
                self.rotate_left(grandparent);
            }
            self[parent].color = Color::BLACK;
            self[grandparent].color = Color::RED;
        }
    }

    fn delete_repair(&mut self, at: Pointer) {
        // Precondition: node is black and has one fewer black nodes on its path
        // to the root than its sibling does. So we need to either add a black
        // node to the node's paths or we need to take one away from the sibling's
        // paths (in which case we need to recurse upwards).
        assert!(self[at].color == Color::BLACK);
        if self.root == at {
            // Case 1: root is black and stays black, not a problem
            return;
        }

        let mut sibling = self.sibling(at);
        let mut parent = self[at].parent;
        let is_left = self[parent].left == at;

        if sibling.is_null() {
            // Impossible, black nodes never have null siblings
            panic!("Black node has a null sibling");
        }

        if self[sibling].color == Color::RED {
            // Case 2: switch parent's and sibling's colors and rotate around
            // parent. The result is that node's parent is red, and we can proceed
            // to cases 4, 5, and 6.
            self[sibling].color = Color::BLACK;
            self[parent].color = Color::RED;
            if is_left {
                self.rotate_left(parent);
            } else {
                self.rotate_right(parent);
            }
            sibling = self.sibling(at);
            parent = self[at].parent;
        } else if
            self[sibling].color == Color::BLACK &&
            self[parent].color == Color::BLACK &&
            self.has_black_children(sibling)
        {
            // Case 3: we can balance the parent tree by setting sibling to RED,
            // but we may invalidate the tree above parent, so recursively call
            // delete_repair
            self[sibling].color = Color::RED;
            return self.delete_repair(parent);
        }

        if
            self[parent].color == Color::RED &&
            self[sibling].color == Color::BLACK &&
            self.has_black_children(sibling)
        {
            // Case 4: easy--we can just swap the colors of parent and sibling,
            // which adds one black node to all of this node's paths and doesn't
            // affect sibling's paths
            self[sibling].color = Color::RED;
            self[parent].color = Color::BLACK;
            return;
        }

        if
            self[sibling].color == Color::BLACK &&
            (is_left && (
                (!self[sibling].left.is_null() && self[self[sibling].left].color == Color::RED) &&
                (self[sibling].right.is_null() || self[self[sibling].right].color == Color::BLACK)
            )) || (!is_left && (
                (!self[sibling].right.is_null() && self[self[sibling].right].color == Color::RED) &&
                (self[sibling].left.is_null() || self[self[sibling].left].color == Color::BLACK)
            ))
        {
            // Case 5 (depends on is_left, which we assume is true in the comment)
            // This one is weird. We rotate at sibling and swap the colors
            // of sibling and its new parent (its old left child). Then, node
            // will have a sibling that has a RED right child, which is addressed
            // in case 6
            if is_left {
                let left = self[sibling].left;
                self[sibling].color = Color::RED;
                self[left].color = Color::BLACK;
                self.rotate_right(sibling);
            } else {
                let right = self[sibling].right;
                self[sibling].color = Color::RED;
                self[right].color = Color::BLACK;
                self.rotate_left(sibling);
            }
            sibling = self.sibling(at);
        }

        // Finally, case 6. Also reversible. Node has a BLACK sibling with a RED
        // right child.
        // Here, we swap parent's and sibling's colors, rotate left at parent,
        // and make sibling's right child black. The result is that paths through
        // node have one additional black ancestor and paths through sibling have
        // the same number as before.
        self[sibling].color = self[parent].color;
        self[parent].color = Color::BLACK;
        if is_left {
            let sibling_right = self[sibling].right;
            if !sibling_right.is_null() {
                // Note: null is already black
                self[sibling_right].color = Color::BLACK;
            }
            self.rotate_left(parent);
        } else {
            let sibling_left = self[sibling].left;
            if !sibling_left.is_null() {
                // Note: null is already black
                self[sibling_left].color = Color::BLACK;
            }
            self.rotate_right(parent);
        }
    }

    fn has_black_children(&self, at: Pointer) -> bool {
        return
            (self[at].left.is_null() || self[self[at].left].color == Color::BLACK) &&
            (self[at].right.is_null() || self[self[at].right].color == Color::BLACK);
    }

    fn sibling(&self, at: Pointer) -> Pointer {
        if self[at].parent.is_null() {
            return Pointer::null();
        }
        let is_right = self[self[at].parent].right == at;
        if is_right {
            return self[self[at].parent].left;
        } else {
            return self[self[at].parent].right;
        }
    }

    fn uncle(&self, at: Pointer) -> Pointer {
        // Make sure parent exists
        if self[at].parent.is_null() {
            return Pointer::null();
        }
        return self.sibling(self[at].parent);
    }

    fn rotate_left(&mut self, at: Pointer) {
        let parent = self[at].parent;
        let new_parent = self[at].right;
        self[at].right = self[new_parent].left;
        self[new_parent].left = at;
        self[at].parent = new_parent;
        self[new_parent].parent = parent;

        if self[at].right != Pointer::null() {
            let new_right = self[at].right;
            self[new_right].parent = at;
        }

        if parent != Pointer::null() {
            if at == self[parent].left {
                self[parent].left = new_parent;
            } else {
                self[parent].right = new_parent;
            }
        }

        if self.root == at {
            self.root = new_parent;
        }
    }

    fn rotate_right(&mut self, at: Pointer) {
        let parent = self[at].parent;
        let new_parent = self[at].left;
        self[at].left = self[new_parent].right;
        self[new_parent].right = at;
        self[at].parent = new_parent;
        self[new_parent].parent = parent;

        if self[at].left != Pointer::null() {
            let new_left = self[at].left;
            self[new_left].parent = at;
        }

        if parent != Pointer::null() {
            if at == self[parent].right {
                self[parent].right = new_parent;
            } else {
                self[parent].left = new_parent;
            }
        }

        if self.root == at {
            self.root = new_parent;
        }
    }

    pub fn in_order<F>(&self, mut f: F) where F: FnMut(&Site) -> () {
        fn in_order_at<F>(tree: &BeachLine, f: &mut F, at: Pointer) where F: FnMut(&Site) -> () {
            if at.is_null() {
                return;
            }

            in_order_at(tree, f, tree[at].left);
            f(&tree[at].value);
            in_order_at(tree, f, tree[at].right);
        }

        in_order_at(self, &mut f, self.root);
    }

    #[allow(dead_code)]
    pub fn depth(&self) -> usize {
        fn depth_inner(tree: &BeachLine, at: Pointer, depth: usize) -> usize {
            if at.is_null() {
                return depth;
            }

            let left_depth = depth_inner(tree, tree[at].left, depth + 1);
            let right_depth = depth_inner(tree, tree[at].right, depth + 1);
            if left_depth < right_depth { right_depth } else { left_depth }
        }

        depth_inner(self, self.root, 0)
    }

    #[allow(dead_code)]
    pub fn print(&self) {
        print(self.root, |at| {
            let left = self[*at].left;
            if left.is_null() { None } else { Some(left) }
        }, |at| {
            let right = self[*at].right;
            if right.is_null() { None } else { Some(right) }
        }, |at| {
            let node = &self[*at];
            match node.color {
                Color::BLACK => format!("B:{}", node.value.id),
                Color::RED => format!("R:{}", node.value.id),
            }
        });
    }
}
