use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum DataFieldType {
    Int,
    Float,
    String,
    Bool,
    Vec3,
}

impl DataFieldType {
    pub fn label(&self) -> &'static str {
        match self { Self::Int => "Int", Self::Float => "Float", Self::String => "String", Self::Bool => "Bool", Self::Vec3 => "Vec3" }
    }
    pub const ALL: [DataFieldType; 5] = [DataFieldType::Int, DataFieldType::Float, DataFieldType::String, DataFieldType::Bool, DataFieldType::Vec3];
}

#[derive(Clone, Debug)]
pub struct DataColumn {
    pub name: String,
    pub field_type: DataFieldType,
}

impl DataColumn {
    pub fn new(name: impl Into<String>, field_type: DataFieldType) -> Self {
        Self { name: name.into(), field_type }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DataValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Vec3([f64; 3]),
    Empty,
}

impl DataValue {
    pub fn label(&self) -> String {
        match self {
            Self::Int(v) => v.to_string(),
            Self::Float(v) => format!("{:.3}", v),
            Self::String(s) => s.clone(),
            Self::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
            Self::Vec3(v) => format!("({:.2}, {:.2}, {:.2})", v[0], v[1], v[2]),
            Self::Empty => String::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DataRow {
    pub id: u32,
    pub name: String,
    pub cells: Vec<DataValue>,
}

impl DataRow {
    pub fn new(id: u32, name: impl Into<String>, columns: usize) -> Self {
        Self { id, name: name.into(), cells: vec![DataValue::Empty; columns] }
    }

    pub fn get(&self, col: usize) -> &DataValue {
        self.cells.get(col).unwrap_or(&DataValue::Empty)
    }

    pub fn set(&mut self, col: usize, value: DataValue) {
        if col < self.cells.len() { self.cells[col] = value; }
    }
}

#[derive(Clone, Debug)]
pub struct DataTable {
    pub name: String,
    pub columns: Vec<DataColumn>,
    pub rows: Vec<DataRow>,
    next_id: u32,
}

impl Default for DataTable {
    fn default() -> Self {
        let mut t = Self { name: "Items".to_string(), columns: Vec::new(), rows: Vec::new(), next_id: 0 };
        t.columns.push(DataColumn::new("Nom", DataFieldType::String));
        t.columns.push(DataColumn::new("Dégâts", DataFieldType::Float));
        t.columns.push(DataColumn::new("Durabilité", DataFieldType::Int));
        t.columns.push(DataColumn::new("Rare", DataFieldType::Bool));
        t.columns.push(DataColumn::new("Prix", DataFieldType::Float));
        let mut epee = DataRow::new(t.next_id, "Épée", 5);
        t.next_id += 1;
        epee.set(0, DataValue::String("Épée en fer".to_string()));
        epee.set(1, DataValue::Float(25.0));
        epee.set(2, DataValue::Int(100));
        epee.set(3, DataValue::Bool(false));
        epee.set(4, DataValue::Float(50.0));
        t.rows.push(epee);
        let mut arc = DataRow::new(t.next_id, "Arc", 5);
        t.next_id += 1;
        arc.set(0, DataValue::String("Arc long".to_string()));
        arc.set(1, DataValue::Float(18.0));
        arc.set(2, DataValue::Int(80));
        arc.set(3, DataValue::Bool(false));
        arc.set(4, DataValue::Float(35.0));
        t.rows.push(arc);
        t
    }
}

impl DataTable {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), columns: Vec::new(), rows: Vec::new(), next_id: 0 }
    }

    pub fn add_column(&mut self, name: impl Into<String>, field_type: DataFieldType) {
        self.columns.push(DataColumn::new(name, field_type));
        for row in &mut self.rows { row.cells.push(DataValue::Empty); }
    }

    pub fn remove_column(&mut self, index: usize) {
        if index < self.columns.len() {
            self.columns.remove(index);
            for row in &mut self.rows { if index < row.cells.len() { row.cells.remove(index); } }
        }
    }

    pub fn add_row(&mut self, name: impl Into<String>) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.rows.push(DataRow::new(id, name, self.columns.len()));
        id
    }

    pub fn remove_row(&mut self, index: usize) {
        if index < self.rows.len() { self.rows.remove(index); }
    }

    pub fn find_row(&self, name: &str) -> Option<&DataRow> {
        self.rows.iter().find(|r| r.name == name)
    }

    pub fn to_map(&self) -> HashMap<String, Vec<DataValue>> {
        let mut map = HashMap::new();
        for row in &self.rows {
            map.insert(row.name.clone(), row.cells.clone());
        }
        map
    }
}

#[derive(Clone, Debug, Default)]
pub struct DataTableLibrary {
    pub tables: Vec<DataTable>,
}

impl DataTableLibrary {
    pub fn new() -> Self { Self::default() }

    pub fn add_table(&mut self, table: DataTable) {
        self.tables.push(table);
    }

    pub fn remove_table(&mut self, index: usize) {
        if index < self.tables.len() { self.tables.remove(index); }
    }

    pub fn find_table(&self, name: &str) -> Option<&DataTable> {
        self.tables.iter().find(|t| t.name == name)
    }
}
