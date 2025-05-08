// This file is based on section 13.2 of the HTML5 specification
// The goal is to implement the parse state (13.2.4)
// https://html.spec.whatwg.org/multipage/parsing.html#parse-state

/// temp
struct Element;

pub struct ParseState<'a> {
    /// The insertion mode is a state variable that controls the primary operation of the tree construction stage.
    ///
    /// https://html.spec.whatwg.org/multipage/parsing.html#the-insertion-mode
    insertion_mode: InsertionMode,
    /// Initially, the stack of open elements is empty. The stack grows downwards; the topmost node on the stack is the first one added to the stack, and the bottommost node of the stack is the most recently added node in the stack.
    ///
    /// https://html.spec.whatwg.org/multipage/parsing.html#the-stack-of-open-elements
    open_elements: Vec<Element>,
    /// Initially, the list of active formatting elements is empty. It is used to handle mis-nested formatting element tags.
    ///
    /// https://html.spec.whatwg.org/multipage/parsing.html#the-list-of-active-formatting-elements
    active_formatting_elements: Vec<Element>,
    /// Once a head element has been parsed (whether implicitly or explicitly) the head element pointer gets set to point to this node.
    ///
    /// https://html.spec.whatwg.org/multipage/parsing.html#the-element-pointers
    head: Option<&'a Element>,
    /// The form element pointer points to the last form element that was opened and whose end tag has not yet been seen. It is used to make form controls associate with forms in the face of dramatically bad markup, for historical reasons.
    ///
    /// https://html.spec.whatwg.org/multipage/parsing.html#the-element-pointers
    form: Option<&'a Element>,
    /// The scripting flag is set to "enabled" if scripting was enabled for the Document with which the parser is associated when the parser was created, and "disabled" otherwise.
    ///
    /// https://html.spec.whatwg.org/multipage/parsing.html#other-parsing-state-flags
    scripting: Scripting,
    /// The frameset-ok flag is set to "ok" when the parser is created. It is set to "not ok" after certain tokens are seen.
    ///
    /// https://html.spec.whatwg.org/multipage/parsing.html#other-parsing-state-flags
    frameset_ok: FramesetOk,
}

impl Default for ParseState<'_> {
    fn default() -> Self {
        ParseState {
            insertion_mode: InsertionMode::Initial,
            open_elements: Vec::new(),
            active_formatting_elements: Vec::new(),
            head: None,
            form: None,
            scripting: Scripting::Disabled,
            frameset_ok: FramesetOk::Ok,
        }
    }
}

enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    InHeadNoscript,
    AfterHead,
    InBody,
    Text,
    InTable,
    InTableText,
    InCaption,
    InColumnGroup,
    InTableBody,
    InRow,
    // lol incel
    InCell,
    InSelect,
    InSelectInTable,
    InTemplate,
    AfterBody,
    InFrameset,
    AfterFrameset,
    AfterAfterBody,
    AfterAfterFrameset,
}

enum Scripting {
    Enabled,
    Disabled,
}

enum FramesetOk {
    Ok,
    NotOk,
}
