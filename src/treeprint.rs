use std::collections::VecDeque;

#[derive(Debug)]
struct NodePrintData {
    space_left: i32,
    space_right: i32,
    bar_left_width: i32,
    bar_right_width: i32,
    node_width: i32,
    node_value: Box<String>,
    children_count: i32,
    left: Box<Option<NodePrintData>>,
    right: Box<Option<NodePrintData>>
}

struct Printer<T, F1: Fn(&T) -> Option<T>, F2: Fn(&T) -> Option<T>, F3: Fn(&T) -> String> {
    get_left: F1,
    get_right: F2,
    to_string: F3,
    blank: Option<T>
}

impl<T, F1, F2, F3> Printer<T, F1, F2, F3> where F1: Fn(&T) -> Option<T>, F2: Fn(&T) -> Option<T>, F3: Fn(&T) -> String {
    fn merge_node_print_data(&self, left: NodePrintData, right: NodePrintData, node: T) -> NodePrintData {
        let value = (self.to_string)(&node);
        return NodePrintData {
            space_left: left.space_left + left.node_width + left.space_right,
            space_right: right.space_left + right.node_width + right.space_right,
            bar_left_width: left.space_right,
            bar_right_width: right.space_left,
            node_width: value.len() as i32,
            node_value: Box::new(value),
            children_count: left.children_count + right.children_count + 1,
            left: Box::new(Some(left)),
            right: Box::new(Some(right))
        };
    }

    fn node_print_data_from_tree(&self, node: Option<T>, depth: usize) -> NodePrintData {
        if node.is_none() {
            NodePrintData {
                space_left: 0,
                space_right: 0,
                bar_left_width: 0,
                bar_right_width: 0,
                node_width: 0,
                node_value: Box::new(String::from("")),
                children_count: 0,
                left: Box::new(None),
                right: Box::new(None)
            }
        } else {
            let node = node.unwrap();
            let left = (self.get_left)(&node);
            let right = (self.get_right)(&node);
            let left_node_print_data = self.node_print_data_from_tree(left, depth + 1);
            let right_node_print_data = self.node_print_data_from_tree(right, depth + 1);
            let node_print_data = self.merge_node_print_data(left_node_print_data, right_node_print_data, node);
            return node_print_data;
        }
    }

    fn print_repeat(&self, c: char, count: i32) {
        for _ in 0..(count) {
            print!("{}", c);
        }
    }

    fn print_node_print_data(&self, node_print_data: &NodePrintData) {
        self.print_repeat(' ', node_print_data.space_left - node_print_data.bar_left_width);
        if node_print_data.bar_left_width > 0 {
            print!("\x08.");
        }
        self.print_repeat('-', node_print_data.bar_left_width);
        print!("{}", *node_print_data.node_value);
        self.print_repeat('-', node_print_data.bar_right_width);
        if node_print_data.bar_right_width > 0 {
            print!("\x08.");
        }
        self.print_repeat(' ', node_print_data.space_right - node_print_data.bar_right_width);
    }

    fn print(&self, root: T) {
        let node_print_data = self.node_print_data_from_tree(Some(root), 0);

        // Breadth first tree traversal to print tree

        let mut queue: VecDeque<(i32, i32, NodePrintData)> = VecDeque::with_capacity(node_print_data.children_count as usize);
        let mut current_depth = 0;
        let mut current_x = 0;

        queue.push_back((0, 0, node_print_data));

        while queue.len() > 0 {
            let (depth, x, current_node) = queue.pop_front().unwrap();
            if depth > current_depth {
                print!("\n");
                current_depth = depth;
                current_x = 0;
            }
            self.print_repeat(' ', x - current_x);
            current_x = x;
            self.print_node_print_data(&current_node);
            match *current_node.left {
                Some(node_print_data) => {
                    if node_print_data.node_width > 0 {
                        // Don't pad left nodes
                        queue.push_back((depth + 1, current_x, node_print_data));
                    }
                }
                _ => {}
            }
            current_x += current_node.space_left + current_node.node_width;
            match *current_node.right {
                Some(node_print_data) => {
                    if node_print_data.node_width > 0 {
                        // Add node_width of padding before right node
                        queue.push_back((depth + 1, current_x, node_print_data));
                    }
                }
                _ => {}
            }
            current_x += current_node.space_right;
        }
        print!("\n");
    }
}

pub fn print<T, F1, F2, F3>(root: T, get_left: F1, get_right: F2, to_string: F3)
    where
        F1: Fn(&T) -> Option<T>,
        F2: Fn(&T) -> Option<T>,
        F3: Fn(&T) -> String
{
    let printer = Printer {
        get_left,
        get_right,
        to_string,
        blank: None
    };

    printer.print(root);
}
