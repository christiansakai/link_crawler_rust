extern crate html5ever;

use std::string::String;

use self::html5ever::tendril::TendrilSink;
use self::html5ever::parse_document;
use self::html5ever::rcdom::{Handle, NodeData, RcDom};
use self::html5ever::interface::Attribute;

pub fn parse_html(source: &str) -> RcDom {
    parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut source.as_bytes())
        .unwrap()
}

pub fn get_urls(handle: Handle) -> Vec<String> {
    let mut urls = vec![];
    let mut anchor_tags = vec![];

    get_elements_by_name(handle, "a", &mut anchor_tags);

    for node in anchor_tags {
        if let NodeData::Element { attrs, .. } = node {
            for attr in attrs.borrow().iter() {
                let Attribute {
                    name,
                    value,
                } = attr;

                if &(*name.local) == "href" {
                    urls.push(value.to_string());
                }
            }
        };
    }

    urls
}

fn get_elements_by_name(handle: Handle, element_name: &str, output: &mut Vec<NodeData>) {
    let node = handle;

    if let NodeData::Element {
        ref name,
        ref attrs,
        ref template_contents,
        ..
    } = node.data {
        if &(*name.local) == element_name {
            output.push(NodeData::Element {
                name: name.clone(),
                attrs: attrs.clone(),
                template_contents: template_contents.clone(),
                mathml_annotation_xml_integration_point: false,
            });
        }
    };

    for n in node.children.borrow().iter() {
        get_elements_by_name(n.clone(), element_name, output);
    }
}
