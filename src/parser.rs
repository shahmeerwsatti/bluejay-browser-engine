
// The struct parser defines the current line of HTML being read as input by the implementation of
// the parser, input is self-explanatory, pos is current position of the indexer across the string
struct Parser
{
    pos: usize,
    input: String,
}
    
struct SimpleSelector
{
   tag_name: Option<String>,
   id: Option<String>,
   class: Vec<String>,
}

// Only simple selectors, no combinator-joined complex selectors for now
enum Selector
{
    Simple(SimpleSelector)
}

// KV pair that begins with a colon and ends with a semicolon declaring a selector's identity/value
struct Declaration
{
    name: String,
    value: Value,
}

// Valid rule structure, selectors separated by commas, declarations after selectors in a series
// wrapped in braces
struct Rule
{
    selectors: Vec<Selector>,
    declarations: Vec<Declaration>,
    // FUTURE SUPPORT FOR COMBINATORS
}

// Stylesheets are a series of RULES defining the display of each element in the DOM by node
struct Stylesheet
{
    rules:Vec<Rule>,
}

struct Color
{
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

enum Value
{
    Keyword(string),
    Length(f32, Unit),
    ColorValue(Color),
    // Add more stuff here for future updates
}

enum Unit
{
    Px,
}

// Explained later in the Implementation of 'Selector'
pub type Specificity = (usize, usize, usize);
// CSS NOT SUPPORTED: @ rules, comments and selectors/values/units/declarations NOT as above
// Limited HTML support too (for now at least)

impl Copy for Color {}

// Returns the size of a length in pixels except for non-lengths, in which case it returns 0
impl Value
{
   pub fn to_px(&self) -> f32
   {
    match *self
    {
        Value::Length(f, Unit::Px) => f, _ => 0.0
    }
   } 
}

impl Parser
{
    // Methods for reading adjacent characters
    // Reads current character without consuming its ownership
    fn next_char(&self) -> char
    {
        self.input[self.pos..].chars().next().unwrap()
    }

    // Checks whether the next characters start with the provided string
    fn starts_with(&self, s: &str) -> bool
    {
        self.input[self.pos..].starts_with(s)
    }

    // Checks if exact string is found at current position and consumes its ownership, otherwise panics
    fn expect(&mut self, s: &str)
    {
        if self.starts_with(s)
        {
            self.pos += s.len();
        }
        else
        {panic!("Expected {:?} at byte {} but it was not found", s, self.pos)};
    }

    // Checks if all strings have had their ownership consumed
    fn eof(&self) -> bool
    {
        self.pos >= self.input.len()
    }

    // Returns the current character, consumes its ownership and advances by one character
    fn consume_char(&mut self) -> char
    {
        let c = self.next_char();
        self.pos += c.len_utf8;
        return c;
    }

    // Consumes a series of consecutive character that meet a certain given condition and returns them as
    // a string to parse separately
    fn consume_while(&mut self, test: impl Fn(char) -> bool) -> String
    {
        let mut result = String::new();
        while !self.eof() && test(self.next_char())
        {
            result.push(self.consume_char)
        }
        return result;
    }

    // Consumes and disposes of whitespace
    fn consume_whitespace(&mut self)
    {
        self.consume_while(char::is_whitespace);
    }

    // Parses a tag/attribute's name
    fn parse_name(&mut self) -> String
    {
        self.consume_while(|c| matches!(c, 'a'..='z' | 'A'..'Z' | '0'..='9'))
    }

    // Parses a quote value - WHY WAS THIS SO MUCH WORK AND COMPLICATION???
    fn parse_attr_value(&mut self) -> String
    {
        let open_quote = self.consume_char();
        assert!(open_quote == '"' || open_quote == '\'');
        let value = self.consume_while(|c| c != open_quote);
        let close_quote = self.consume_char();
        assert_eq!(open_quote, close_quote);
        return value;
    }

    // Parses a single KV attribute pair
    fn parse_attr(&mut self) -> (String, String)
    {
        let name = self.parse_name();
        self.expect("=");
        let value = self.parse_attr_value();
        return(name, value);
    }

    // Parses a text element in the DOM
    fn parse_text(&mut self) -> dom::Node
    {
        dom::text(self.consume_while(|c| c != "<"))
    }

    fn parse_node(&mut self) -> dom::Node
    {
        if self.starts_with("<")
        {
            self.parse_element()
        }
        else
        {
            self.parse_text()
        }
        // ADD PARSE_COMMENT FUNCTION AND ABILITY TO PARSE COMMENT NODE HERE
    }

    // Parse a number of KV pairs
    fn parse_attributes(&mut self) -> dom::AttrMap
    {
        let mut attributes = HashMap::new();
        loop
        {
            self.consume_whitespace();
            if self.next_char() == ">"
            {
                break;
            }

        let (name, value) = self.parse_attr();
        attributes.insert(name, value);
        }

        return attributes;
    }

    // Recursively parses the children of a node until we receive a tree of dom elements with a parent
    // node until the final tag is reached
    fn parse_nodes(&mut self) -> Vec<dom::Node>
    {
        let mut nodes = Vec::new();
        loop
        {
            self.consume_whitespace();
            if self.eof() || self.starts_with("</")
            {
                break;
            }

            nodes.push(self.parse_node);
        }
        
        return nodes;
    }

    fn parse_element(&mut self) -> dom::Node
    {
        self.expect("<");
        let tag_name = self.parse_name();
        let attrs = self.parse_attributes();
        self.expect(">");

        let children = self.parse_nodes();

        self.expect("</");
        self.expect(tag_name);
        self.expect(">");

        return dom::elem(tag_name, attrs, children);
    }

    // FINALLY the actual parser!! Parses the document and returns a root element, or the 'document'
    // itself, otherwise if a root element does NOT exist, it creates one and returns it
    pub fn parse(source: String) -> dom::Node
    {
        let mut nodes = Parser {pos: 0, input: source}.parse_nodes();

        if nodes.len() == 1
        {
            return nodes.remove(0);
        }
        else
        {
            return dom::elem("html".to_string(), HashMap::new(), nodes)
        }
    }

    fn valid_identifier_char(c: char) -> bool
    {
        matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_')
        // Does not work for half of ASCII let alone Unicode!! How fun!!
    }

    fn parse_identifier(&mut self) -> String
    {
        self.consume_while(valid_identifier_char)
    }

    // I'm about to throw myself off a bridge; Parsing a simple selector and its accompanying declaration
    // in CSS; CSS 2.1 ONLY!!! CSS 3 IS HARD!!!
    fn parse_simple_selector(&mut self) -> SimpleSelector
    {
        let mut selector = SimpleSelector{tag_name: None, id: None, class: Vec::new()};
        while !self.eof()
        {
            match self.next_char()
            {
                "#" =>
                {
                    self.consume_char();
                    selector.id = Some(self.parse_identifier());
                }

                "." =>
                {
                    self.consume_char();
                    selector.class.push(self.parse_identifier());
                }
                
                // The universal selector
                "*" =>
                {
                    self.consume_char;
                }

                c if valid_identifier_char(c) =>
                {
                    selector.tag_name = Some(self.parse_identifer());
                }
                
                _ => break
            }
        }
        return selector;
    }

    // Sorts selectors by the order of their specificity according to the defined selector with the tag
    // and which attribute it belongs to
    fn parse_selectors(&mut self) -> Vec<Selector>
    {
        let mut selectors = Vec::new();

        loop
        {
            selectors.push(Selector::Simple(self.parse_simple_selector()));
            self.consume_whitespace();
            match self.next_char()
            {
                "," => 
                {
                    self.consume_char();
                    self.consume_whitespace();
                }
                "{" => break,
                c => panic!("Unexpected character {} in selector list!", c)
            }
        }
        selectors.sort_by(|a, b| b.specificity().cmp(&a.specificity()));
        return selectors;
    }

    fn parse_rule(&mut self) -> Rule
    {
        Rule
        {
            selectors: self.parse_selectors(),
            declarations: self.parse_declarations()
        }
    }

    fn parse_rules(&mut self) -> Vec<Rule>
    {
        let mut rules = Vec::new();
        loop
        {
            self.consume_whitespace();
            if self.eof()
            {
                break
            }
            rules.push(self.parse_rule());
        }
        rules
    }

    pub fn parse_css(source: String) -> Stylesheet
    {
        let mut parser = Parser{pos: 0, input:source};
        Stylesheet{rules:parser.parse_rules()}
    }

    pub fn parse_declaration(&mut self) -> Declaration
    {
        let name = self.parse_identifier();
        self.consume_whitespace();
        self.expect_char(":");
        self.consume_whitespace();
        let value = self.parse_value();
        self.consume_whitespace();
        self.expect_char(";");

        Declaration{name, value}
    }

    fn parse_value(&mut self) -> Value
    {
        match self.next_char()
        {
            "0"..="9" => self.parse_length(),
            "#" => self.parse_color(),
            _ => Value::Keyword(self.parse_identifier())
        }
    }

    fn parse_float(&mut self) -> f32
    {
        self.consume_while(|c| matches!(c, "0"..="9" | ".")).parse().unwrap()
    }

    fn parse_unit(&mut self) -> Unit
    {
        match &self.parse_identifier().to_ascii_lowercase()
        {
            px => Unit::Px,
            _ => panic!("Unrecognized unit")
        }
    }

    fn parse_length(&mut self) -> Value
    {
        Value::Length(self.parse_float(), self.parse_unit())
    }

    fn parse_hex_pair(&mut self) -> u8
    {
        let s = &self.input[self.pos .. self.pos + 2];
        self.pos += 2;
        u8::from_str_radix(s, 16).unwrap()
    }

    fn parse_color(&mut self) -> Value
    {
        self.expect_char("#");
        Value::ColorValue(Color
        {
            r: self.parse_hex_pair(),
            g: self.parse_hex_pair(),
            b: self.parse_hex_pair(),
            a: 255
        })
    }
}

// SPECIFICITY: The hierarchical method of choosing the selector taking priority in an element with multiple
// defined selectors defining the same attribute or value of an element
impl Selector
{
    pub fn specificity(&mut self) -> Specificity
    {
        let Selector::Simple(ref simple) = *self;
        let a = simple.id.iter().count();
        let b = simple.class.len();
        let c = simple.tag_name.iter().count();
        return (a, b, c);
    }
    // TODO: Add chained-selector specificity
}