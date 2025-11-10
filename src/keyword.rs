use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Keyword {
    Select,
    From,
    Where,
    Insert,
    Into,
    Values,
    Update,
    Set,
    Delete,
    Create,
    Table,
    Drop,
    Alter,
    Add,
    Join,
    On,
    As,
    And,
    Or,
    Not,
    Null,
    Is,
    In,
    Like,
    Order,
    By,
    Group,
    Having,
    Limit,
    Offset,
    Distinct,
    Union,
    All,
    Exists,
    Between,
    Case,
    When,
    Then,
    Else,
    End,
}

impl Keyword {
    pub fn as_str(&self) -> &'static str {
        match self {
            Keyword::Select => "SELECT",
            Keyword::From => "FROM",
            Keyword::Where => "WHERE",
            Keyword::Insert => "INSERT",
            Keyword::Into => "INTO",
            Keyword::Values => "VALUES",
            Keyword::Update => "UPDATE",
            Keyword::Set => "SET",
            Keyword::Delete => "DELETE",
            Keyword::Create => "CREATE",
            Keyword::Table => "TABLE",
            Keyword::Drop => "DROP",
            Keyword::Alter => "ALTER",
            Keyword::Add => "ADD",
            Keyword::Join => "JOIN",
            Keyword::On => "ON",
            Keyword::As => "AS",
            Keyword::And => "AND",
            Keyword::Or => "OR",
            Keyword::Not => "NOT",
            Keyword::Null => "NULL",
            Keyword::Is => "IS",
            Keyword::In => "IN",
            Keyword::Like => "LIKE",
            Keyword::Order => "ORDER",
            Keyword::By => "BY",
            Keyword::Group => "GROUP",
            Keyword::Having => "HAVING",
            Keyword::Limit => "LIMIT",
            Keyword::Offset => "OFFSET",
            Keyword::Distinct => "DISTINCT",
            Keyword::Union => "UNION",
            Keyword::All => "ALL",
            Keyword::Exists => "EXISTS",
            Keyword::Between => "BETWEEN",
            Keyword::Case => "CASE",
            Keyword::When => "WHEN",
            Keyword::Then => "THEN",
            Keyword::Else => "ELSE",
            Keyword::End => "END",
        }
    }

    pub const fn all_keywords() -> [Keyword; 40] {
        [
            Keyword::Select,
            Keyword::From,
            Keyword::Where,
            Keyword::Insert,
            Keyword::Into,
            Keyword::Values,
            Keyword::Update,
            Keyword::Set,
            Keyword::Delete,
            Keyword::Create,
            Keyword::Table,
            Keyword::Drop,
            Keyword::Alter,
            Keyword::Add,
            Keyword::Join,
            Keyword::On,
            Keyword::As,
            Keyword::And,
            Keyword::Or,
            Keyword::Not,
            Keyword::Null,
            Keyword::Is,
            Keyword::In,
            Keyword::Like,
            Keyword::Order,
            Keyword::By,
            Keyword::Group,
            Keyword::Having,
            Keyword::Limit,
            Keyword::Offset,
            Keyword::Distinct,
            Keyword::Union,
            Keyword::All,
            Keyword::Exists,
            Keyword::Between,
            Keyword::Case,
            Keyword::When,
            Keyword::Then,
            Keyword::Else,
            Keyword::End,
        ]
    }
}

impl std::str::FromStr for Keyword {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_uppercase().as_str() {
            "SELECT" => Ok(Keyword::Select),
            "FROM" => Ok(Keyword::From),
            "WHERE" => Ok(Keyword::Where),
            "INSERT" => Ok(Keyword::Insert),
            "INTO" => Ok(Keyword::Into),
            "VALUES" => Ok(Keyword::Values),
            "UPDATE" => Ok(Keyword::Update),
            "SET" => Ok(Keyword::Set),
            "DELETE" => Ok(Keyword::Delete),
            "CREATE" => Ok(Keyword::Create),
            "TABLE" => Ok(Keyword::Table),
            "DROP" => Ok(Keyword::Drop),
            "ALTER" => Ok(Keyword::Alter),
            "ADD" => Ok(Keyword::Add),
            "JOIN" => Ok(Keyword::Join),
            "ON" => Ok(Keyword::On),
            "AS" => Ok(Keyword::As),
            "AND" => Ok(Keyword::And),
            "OR" => Ok(Keyword::Or),
            "NOT" => Ok(Keyword::Not),
            "NULL" => Ok(Keyword::Null),
            "IS" => Ok(Keyword::Is),
            "IN" => Ok(Keyword::In),
            "LIKE" => Ok(Keyword::Like),
            "ORDER" => Ok(Keyword::Order),
            "BY" => Ok(Keyword::By),
            "GROUP" => Ok(Keyword::Group),
            "HAVING" => Ok(Keyword::Having),
            "LIMIT" => Ok(Keyword::Limit),
            "OFFSET" => Ok(Keyword::Offset),
            "DISTINCT" => Ok(Keyword::Distinct),
            "UNION" => Ok(Keyword::Union),
            "ALL" => Ok(Keyword::All),
            "EXISTS" => Ok(Keyword::Exists),
            "BETWEEN" => Ok(Keyword::Between),
            "CASE" => Ok(Keyword::Case),
            "WHEN" => Ok(Keyword::When),
            "THEN" => Ok(Keyword::Then),
            "ELSE" => Ok(Keyword::Else),
            "END" => Ok(Keyword::End),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct KeywordMap {
    inner: BTreeMap<usize, Vec<Keyword>>,
}

impl KeywordMap {
    pub fn new() -> Self {
        let inner =
            Keyword::all_keywords()
                .into_iter()
                .fold(BTreeMap::new(), |mut map, keyword| {
                    let len = keyword.as_str().len();
                    map.entry(len)
                        .and_modify(|list: &mut Vec<Keyword>| {
                            list.push(keyword);
                        })
                        .or_insert(vec![keyword]);
                    map
                });
        Self { inner }
    }

    pub fn get(&self, len: usize) -> Option<&Vec<Keyword>> {
        self.inner.get(&len)
    }
}
