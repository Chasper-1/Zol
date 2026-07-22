use super::*;
use crate::markup::segment::STYLE_BOLD;
use crate::markup::segment::STYLE_PLAIN;
use zoll::ast::{MarkupDoc, MarkupNode, MarkupStyle};

#[test]
fn plain_text_segments() {
    let doc = MarkupDoc {
        children: vec![MarkupNode::Text("hello".to_string())],
    };
    let cache = to_document_cache(&doc);
    assert_eq!(cache.lines.len(), 1);
    assert_eq!(cache.lines[0].segments.len(), 1);
    assert_eq!(cache.lines[0].segments[0].text, "hello");
    assert_eq!(cache.lines[0].segments[0].style, STYLE_PLAIN);
}

#[test]
fn multiline_formatted_correct_line_assignment() {
    let doc = MarkupDoc {
        children: vec![
            MarkupNode::Text("a\n".to_string()),
            MarkupNode::Formatted {
                style: MarkupStyle::BOLD,
                children: vec![MarkupNode::Text("bold".to_string())],
            },
            MarkupNode::Text("\nc".to_string()),
        ],
    };
    let cache = to_document_cache(&doc);
    assert_eq!(cache.lines.len(), 3, "should be 3 lines");
    assert_eq!(cache.lines[0].segments.len(), 1);
    assert_eq!(cache.lines[0].segments[0].text, "a");
    assert_eq!(cache.lines[1].segments.len(), 1);
    assert_eq!(cache.lines[1].segments[0].text, "bold");
    assert_ne!(cache.lines[1].segments[0].style & STYLE_BOLD, 0, "line 1 should be BOLD");
    assert_eq!(cache.lines[2].segments.len(), 1);
    assert_eq!(cache.lines[2].segments[0].text, "c");
}

#[test]
fn bold_segment_raw_positions() {
    let doc = MarkupDoc {
        children: vec![MarkupNode::Formatted {
            style: MarkupStyle::BOLD,
            children: vec![MarkupNode::Text("bold".to_string())],
        }],
    };
    let cache = to_document_cache(&doc);
    assert_eq!(cache.lines.len(), 1);
    assert_eq!(cache.lines[0].segments.len(), 1);
    let seg = &cache.lines[0].segments[0];
    assert_eq!(seg.text, "bold");
    assert_eq!(seg.raw_start, 2);
    assert_eq!(seg.raw_end, 6);
}
