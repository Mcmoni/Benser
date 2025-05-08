use std::collections::{HashMap, HashSet};

#[derive(PartialEq, Debug, Clone)]
pub struct Node {
    /// data common to all nodes:
    pub children: Vec<Node>,

    /// data specific to each node type:
    pub node_type: NodeType,
}

impl Node {
    pub fn text(data: String) -> Self {
        Node {
            children: Vec::new(),
            node_type: NodeType::Text(data),
        }
    }

    pub fn elem(name: String, attrs: AttrMap, children: Vec<Node>) -> Self {
        Node {
            children,
            node_type: NodeType::Element(ElementData {
                tag_name: name,
                attributes: attrs,
            }),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum NodeType {
    Text(String),
    Element(ElementData),
}

pub type AttrMap = HashMap<String, String>;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ElementData {
    pub tag_name: String,
    pub attributes: AttrMap,
}

impl ElementData {
    pub fn id(&self) -> Option<&String> {
        self.attributes.get("id")
    }

    pub fn classes(&self) -> HashSet<&str> {
        match self.attributes.get("class") {
            Some(classlist) => classlist.split(' ').collect(),
            None => HashSet::new(),
        }
    }
}
