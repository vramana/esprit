extern crate esprit;

use std::fs::File;
use std::io::prelude::*;

mod pretty {
    struct DocHandle(i32);

    // TODO: concat and union probably should accept Vec<DocHandle>
    enum Doc {
        Concat(Vec<DocHandle>),
        Union(DocHandle, DocHandle),
        Text(String),
        Nest(i32, DocHandle),
        Line,
    }

    struct Document {
        docs: Vec<Doc>,
    }

    impl Document {
        fn push(&mut self, doc: Doc) -> DocHandle {
            self.docs.push(doc);
            DocHandle(self.docs.len() as i32 - 1)
        }

        fn push_handles(&mut self, handles: Vec<DocHandle>) -> DocHandle {
            self.push(Doc::Concat(handles))
        }
    }

    // Example
    struct Tree {
        node: String,
        children: Box<Vec<Tree>>,
    }

    impl Tree {
        fn print_children(&self, document: &mut Document) -> DocHandle {
            let handles = self
                .children
                .iter()
                .map(|c| c.print_tree(document))
                .collect();
            document.push_handles(handles)
        }

        fn print_node(&self, document: &mut Document) -> DocHandle {
            let d = Doc::Text(self.node.clone());
            document.push(d)
        }

        fn print_tree(&self, document: &mut Document) -> DocHandle {
            let node_handle = self.print_node(document);

            let left_bracket_doc = Doc::Text(" { ".to_string());
            let left_bracket_doc_handle = document.push(left_bracket_doc);

            let children_handle = self.print_children(document);

            let right_bracket_doc = Doc::Text(" { ".to_string());
            let right_bracket_doc_handle = document.push(right_bracket_doc);

            document.push_handles(vec![
                node_handle,
                left_bracket_doc_handle,
                children_handle,
                right_bracket_doc_handle,
            ])
        }
    }

    pub fn pretty_print() {
        let a = Tree {
            node: "Hello".to_string(),
            children: Box::new(vec![]),
        };

        let b = Tree {
            node: "world".to_string(),
            children: Box::new(vec![]),
        };

        let c = Tree {
            node: "rust".to_string(),
            children: Box::new(vec![a, b]),
        };

        let mut d = Document { docs: vec![] };

        c.print_tree(&mut d);
    }

}

fn main() {
    let filename = "tests/adhoc/angular.js".to_string();

    println!("In file {}", filename);

    let mut f = File::open(filename).expect("file not found");

    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect("something went wrong reading the file");

    // println!("With text:\n{}", contents);

    let p = esprit::script(&contents);

    if let Ok(_) = p {
        println!("successful parse");
    }

    pretty::pretty_print();
}
