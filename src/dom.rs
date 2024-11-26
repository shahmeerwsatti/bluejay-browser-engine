type AttrMap = HashMap<String, String>;
    
struct ElementData
{
    tag_name: String,
    attrs: AttrMap,
}

enum NodeType
{
    // BlueJay's supported node types (ElementData is a custom type of node to support complex HTML
    // components with 'attributes' that are uniquely identified by a map of unique attributes for each
    // unique element
    // CDATA = Untracked parts between 2 tags (Note to self)
    Text(String),
    Element(ElementData),
    Comment(String)
}

struct Node
{
    // Defines a node, i.e: A component of the DOM being rendered by the engine as a vector with
    // several subcomponents of the NodeType enum
    children: Vec<Node>,
    node_type: NodeType,
}

// Constructor function to easily create a new node of the types 'Text', 'Element' or 'Comment'
pub fn text(data: String) -> Node
{
    Node{children: Vec::new(), node_type: NodeType::Text(data)}
}

pub fn elem(tag_name: String, attrs: AttrMap, children:Vec<Node>) -> Node
{
    Node
    {
        children,
        node_type: NodeType::Element(ElementData {tag_name, attrs})
    }
}

pub fn comm(data: String) -> Node
{
    Node{children: Vec::new(), node_type: NodeType::Text(data)}
}

// CDATA is implicitly supported, DocumentFragment is merely supported as a separate DOM tree stored
// in memory
// TODO: Allow custom notation definitions in HTML!!!

impl ElementData
{
    pub fn id(&self) -> Option<&String>
    {
        self.attributes.get("id")
    }

    pub fn classes(&self) -> HashSet<&str>
    {
        match self.attributes.get("class")
        {
            Some(classlist) => classlist.split(" ").collect(),
            None => HashSet::new()
        }
    }
}