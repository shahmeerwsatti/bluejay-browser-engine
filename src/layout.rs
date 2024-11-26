// CSS is not just defined by layouts, but also LAYOUTS
// A layout is essentially an element that is a rectangular box with a particular size and position on
// a page, this file takes the HTML code (DOM elements) and CSS code (Stylesheet) and mixes them together
// to (kinda brokenly) render a full webpage
struct Rect
{
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

struct EdgeSizes
{
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

struct Dimensions
{
    content: Rect,
    padding: EdgeSizes,
    border: EdgeSizes,
    margin: EdgeSizes,
}

enum BoxType<'a>
{
    BlockNode(&'a StyledNode <'a>),
    InlineNode(&'a StyledNode<'a>),
    // The Anonymous Block is introduced as an invisible element when Inline and Block nodes are mixed
    // together, as they actually cannot be mixed, so the anonymous node is introduced to create an
    // "invisible" block around the mixed in element so that it is technically separate in the DOM
    // tree
    AnonymousBlock,
}

struct LayoutBox<'a>
{
    dimensions: Dimensions,
    box_type: BoxType<'a>,
    children: Vec<LayoutBox<'a>>
}

// Walks through the stylesheet tree and builds a box for each node, before building boxes for each
// child element of a node
fn build_layout_tree<'a>(style_node: &'a StyledNode<'a>) -> LayoutBox<'a>
{
    let mut root = LayoutBox::new(match style_node.display()
    {
       Display::Block => BlockNode(style_node),
       Display::Inline => InlineNode(style_node),
       Display::None => panic!("Root node has no display: None") 
    });

    for child in &style_node.children
    {
        match child.display()
        {
            Display::Block => root.children.push(build_layout_tree(child)),
            Display::Inline => root.get_inline_container().children.push(build_layout_tree(child)),
            Display::None => {}
            // Don't lay out nodes with the display type none.
        }
    }
    return root;
}

// Defines a function for creating a new Layout Box
impl LayoutBox
{
    fn new(box_type: BoxType) -> LayoutBox
    {
        LayoutBox
        {
            box_type,
            dimensions: Default::default(),
            children: Vec::new(),
        }
    }
}

// When inline and block containers are mixed, this function applies anonymous boxes to ensure seamless
// mixing with each other
fn get_inline_container(&mut self) -> &mut LayoutBox
{
    match self.box_type
    {
        InlineNode(_) | AnonymousBlock => self,
        BlockNode(_) => 
        {
            match self.children.last()
            {
                Some(&LayoutBox {box_type: AnonymousBlock,..}) => {}
                _ => self.children.push(LayoutBox::new(AnonymousBlock))
            }
            self.children.last_mut().unwrap()
        }
    }
}
// VERY SIMPLE, GENERATES AN EXTRA UNNECESSARY BOX
// TODO: Update to standard CSS box generation algorithm