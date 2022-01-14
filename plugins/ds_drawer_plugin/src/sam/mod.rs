use std::{collections::HashMap, fmt::Display};

use dot_writer::Attributes;

pub struct SAMPool {
    pub nodes: Vec<Box<SAMNode>>,
    pub root: *mut SAMNode,
    pub last: *mut SAMNode,
    pub str_ids: Vec<i32>,
}

// pub type NodeRef = Weak<RefCell<SAMNode>>;
pub type NodePtr = *mut SAMNode;
#[derive(Debug)]
pub struct SAMNode {
    pub link: NodePtr,
    pub chds: HashMap<char, NodePtr>,
    pub max_len: i32,
    // 字符串ID -> right_size
    pub right_size: HashMap<i32, i32>,
    pub vtx_id: i32,
    pub accept: bool,
    pub chd_on_tree: Vec<NodePtr>,
    pub self_ptr: NodePtr,
}

impl Display for SAMNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = String::new();
        buf.push_str(
            format!(
                "SAMNode{{ID:{},max_len:{},right_size:{:?},link=",
                self.vtx_id, self.max_len, self.right_size
            )
            .as_str(),
        );
        if self.link.is_null() {
            buf.push_str("null,");
        } else {
            unsafe {
                buf.push_str(format!("<id={}>,", (*self.link).vtx_id).as_str());
            }
        }
        for (chr, vtx) in self.chds.iter() {
            buf.push_str(format!("{}->{},", *chr, unsafe { (*(*vtx)).vtx_id }).as_str());
        }
        buf.push('}');
        f.write_str(buf.as_str())?;
        Ok(())
    }
}

impl Default for SAMNode {
    fn default() -> Self {
        Self {
            link: std::ptr::null_mut(),
            chds: Default::default(),
            max_len: 0,
            right_size: Default::default(),
            vtx_id: 1,
            accept: false,
            chd_on_tree: Default::default(),
            self_ptr: std::ptr::null_mut(),
        }
    }
}
impl SAMNode {
    pub fn sam_clone(&self) -> SAMNode {
        let cloned = SAMNode {
            accept: false,
            right_size: Default::default(),
            chds: self.chds.clone(),
            link: self.link.clone(),
            max_len: self.max_len,
            vtx_id: -1,
            chd_on_tree: vec![],
            self_ptr: std::ptr::null_mut(),
        };
        return cloned;
    }
}
impl Default for SAMPool {
    fn default() -> Self {
        let mut root_node = Box::new(SAMNode::default());
        let addr = &mut *root_node as *mut SAMNode;
        root_node.self_ptr = addr;
        Self {
            nodes: vec![root_node],
            root: addr,
            last: addr,
            str_ids: vec![],
        }
    }
}
impl SAMPool {
    pub fn dfs(&self, vtx: *mut SAMNode) {
        unsafe {
            for chd in (*vtx).chd_on_tree.iter() {
                self.dfs(*chd);
                for (k, v) in (*(*chd)).right_size.iter() {
                    let raw_val = (*vtx).right_size.get(k).map(|v| *v).unwrap_or(0);
                    (*vtx).right_size.insert(*k, raw_val + *v);
                }
            }
        }
    }
    pub fn collect(&mut self) {
        for node in self.nodes.iter() {
            if !node.link.is_null() {
                unsafe {
                    (*node.link).chd_on_tree.push(node.self_ptr);
                }
                for id in self.str_ids.iter() {
                    if !node.right_size.contains_key(id) {
                        unsafe {
                            (*node.self_ptr).right_size.insert(*id, 0);
                        }
                    }
                }
            }
        }
        self.dfs(self.root);
    }
    pub fn join_string(&mut self, text: &str, str_id: i32) {
        self.last = self.root;
        for chr in text.chars() {
            unsafe {
                self.append(chr, str_id);
            }
        }
    }
    pub unsafe fn append(&mut self, chr: char, str_id: i32) {
        let new = {
            let mut new = Box::new(SAMNode::default());
            new.accept = true;
            new.max_len = (*self.last).max_len + 1;
            new.right_size.insert(str_id, 1);
            new.vtx_id = self.nodes.len() as i32 + 1;
            let ptr = &mut *new as NodePtr;
            new.self_ptr = ptr;
            self.nodes.push(new);
            ptr
        };

        let mut curr = self.last;
        while !curr.is_null() && !(*curr).chds.contains_key(&chr) {
            (*curr).chds.insert(chr, new);
            curr = (*curr).link;
        }
        if curr.is_null() {
            (*new).link = self.root;
        } else if (*(*(*curr).chds.get(&chr).unwrap())).max_len == (*curr).max_len + 1 {
            (*new).link = *(*curr).chds.get(&chr).unwrap();
        } else {
            let old_node = *(*curr).chds.get(&chr).unwrap();
            let new_node = {
                let mut new_node_ptr = Box::new((*old_node).sam_clone());
                new_node_ptr.vtx_id = self.nodes.len() as i32 + 1;
                let ptr = &mut *new_node_ptr as NodePtr;
                new_node_ptr.self_ptr = ptr;
                self.nodes.push(new_node_ptr);
                ptr
            };
            (*new).link = new_node;
            (*old_node).link = new_node;
            (*new_node).max_len = (*curr).max_len + 1;
            while !curr.is_null() && ((*curr).chds[&chr]) == old_node {
                (*curr).chds.insert(chr, new_node);
                curr = (*curr).link;
            }
        }
        self.last = new;
    }
    pub fn generate_graph(&mut self) -> Vec<u8> {
        let mut out_buf = Vec::<u8>::new();
        {
            use dot_writer::{Color, DotWriter};
            let mut writer = DotWriter::from(&mut out_buf);
            let mut digraph = writer.digraph();
            {
                self.nodes.sort_by_key(|v| v.vtx_id);
                for node in self.nodes.iter() {
                    let mut label = format!("{}\nMax={}", node.vtx_id, node.max_len);
                    for (k, v) in node.right_size.iter() {
                        label.push_str(format!("\nsize{}={}", k, v).as_str());
                    }
                    {
                        let mut curr_node = digraph.node_named(node.vtx_id.to_string());
                        curr_node.set_label(label.as_str());
                    }
                    {
                        if !node.link.is_null() {
                            let link_id = unsafe { (*node.link).vtx_id };
                            digraph
                                .edge(node.vtx_id.to_string(), link_id.to_string())
                                .attributes()
                                .set_color(Color::Red);
                        }
                    }
                    {
                        for (k, v) in node.chds.iter() {
                            let link_id = unsafe { (*(*v)).vtx_id };
                            digraph
                                .edge(node.vtx_id.to_string(), link_id.to_string())
                                .attributes()
                                .set_label(&String::from(*k));
                        }
                    }
                }
            }
        }
        return out_buf;
    }
}
