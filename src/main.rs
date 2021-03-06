#![allow(unused)]

extern crate esprit;

use std::fs::File;
use std::io::prelude::*;

mod pretty {
    #[derive(Debug, Clone, Copy)]
    struct DocHandle(i32);

    impl DocHandle {
        fn index(&self) -> usize {
            self.0 as usize
        }
    }

    #[derive(Debug)]
    enum Doc {
        Concat(Vec<DocHandle>),
        Group(DocHandle, Mode),
        Text(String),
        Nest(i32, DocHandle),
        Line(LineMode),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum LineMode {
        SoftSpace,
        SoftEmpty,
        Hard,
        Literal,
    }

    use self::LineMode::*;

    impl LineMode {
        fn hard(&self) -> bool {
            return *self == Hard || *self == Literal;
        }

        fn soft(&self) -> bool {
            return *self == SoftEmpty;
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Mode {
        Flat,
        Break,
    }
    use self::Mode::*;

    #[derive(Debug, Clone, Copy)]
    struct Command {
        indent: i32,
        mode: Mode,
        handle: DocHandle,
    }

    // TODO May be estimate the size of output string
    struct Document {
        docs: Vec<Doc>,
        tab_space: i32,
    }

    enum OutText<'a> {
        Text(&'a str),
        LineIndent(i32),
    }

    impl Document {
        fn last(&self) -> i32 {
            self.docs.len() as i32 - 1
        }

        fn push(&mut self, doc: Doc) -> DocHandle {
            self.docs.push(doc);
            DocHandle(self.last())
        }

        fn push_text(&mut self, text: &str) -> DocHandle {
            self.push(Doc::Text(text.to_string()))
        }

        fn push_concat(&mut self, handles: Vec<DocHandle>) -> DocHandle {
            self.push(Doc::Concat(handles))
        }

        fn push_group(&mut self, handle: DocHandle) -> DocHandle {
            self.push(Doc::Group(handle, Flat))
        }

        fn push_indent(&mut self, handle: DocHandle, indent: i32) -> DocHandle {
            self.push(Doc::Nest(indent, handle))
        }

        fn push_soft_space(&mut self) -> DocHandle {
            self.push(Doc::Line(SoftSpace))
        }

        fn push_soft_break(&mut self) -> DocHandle {
            self.push(Doc::Line(SoftEmpty))
        }

        fn push_hard_break(&mut self) -> DocHandle {
            self.push(Doc::Line(Hard))
        }
    }

    impl Document {
        fn fits(&self, next: Command, rest_commands: &Vec<Command>, width: i32) -> bool {
            let mut width = width;
            let mut cmds = vec![next];
            let mut rest_index = rest_commands.len();
            while (width >= 0) {
                match cmds.pop() {
                    None => {
                        if rest_index == 0 {
                            return true;
                        } else {
                            cmds.push(rest_commands[rest_index - 1]);
                            rest_index -= 1;
                            continue;
                        }
                    }
                    Some(cmd) => {
                        let doc = &self.docs[cmd.handle.index()];

                        match doc {
                            &Doc::Concat(ref handles) => {
                                for handle in handles {
                                    let new_cmd = Command {
                                        indent: cmd.indent,
                                        mode: cmd.mode,
                                        handle: *handle,
                                    };
                                    cmds.push(new_cmd);
                                }
                            }
                            &Doc::Text(ref text) => {
                                width -= text.len() as i32;
                            }
                            &Doc::Nest(i, handle) => {
                                let new_cmd = Command {
                                    indent: cmd.indent + i,
                                    mode: cmd.mode,
                                    handle: handle,
                                };
                                cmds.push(new_cmd);
                            }
                            &Doc::Group(handle, mode) => {
                                let new_cmd = Command {
                                    indent: cmd.indent,
                                    mode: mode,
                                    handle: handle,
                                };
                                cmds.push(new_cmd);
                            }
                            &Doc::Line(line_mode) => match cmd.mode {
                                Flat => {
                                    if line_mode == SoftSpace {
                                        width -= 1;
                                    }
                                }
                                Break => return true,
                            },
                            _ => {}
                        }
                    }
                }
            }
            return false;
        }

        fn pretty(&self, max_width: i32) -> String {
            let top_doc_handle = DocHandle(self.docs.len() as i32 - 1);
            let mut cmds = vec![Command {
                indent: 0,
                mode: Break,
                handle: top_doc_handle,
            }];

            let mut out_text = vec![];
            let mut pos = 0;
            let mut should_remeasure = false;

            while let Some(cmd) = cmds.pop() {
                let index = cmd.handle.index();
                let doc = &self.docs[index];

                match *doc {
                    Doc::Concat(ref handles) => {
                        for handle in handles.iter().rev() {
                            let new_cmd = Command {
                                indent: cmd.indent,
                                mode: cmd.mode,
                                handle: *handle,
                            };
                            cmds.push(new_cmd);
                        }
                    }
                    Doc::Text(ref text) => {
                        pos += text.len() as i32;
                        out_text.push(OutText::Text(text))
                    }
                    Doc::Nest(i, handle) => {
                        let new_cmd = Command {
                            indent: cmd.indent + i,
                            mode: cmd.mode,
                            handle: handle,
                        };
                        cmds.push(new_cmd);
                    }
                    Doc::Group(handle, mode) => match cmd.mode {
                        Flat => {
                            if !should_remeasure {
                                let new_cmd = Command {
                                    indent: cmd.indent,
                                    mode: mode,
                                    handle: handle,
                                };
                                cmds.push(new_cmd);
                            }
                        }
                        Break => {
                            should_remeasure = false;

                            let mut next = Command {
                                indent: cmd.indent,
                                mode: Flat,
                                handle: handle,
                            };
                            let remaining_width = max_width - pos;
                            let fits = self.fits(next, &mut cmds, remaining_width);
                            println!("Remaining Width:{}, Fits:{}", remaining_width, fits);

                            if mode == Break || !fits {
                                next.mode = Break;
                            }
                            cmds.push(next)
                        }
                    },
                    Doc::Line(line_mode) => match cmd.mode {
                        Flat => {
                            if !line_mode.hard() {
                                if !line_mode.soft() {
                                    out_text.push(OutText::Text(" "));
                                    pos += 1;
                                }
                            } else {
                                should_remeasure = true;
                            }
                        }
                        Break => {
                            println!("{:?}, {:?}", cmd, doc);
                            // trim multiple new lines to single new line
                            if (out_text.len() > 0) {
                                let index = out_text.len() - 1;
                                let is_last_string_traling_white_space = {
                                    let last_string = &out_text[index];
                                    match last_string {
                                        OutText::Text(last_string) => {
                                            last_string.trim().is_empty()
                                                && last_string.contains("\n")
                                        }
                                        OutText::LineIndent(_) => true,
                                    }
                                };

                                if is_last_string_traling_white_space {
                                    out_text[index] = OutText::Text("\n");
                                }
                            }

                            if line_mode == Literal {
                                out_text.push(OutText::Text("\n"));
                                pos = 0;
                            } else {
                                out_text.push(OutText::LineIndent(cmd.indent));
                                pos = cmd.indent * self.tab_space;
                            }
                        }
                    },
                }
            }

            self.print_out_text(out_text)
        }

        fn print_out_text(&self, out_text: Vec<OutText>) -> String {
            let mut t = String::new();
            for o in out_text {
                match o {
                    OutText::Text(text) => t.push_str(text),
                    OutText::LineIndent(n) => {
                        t.push_str("\n");
                        t.push_str(&" ".repeat((n * self.tab_space) as usize));
                    }
                }
            }
            t
        }
    }

    // Example
    #[derive(Clone)]
    struct Tree {
        node: String,
        children: Box<Vec<Tree>>,
    }

    impl Tree {
        fn print_children(&self, document: &mut Document) -> DocHandle {
            let length = self.children.len();

            if length == 1 {
                return self.children[0].print_tree(document);
            }

            let mut handles = Vec::with_capacity(length * 2 + 1);
            let comma_handle = document.push_text(",");
            for (index, child) in self.children.iter().enumerate() {
                let child_handle = child.print_tree(document);
                handles.push(child_handle);
                if index != length - 1 {
                    handles.push(comma_handle);
                    handles.push(document.push_soft_space());
                }
            }
            document.push_concat(handles)
        }

        fn print_node(&self, document: &mut Document) -> DocHandle {
            document.push_text(&self.node)
        }

        fn print_bracket(&self, document: &mut Document) -> DocHandle {
            // if self.children.len() == 0 {
            //     return document.push_text("");
            // }

            let children_handle = self.print_children(document);
            let soft_space_handle = if self.children.len() == 0 {
                document.push_soft_break()
            } else {
                document.push_soft_space()
            };
            let handle = document.push_concat(vec![soft_space_handle, children_handle]);

            let handles = vec![
                document.push_text("{"),
                document.push_indent(handle, 1),
                soft_space_handle,
                document.push_text("}"),
            ];
            document.push_concat(handles)
        }

        fn print_tree(&self, document: &mut Document) -> DocHandle {
            let bracket_handle = self.print_bracket(document);
            let handles = vec![self.print_node(document), bracket_handle];

            let handle = document.push_concat(handles);
            document.push_group(handle)
        }
    }

    pub fn pretty_print() {
        let a = Tree {
            node: "Hello".to_string(),
            children: Box::new(vec![]),
        };

        let b = Tree {
            node: "world".to_string(),
            children: Box::new(vec![a.clone(), a.clone()]),
        };

        let c = Tree {
            node: "rust".to_string(),
            children: Box::new(vec![a, b.clone(), b]),
        };

        let mut d = Document {
            docs: vec![],
            tab_space: 2,
        };

        c.print_tree(&mut d);

        println!("{:?}", d.docs);
        println!("{}", d.pretty(40));
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

    // let p = esprit::script(&contents);

    // if let Ok(_) = p {
    //     println!("successful parse");
    // }

    pretty::pretty_print();
}
