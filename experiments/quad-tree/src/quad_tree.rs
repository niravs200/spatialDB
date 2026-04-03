#[derive(Clone)]
struct Coordinate {
    x: i64,
    y: i64,
}

#[derive(Clone)]
struct Entry<T> {
    coordinate: Coordinate,
    data: T,
}

// Min represents top-left corner, max represents bottom-right
#[derive(Clone)]
struct Range {
    min: Coordinate,
    max: Coordinate,
}

impl Range {
    fn contains(&self, point: &Coordinate) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    fn intersects(&self, other: &Range) -> bool {
        !(self.max.x < other.min.x
            || self.min.x > other.max.x
            || self.max.y < other.min.y
            || self.min.y > other.max.y)
    }
}

struct Node<T> {
    range: Range,
    capacity: u64,
    entries: Vec<Entry<T>>,
    children: Option<[Box<Node<T>>; 4]>,
}

impl<T: Clone> Node<T> {
    pub fn new(capacity: u64, range: Range) -> Self {
        Self {
            range,
            capacity,
            entries: Vec::new(),
            children: None,
        }
    }

    pub fn get(&self, query: &Range) -> Option<Vec<Entry<T>>> {
        if !self.range.intersects(query) {
            return None;
        }

        if self.children.is_none() {
            let filtered: Vec<Entry<T>> = self
                .entries
                .iter()
                .filter(|e| query.contains(&e.coordinate))
                .cloned()
                .collect();

            if filtered.is_empty() {
                None
            } else {
                Some(filtered)
            }
        } else {
            let mut result = Vec::new();

            if let Some(children) = &self.children {
                for child in children {
                    if let Some(mut child_result) = child.get(query) {
                        result.append(&mut child_result);
                    }
                }
            }

            if result.is_empty() {
                None
            } else {
                Some(result)
            }
        }
    }

    pub fn insert(&mut self, entry: &Entry<T>) -> bool {
        if !self.range.contains(&entry.coordinate) {
            return false;
        }

        if self.children.is_none() {
            if (self.entries.len() as u64) < self.capacity {
                self.entries.push(entry.clone());
                return true;
            } else {
                self.subdivide();
            }
        }

        if let Some(children) = self.children.as_mut() {
            for child in children.iter_mut() {
                if child.range.contains(&entry.coordinate) {
                    return child.insert(entry);
                }
            }
        }

        false
    }

    fn subdivide(&mut self) {
        let mid_x = (self.range.min.x + self.range.max.x) / 2;
        let mid_y = (self.range.min.y + self.range.max.y) / 2;

        let nw = Node::new(
            self.capacity,
            Range {
                min: self.range.min.clone(),
                max: Coordinate { x: mid_x, y: mid_y },
            },
        );
        let ne = Node::new(
            self.capacity,
            Range {
                min: Coordinate {
                    x: mid_x + 1,
                    y: self.range.min.y,
                },
                max: Coordinate {
                    x: self.range.max.x,
                    y: mid_y,
                },
            },
        );
        let sw = Node::new(
            self.capacity,
            Range {
                min: Coordinate {
                    x: self.range.min.x,
                    y: mid_y + 1,
                },
                max: Coordinate {
                    x: mid_x,
                    y: self.range.max.y,
                },
            },
        );
        let se = Node::new(
            self.capacity,
            Range {
                min: Coordinate {
                    x: mid_x + 1,
                    y: mid_y + 1,
                },
                max: self.range.max.clone(),
            },
        );

        self.children = Some([Box::new(nw), Box::new(ne), Box::new(sw), Box::new(se)]);

        let old_entries = std::mem::take(&mut self.entries);
        for entry in old_entries {
            self.insert_into_correct_child(&entry);
        }
    }

    fn insert_into_correct_child(&mut self, entry: &Entry<T>) {
        if let Some(children) = self.children.as_mut() {
            for child in children.iter_mut() {
                if child.range.contains(&entry.coordinate) {
                    let _ = child.insert(entry);
                    return;
                }
            }
        }
    }


    fn get_node_from_coordinates(&mut self, coordinate: &Coordinate) -> Option<&mut Node<T>>{
        if !self.range.contains(coordinate) {
            return None
        } 

        if self.children.is_none() {
            return self
        }

        if let Some(children) = self.children {
            for child in self.children.iter() {
                if let Some(node) = child.get_node_from_coordinates_mut(coordinate) {
                    return Some(node);
                }
            }
        }

        return None
    }

    pub fn update(&mut self, entry: &Entry<T>) {
        if let Some(node) = self.get_node_from_coordinates_mut(&entry.coordinate) {
            for e in node.entries.iter_mut() {
                if e.coordinate.x == entry.coordinate.x && e.coordinate.y == entry.coordinate.y {
                    e.data = entry.data.clone();
                    return true;
                }
            }
        }
        false
    }
}