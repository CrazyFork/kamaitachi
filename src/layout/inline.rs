use std::mem;
use std::collections::{VecDeque};
use std::ops::Range;
use super::{Dimensions, Rect, LayoutBox, BoxType, TextNode};
use crate::dom::{NodeType};
use crate::cssom::{Value, Unit};

#[derive(Clone)]
struct Line {
  range: Range<usize>,
  bounds: Dimensions,
  green_zone: Rect,
}

impl Line {
  pub fn new(bounds: Dimensions) -> Line {
    Line { range: 0..0, bounds, green_zone: Default::default() }
  }
}

struct LineBreaker<'a> {
  work_list: VecDeque<LayoutBox<'a>>,
  new_boxes: Vec<LayoutBox<'a>>,
  lines: Vec<Line>,
  pending_line: Line,
  // Largest width in each lines
  max_width: f32,
  cur_height: f32,
}

impl<'a> LineBreaker<'a> {
  fn new() -> LineBreaker<'a> {
    LineBreaker { 
      work_list: VecDeque::new(),
      new_boxes: vec![],
      lines: vec![],
      pending_line: Line::new(Default::default()),
      max_width: 0.0,
      cur_height: 0.0,
    }
  }

  fn scan_for_line(&mut self, root: &LayoutBox<'a>, old_boxes: Vec<LayoutBox<'a>>) {
    self.layout_list(root, old_boxes);
    loop {
      match self.work_list.pop_front() {
        Some(item) => self.new_boxes.push(item),
        None => break,
      };
    }
  }

  pub fn layout_list(&mut self, root: &LayoutBox<'a>, old_boxes: Vec<LayoutBox<'a>>) {
    for layout_box in &old_boxes {
      self.layout(root, &layout_box);
    }

    if !self.pending_line_is_empty() {
      self.lines.push(self.pending_line.clone());
      self.pending_line.range = 0..0;
    }
  }

  fn layout(&mut self, root: &LayoutBox<'a>, layout_box: &LayoutBox<'a>) {
    if self.pending_line_is_empty() {
      let line_bounds = self.initial_line_placement(root, layout_box);
      self.pending_line.bounds.content.x = line_bounds.content.x;
      self.pending_line.bounds.content.y = line_bounds.content.y;
      self.pending_line.green_zone.width = line_bounds.margin_box().width;
    }

    // TODO: Check inline box is fit in green_zone
    self.pending_line.range.end += 1;

    match &layout_box.box_type {
      BoxType::InlineNode(node) => {
        // TODO: height, cur_height, max_width, width, margin, padding, border
        for child in &layout_box.children {
          self.layout(root, child);
        }

        let mut total_width = 0.0;
        let mut max_height = 0.0;
        for child in &layout_box.children {
          let mut d = child.dimensions.borrow_mut();
          let margin_box = d.margin_box();
          d.content.x = total_width;
          total_width += margin_box.width;
          max_height = margin_box.height.max(max_height);
          self.work_list.pop_back();
          self.pending_line.range.end -= 1;
        }

        let mut d = layout_box.dimensions.borrow_mut();
        d.content.width = total_width;
        
        let zero = Value::Length(0.0, Unit::Px);

        let margin_left = node.lookup("margin-left", "margin", &zero);
        let margin_right = node.lookup("margin-right", "margin", &zero);
    
        let border_left = node.lookup("border-left-width", "border", &zero);
        let border_right = node.lookup("border-right-width", "border", &zero);
    
        let padding_left = node.lookup("padding-left", "padding", &zero);
        let padding_right = node.lookup("padding-right", "padding", &zero);

        d.margin.left = margin_left.to_px();
        d.margin.right = margin_right.to_px();
        
        d.padding.left = padding_left.to_px();
        d.padding.right = padding_right.to_px();

        d.border.left = border_left.to_px();
        d.border.right = border_right.to_px();

        d.content.height = max_height;

        let margin_top = node.lookup("margin-top", "margin", &zero);
        let margin_bottom = node.lookup("margin-bottom", "margin", &zero);
    
        let border_top = node.lookup("border-top-width", "border", &zero);
        let border_bottom = node.lookup("border-bottom-width", "border", &zero);
    
        let padding_top = node.lookup("padding-top", "padding", &zero);
        let padding_bottom = node.lookup("padding-bottom", "padding", &zero);

        d.margin.top = margin_top.to_px();
        d.margin.bottom = margin_bottom.to_px();
        
        d.padding.top = padding_top.to_px();
        d.padding.bottom = padding_bottom.to_px();

        d.border.top = border_top.to_px();
        d.border.bottom = border_bottom.to_px();
        self.work_list.push_back(layout_box.clone());
      },
      BoxType::TextNode(node) => {
        let mut d = layout_box.dimensions.borrow_mut();
        d.content.width = self.text_width(&node);
        // TODO: calculate leading
        // let (above_baseline, under_baseline) = font.height(line_height)
        d.content.height = node.styled_node.line_height();
        self.work_list.push_back(layout_box.clone());
      },
      BoxType::BlockNode(_) => unimplemented!(),
      _ => unreachable!(),
    }
  }

  fn initial_line_placement(&self, root: &LayoutBox, _layout_box: &LayoutBox) -> Dimensions {
    // refer: https://github.com/servo/servo/blob/3f7697690aabd2d8c31bc880fcae21250244219a/components/layout/inline.rs#L500
    // let width = if layout_box.can_split() {
    //   self.minimum_splittable_inline_width(&layout_box)
    // } else {
    //   // TODO: for `block box` and `inline block box`
    //   unimplemented!();
    // };

    // TODO: calculate float dimensions
    root.dimensions.borrow().clone()
  }

  fn text_width(&self, node: &TextNode<'a>) -> f32 {
    let styled_node = node.styled_node;
    let text = if let NodeType::Text(text) = &styled_node.node.node_type {
      text
    } else {
      unreachable!();
    };
    // TODO: optimize to load font only once
    node.font.width(text)
  }

  fn pending_line_is_empty(&self) -> bool {
    self.pending_line.range.end == 0
  }
}

pub struct InlineBox<'a> {
  pub root: LayoutBox<'a>,
  pub boxes: Vec<LayoutBox<'a>>,
  pub width: f32,
  pub height: f32,
}

impl<'a> InlineBox<'a> {
  pub fn new(root: LayoutBox<'a>, boxes: Vec<LayoutBox<'a>>) -> InlineBox<'a> {
    InlineBox { root, boxes, width: 0.0, height: 0.0 }
  }

  pub fn layout(&mut self) {
    let mut line_breaker = LineBreaker::new();
    let old_boxes = mem::replace(&mut self.boxes, Vec::new());
    line_breaker.scan_for_line(&self.root, old_boxes);
    self.assign_position(&mut line_breaker);
    self.boxes = line_breaker.new_boxes;
    self.width = line_breaker.max_width;
    self.height = line_breaker.cur_height;
  }

  fn recursive_position(&self, layout_box: &mut LayoutBox<'a>, additional_x: f32, additional_y: f32) {
    for child in &mut layout_box.children {
      if let BoxType::InlineNode(_) = child.box_type {
        self.recursive_position(child, additional_x, additional_y);
      }
      let mut d = child.dimensions.borrow_mut();
      d.content.x += additional_x;
      d.content.y += additional_y;
    }
  }

  fn assign_position(&self, line_breaker: &mut LineBreaker<'a>) {
    for line in &line_breaker.lines {
      let mut line_box_x = line.bounds.content.x;
      let mut line_box_height = 0.;
      for item in &mut line_breaker.new_boxes[line.range.clone()] {
        {
          let mut d = item.dimensions.borrow_mut();
          d.content.x = line_box_x;
          d.content.y = line.bounds.content.y;
        }
        if let BoxType::InlineNode(_) = item.box_type {
          self.recursive_position(item, line_box_x, line.bounds.content.y);
        }
        let d = item.dimensions.borrow();
        let margin_box = d.margin_box();
        let line_box_width = margin_box.width;
        line_box_x += line_box_width;
        line_breaker.max_width = line_box_width.max(line_breaker.max_width);
        line_box_height = margin_box.height.max(line_box_height);
      }
      line_breaker.cur_height += line_box_height;
    }
  }
}

impl<'a> LayoutBox<'a> {
  /// Block box should be processed as a line box in inline layout process,
  /// but block box is treated as a line box.
  fn can_split(&self) -> bool {
    match self.box_type {
      BoxType::TextNode(_) => true,
      _ => false,
    }
  }
}