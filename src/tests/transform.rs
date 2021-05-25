use prosemirror_model::{Fragment, Node, Slice};
use prosemirror_transform::{AddMarkStep, ReplaceStep, Span, Step, StepKind};
use prosemirror_markdown::{
    helper::{doc, node, p, strong},
    MarkdownMark, MarkdownNode, MD,
};

#[test]
fn test_apply() {
    let d1 = doc(p("Hello World!"));
    let step1 = AddMarkStep::<MD> {
        span: Span { from: 1, to: 9 },
        mark: MarkdownMark::Strong,
    };
    let d2 = step1.apply(&d1).unwrap();
    assert_eq!(d2, doc(p(vec![strong("Hello Wo"), node("rld!")])));
}

#[test]
fn test_deserialize() {
    let s1: Step<MD> =
        serde_json::from_str(r#"{"stepType":"addMark","mark":{"type":"em"},"from":61,"to":648}"#)
            .unwrap();

    assert_eq!(
        s1,
        Step::AddMark(AddMarkStep {
            span: Span { from: 61, to: 648 },
            mark: MarkdownMark::Em,
        })
    );

    let s2: Step<MD> = serde_json::from_str(
            r#"{"stepType":"replace","from":986,"to":986,"slice":{"content":[{"type":"text","text":"!"}]}}"#
        ).unwrap();

    assert_eq!(
        s2,
        Step::Replace(ReplaceStep {
            span: Span { from: 986, to: 986 },
            slice: Slice {
                content: Fragment::from((MarkdownNode::text("!"),)),
                open_start: 0,
                open_end: 0,
            },
            structure: false,
        })
    );
}
