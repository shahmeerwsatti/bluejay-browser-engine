// The BlueJay engine creates a tree of associated CSS properties pointing to each defined
// node in the DOM with a PropertyMap which assigns it some value for all properties currently supported
// in the limited CSS 2.1 engine of BlueJay
pub type PropertyMap = HashMap<String, Value>;

// 'a is used as a memory dereferencing mechanism to pass ownership without killing the function entirely
// because I hate Rust lifetimes!!!
pub struct StyledNode<'a>
{
    node: &'a Node,
    specified_values: PropertyMap,
    children: Vec<StyledNode<'a>>
}

pub enum Display
{
    Inline,
    Block,
    None
}

// Gets the display value for a node and returns it to the Layout module, if empty, returns inline by default
// as the layout mechanism
impl StyledNode
{
    fn value(&self, name: &str) -> Option<Value>
    {
        self.specified_values.get(name).map(|v| v.clone())
    }

    fn display(&self) -> Display
    {
        match self.value("display")
        {
            Some(Keyword(s)) => match &*s
            {
                "block" => Display::Block,
                "none" => Display::None,
                _ => Display::Inline
            },
            _ => Display::Inline
        }
    }
}

// Returns an id, and/or a hash table of space-separated classes, but more IMPORTANTLY, it notes whether
// the node has an id/class at all (i.e: If it doesn't match a selector, id OR class, it returns false),
// and otherwise returns true
fn matches_simple_selector(elem: &ElementData, selector: &SimpleSelector) -> bool
{
    // Checks the type of the CSS element being defined (External, internal or inline, but BlueJay does
    // NOT support external)
    if selector.tag_name.iter().any(|name| elem.tag_name != *name)
    {
        return false;
    }

    // Checks the ID of the element
    if selector.id.iter().any(|id| elem.id() != Some(id))
    {
        return false;
    }

    // Checks the class of the element
    if selector.class.iter().any(|class| !elem.classes().contains(class.as_str()))
    {
        return false;
    }

    true
}

// Observing a DOM module node's CSS properties is relatively simple in BlueJay, as the engine doesn't
// support encapsulation or inheritance of nodes, so no need to use a tree structure to traverse any
// particular element's parent/children properties, because all of them are directly specified in that
// element's CSS tag due to a lack of complex selectors.
fn matches(elem: &ElementData, selector: &Selector) -> bool
{
    match selector
    {
        Simple(s) => matches_simple_selector(elem, s)
    }
}

// See as below
type MatchedRule<'a> = (Specificity, &'a Rule);

// SPECIFICITY RETURNS!!!!!!!!11!!! 
// Searches the tree of DOM elements/nodes and grabs any associated stylesheet tree for matching CSS
// or "rules", due to the fact we store specificity from highest to lowest (i.e: Priority when there
// are multiple CSS selectors) we need only grab the first element in the list and return a pointer
// to it, HOWEVER!! this function only returns the element AND its specificity (i.e: Pos. in the list)
// where we put them into a HashMap later on to process
fn match_rule<'a>(elem: &ElementData, rule: &Rule) -> Option<MatchedRule<'a>>
{
    rule.selectors.iter().find(|selector| matches(elem, selector)).map(|selector| (selector.specificity(), rule))
}

// Traverses the rule Vector for a certain node in the DOM and checks the rules, whether they belong, and
// if not throws them out and creates a new vector for matched rules
// TODO: When updating HTML/CSS spec support, add hash-table based lookup to speed up memory access
// and processing times and to increase efficiency
fn matching_rules<'a>(elem: &ElementData, stylesheet: &'a Stylesheet) -> Vec<MatchedRule<'a>>
{
    stylesheet.rules.iter().filter_map(|rule| match_rule(elem.rule)).collect()
}

// Inserts each rule for the CSS properties of an element into a HashMap along with its accompanying
// specificity and sorts the HashMap by order of specificity before processing each rule in order of
// specificity, so that more specific (a) rules can overwrite less specific (b) rules (a > b) in the HashMap
fn specified_rules(elem: &ElementData, stylesheet: &Stylesheet) -> PropertyMap
{
    let mut values = HashMap::new();
    let mut rules = matching_rules(elem, stylesheet);

    rules.sort_by(|&(a, _), &(b, _)| a.cmp(&b));

    for (_, rule) in rules
    {
        for declaration in &rule.declarations
        {
            values.insert(declaration.name.clone(), declaration.value.clone());
        }
    }
    return values;
}

// Takes all the data we just collected and constructs a style tree, with a node in the style tree associated
// with each element in the DOM tree
// NO SUPPORT FOR FONTS/TEXT STYLES SO NO SUPPORT FOR CSS TEXT, which has an empty HashMap of rules associated
// with it, only elements can have attributes in their CSS
pub fn style_tree<'a>(root: &'a Node, stylesheet: &'a Stylesheet) -> StyledNode<'a>
{
    StyledNode
    {
        node: root,
        specified_values: match root.node_type
        {
            Element(ref elem) => specified_values(elem, stylesheet),
            Text(_) => HashMap::new()
        },
        children: root.children.iter().map(|child| style_tree(child, stylesheet)).collect(),
    }
}

// TODO: Cascading
// TODO: Inheritance
// TODO: Implementation of the style attribute