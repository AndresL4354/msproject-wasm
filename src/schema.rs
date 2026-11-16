//! Capa comun a las salidas (Excel y JSON): que columnas mostrar y que valor va en cada celda.
//!
//! Las columnas de tareas replican la tabla de la vista Gantt guardada en el propio .mpp
//! (mismo orden y mismos titulos que ve el planificador en Project).

use crate::model::*;
use crate::mpp::fields::{self, resource as rf, task as tf, FieldTable, RESOURCE, TASK};
use crate::mpp::util::Scale;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Cell {
    Empty,
    Text(String),
    Num(f64),
    Int(i64),
    Date(i64),
    Dur(Dur),
    Bool(bool),
    /// Porcentaje en escala 0..100.
    Pct(f64),
}

impl Cell {
    pub fn is_empty(&self) -> bool {
        matches!(self, Cell::Empty)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Key {
    Field(u16),
    Flag(usize),
    Id,
    Uid,
    Name,
    Start,
    Finish,
    Duration,
    Preds,
    Succs,
    UidPreds,
    UidSuccs,
    ResNames,
    ResInitials,
    ResGroup,
    Critical,
    Summary,
    Milestone,
    Active,
    Mode,
    OutlineLevel,
    OutlineNumber,
    TotalSlack,
    ConstraintType,
    TaskType,
    Notes,
    // Recursos
    ResType,
    BaseCalendar,
}

#[derive(Clone, Debug)]
pub struct Col {
    pub title: String,
    pub key: Key,
    /// Ancho en caracteres.
    pub width: f64,
    /// Campo personalizado (Text1, Number4...).
    pub custom: bool,
}

/// Indices para resolver relaciones entre tareas, recursos y asignaciones.
pub struct Index<'a> {
    pub p: &'a Project,
    pub sc: Scale,
    task_ix: HashMap<i32, usize>,
    res_ix: HashMap<i32, usize>,
    preds: HashMap<i32, Vec<usize>>,
    succs: HashMap<i32, Vec<usize>>,
    assigns: HashMap<i32, Vec<usize>>,
}

impl<'a> Index<'a> {
    pub fn new(p: &'a Project) -> Index<'a> {
        let mut ix = Index {
            p,
            sc: Scale {
                minutes_per_day: p.props.minutes_per_day as f64,
                minutes_per_week: p.props.minutes_per_week as f64,
                days_per_month: p.props.days_per_month as f64,
                default_units: p.props.default_units,
            },
            task_ix: p.tasks.iter().enumerate().map(|(i, t)| (t.uid, i)).collect(),
            res_ix: p.resources.iter().enumerate().map(|(i, r)| (r.uid, i)).collect(),
            preds: HashMap::new(),
            succs: HashMap::new(),
            assigns: HashMap::new(),
        };
        for (i, r) in p.relations.iter().enumerate() {
            ix.preds.entry(r.succ_uid).or_default().push(i);
            ix.succs.entry(r.pred_uid).or_default().push(i);
        }
        for (i, a) in p.assignments.iter().enumerate() {
            ix.assigns.entry(a.task_uid).or_default().push(i);
        }
        // Orden estable: por ID de la tarea relacionada / del recurso.
        let task_id = |uid: i32| ix.task_ix.get(&uid).map_or(i32::MAX, |&i| p.tasks[i].id);
        for v in ix.preds.values_mut() {
            v.sort_by_key(|&i| task_id(p.relations[i].pred_uid));
        }
        for v in ix.succs.values_mut() {
            v.sort_by_key(|&i| task_id(p.relations[i].succ_uid));
        }
        let res_id = |uid: Option<i32>| uid.and_then(|u| ix.res_ix.get(&u)).map_or(i32::MAX, |&i| p.resources[i].id);
        for v in ix.assigns.values_mut() {
            v.sort_by_key(|&i| res_id(p.assignments[i].resource_uid));
        }
        ix
    }

    pub fn task(&self, uid: i32) -> Option<&'a Task> {
        self.task_ix.get(&uid).map(|&i| &self.p.tasks[i])
    }

    pub fn resource(&self, uid: i32) -> Option<&'a Resource> {
        self.res_ix.get(&uid).map(|&i| &self.p.resources[i])
    }

    pub fn predecessors(&self, uid: i32) -> Vec<&'a Relation> {
        self.preds.get(&uid).map_or(Vec::new(), |v| v.iter().map(|&i| &self.p.relations[i]).collect())
    }

    pub fn successors(&self, uid: i32) -> Vec<&'a Relation> {
        self.succs.get(&uid).map_or(Vec::new(), |v| v.iter().map(|&i| &self.p.relations[i]).collect())
    }

    /// Asignaciones de la tarea con recurso real, ordenadas por ID de recurso.
    pub fn assignments(&self, task_uid: i32) -> Vec<(&'a Assignment, &'a Resource)> {
        self.assigns.get(&task_uid).map_or(Vec::new(), |v| {
            v.iter()
                .filter_map(|&i| {
                    let a = &self.p.assignments[i];
                    let r = self.resource(a.resource_uid?)?;
                    r.name.as_ref()?;
                    Some((a, r))
                })
                .collect()
        })
    }

    /// Lista de relaciones al estilo de Project: "7", "20SS", "146FS+2h".
    fn relation_list(&self, rels: &[&Relation], pred_side: bool, by_uid: bool) -> String {
        let mut parts = Vec::new();
        for r in rels {
            let other = if pred_side { r.pred_uid } else { r.succ_uid };
            let num = if by_uid { Some(other) } else { self.task(other).map(|t| t.id) };
            let num = match num {
                Some(n) => n,
                None => continue,
            };
            let lag = r.lag.filter(|l| l.value != 0.0);
            let mut s = num.to_string();
            if r.kind != RelType::FinishStart || lag.is_some() {
                s.push_str(r.kind.code());
            }
            if let Some(l) = lag {
                s.push_str(if l.value > 0.0 { "+" } else { "-" });
                s.push_str(&fmt_num(l.value.abs()));
                s.push_str(l.unit.suffix());
            }
            parts.push(s);
        }
        parts.join(";")
    }

    pub fn resource_names(&self, task_uid: i32) -> String {
        self.assignments(task_uid)
            .iter()
            .map(|(a, r)| {
                let name = r.name.clone().unwrap_or_default();
                match a.units {
                    Some(u) if (u - 1.0).abs() > 1e-9 && r.kind == ResourceKind::Work => format!("{}[{}]", name, fmt_num(u)),
                    _ => name,
                }
            })
            .collect::<Vec<_>>()
            .join(";")
    }

    fn resource_texts(&self, task_uid: i32, field: u16) -> String {
        let mut seen: Vec<String> = Vec::new();
        for (_, r) in self.assignments(task_uid) {
            if let Some(Value::Text(s)) = r.fields.get(&field) {
                if !s.is_empty() && !seen.contains(s) {
                    seen.push(s.clone());
                }
            }
        }
        seen.join(";")
    }

    pub fn task_cell(&self, t: &Task, key: &Key) -> Cell {
        let text = |s: String| if s.is_empty() { Cell::Empty } else { Cell::Text(s) };
        match key {
            Key::Id => Cell::Int(t.id as i64),
            Key::Uid => Cell::Int(t.uid as i64),
            Key::Name => text(t.name.clone().unwrap_or_default()),
            Key::Start => t.start.map_or(Cell::Empty, Cell::Date),
            Key::Finish => t.finish.map_or(Cell::Empty, Cell::Date),
            Key::Duration => t.duration.map_or(Cell::Empty, Cell::Dur),
            Key::Preds => text(self.relation_list(&self.predecessors(t.uid), true, false)),
            Key::Succs => text(self.relation_list(&self.successors(t.uid), false, false)),
            Key::UidPreds => text(self.relation_list(&self.predecessors(t.uid), true, true)),
            Key::UidSuccs => text(self.relation_list(&self.successors(t.uid), false, true)),
            Key::ResNames => text(self.resource_names(t.uid)),
            Key::ResInitials => text(self.resource_texts(t.uid, rf::INITIALS)),
            Key::ResGroup => text(self.resource_texts(t.uid, rf::GROUP)),
            Key::Critical => Cell::Bool(t.critical),
            Key::Summary => Cell::Bool(t.summary),
            Key::Milestone => Cell::Bool(t.milestone),
            Key::Active => Cell::Bool(t.active),
            Key::Mode => Cell::Text(if t.manual { "Manually Scheduled" } else { "Auto Scheduled" }.into()),
            Key::OutlineLevel => Cell::Int(t.outline_level as i64),
            Key::OutlineNumber => text(t.outline_number.clone()),
            Key::TotalSlack => t.total_slack.map_or(Cell::Empty, Cell::Dur),
            Key::Notes => text(t.notes.clone().unwrap_or_default()),
            Key::Flag(i) => Cell::Bool(t.flags.get(*i).copied().unwrap_or(false)),
            Key::ConstraintType => match t.fields.get(&tf::CONSTRAINT_TYPE) {
                Some(Value::Int(v)) => Cell::Text(constraint_name(*v).into()),
                _ => Cell::Empty,
            },
            Key::TaskType => match t.fields.get(&tf::TYPE) {
                Some(Value::Int(v)) => Cell::Text(match v {
                    0 => "Fixed Units",
                    1 => "Fixed Duration",
                    _ => "Fixed Work",
                }
                .into()),
                _ => Cell::Empty,
            },
            Key::Field(id) => field_cell(&TASK, &t.fields, *id),
            Key::ResType | Key::BaseCalendar => Cell::Empty,
        }
    }

    pub fn resource_cell(&self, r: &Resource, key: &Key) -> Cell {
        match key {
            Key::Id => Cell::Int(r.id as i64),
            Key::Uid => Cell::Int(r.uid as i64),
            Key::Name => r.name.clone().map_or(Cell::Empty, Cell::Text),
            Key::Start => r.start.map_or(Cell::Empty, Cell::Date),
            Key::Finish => r.finish.map_or(Cell::Empty, Cell::Date),
            Key::Notes => r.notes.clone().map_or(Cell::Empty, Cell::Text),
            Key::Flag(i) => Cell::Bool(r.flags.get(*i).copied().unwrap_or(false)),
            Key::BaseCalendar => r.base_calendar.clone().map_or(Cell::Empty, Cell::Text),
            Key::ResType => Cell::Text(
                match r.kind {
                    ResourceKind::Work => "Work",
                    ResourceKind::Material => "Material",
                    ResourceKind::Cost => "Cost",
                }
                .into(),
            ),
            Key::Field(id) => {
                // Las unidades de recurso se guardan como porcentaje (400 = 4 personas).
                if *id == rf::MAX_UNITS || *id == rf::PEAK {
                    match r.fields.get(id) {
                        Some(Value::Num(v)) => Cell::Num(*v / 100.0),
                        _ => Cell::Empty,
                    }
                } else {
                    field_cell(&RESOURCE, &r.fields, *id)
                }
            }
            _ => Cell::Empty,
        }
    }
}

fn field_cell(table: &FieldTable, f: &Fields, id: u16) -> Cell {
    let pct = table.dt.get(id as usize).copied() == Some(fields::DT_PERCENTAGE);
    match f.get(&id) {
        None => Cell::Empty,
        Some(Value::Text(s)) => {
            if s.is_empty() {
                Cell::Empty
            } else {
                Cell::Text(s.clone())
            }
        }
        Some(Value::Int(v)) => {
            if pct {
                Cell::Pct(*v as f64)
            } else {
                Cell::Int(*v as i64)
            }
        }
        Some(Value::Num(v)) => {
            if pct {
                Cell::Pct(*v)
            } else {
                Cell::Num(*v)
            }
        }
        Some(Value::Date(d)) => Cell::Date(*d),
        Some(Value::Dur(d)) => Cell::Dur(*d),
        Some(Value::Bool(b)) => Cell::Bool(*b),
        Some(Value::Unit(u)) => Cell::Text(u.suffix().into()),
    }
}

pub fn constraint_name(v: i32) -> &'static str {
    match v {
        0 => "As Soon As Possible",
        1 => "As Late As Possible",
        2 => "Must Start On",
        3 => "Must Finish On",
        4 => "Start No Earlier Than",
        5 => "Start No Later Than",
        6 => "Finish No Earlier Than",
        7 => "Finish No Later Than",
        _ => "",
    }
}

/// Numero sin ceros sobrantes: 2, 0.5, 1046.42.
pub fn fmt_num(v: f64) -> String {
    let r = (v * 1e6).round() / 1e6;
    if r == r.trunc() && r.abs() < 1e15 {
        format!("{}", r as i64)
    } else {
        format!("{}", r)
    }
}

/// Nombre estandar (en ingles, como en Project) de un campo.
pub fn field_name(table: &FieldTable, id: u16) -> Option<&'static str> {
    table.names.binary_search_by_key(&id, |(i, _)| *i).ok().map(|i| table.names[i].1)
}

/// Familia y numero de un campo personalizado: "Text24" -> (0, 24).
fn custom_rank(name: &str) -> Option<(usize, u32)> {
    const FAMILIES: [&str; 9] = ["Text", "Number", "Date", "Start", "Finish", "Duration", "Cost", "Flag", "Outline Code"];
    for (i, fam) in FAMILIES.iter().enumerate() {
        if let Some(rest) = name.strip_prefix(fam) {
            if !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit()) {
                return rest.parse().ok().map(|n| (i, n));
            }
        }
    }
    None
}

fn key_for_task_field(id: u16) -> Option<Key> {
    Some(match id {
        tf::ID => Key::Id,
        tf::UNIQUE_ID => Key::Uid,
        tf::NAME => Key::Name,
        tf::START | tf::START_TEXT => Key::Start,
        tf::FINISH | tf::FINISH_TEXT => Key::Finish,
        tf::DURATION_TEXT | tf::SCHEDULED_DURATION => Key::Duration,
        tf::PREDECESSORS => Key::Preds,
        tf::SUCCESSORS => Key::Succs,
        tf::UNIQUE_ID_PREDECESSORS => Key::UidPreds,
        tf::UNIQUE_ID_SUCCESSORS => Key::UidSuccs,
        tf::RESOURCE_NAMES => Key::ResNames,
        tf::RESOURCE_INITIALS => Key::ResInitials,
        tf::RESOURCE_GROUP => Key::ResGroup,
        tf::CRITICAL => Key::Critical,
        tf::SUMMARY => Key::Summary,
        tf::MILESTONE => Key::Milestone,
        tf::ACTIVE => Key::Active,
        tf::TASK_MODE => Key::Mode,
        tf::OUTLINE_LEVEL => Key::OutlineLevel,
        tf::OUTLINE_NUMBER => Key::OutlineNumber,
        tf::TOTAL_SLACK => Key::TotalSlack,
        tf::CONSTRAINT_TYPE => Key::ConstraintType,
        tf::TYPE => Key::TaskType,
        tf::NOTES => Key::Notes,
        // Columnas graficas o calculadas al vuelo por Project: no hay dato que transcribir.
        tf::INDICATORS | tf::STATUS_INDICATOR | tf::STATUS => return None,
        _ => match field_name(&TASK, id).and_then(custom_rank) {
            Some((7, n)) if (1..=20).contains(&n) => Key::Flag(n as usize - 1),
            _ => Key::Field(id),
        },
    })
}

fn key_for_resource_field(id: u16) -> Option<Key> {
    Some(match id {
        rf::ID => Key::Id,
        rf::UNIQUE_ID => Key::Uid,
        rf::NAME => Key::Name,
        rf::START => Key::Start,
        rf::FINISH => Key::Finish,
        rf::NOTES => Key::Notes,
        rf::BASE_CALENDAR => Key::BaseCalendar,
        rf::TYPE => Key::ResType,
        rf::INDICATORS => return None,
        _ => match field_name(&RESOURCE, id).and_then(custom_rank) {
            Some((7, n)) if (1..=20).contains(&n) => Key::Flag(n as usize - 1),
            _ => Key::Field(id),
        },
    })
}

fn find_table<'a>(p: &'a Project, resource: bool) -> Option<&'a Table> {
    let candidates: Vec<&Table> = p.tables.iter().filter(|t| t.resource == resource && !t.columns.is_empty()).collect();
    candidates
        .iter()
        .find(|t| t.name.eq_ignore_ascii_case("Entry") || t.name.eq_ignore_ascii_case("Entrada"))
        .or_else(|| candidates.first())
        .copied()
}

fn push(cols: &mut Vec<Col>, title: &str, key: Key, width: f64, custom: bool) {
    if !cols.iter().any(|c| c.key == key) {
        cols.push(Col { title: title.to_string(), key, width, custom });
    }
}

fn clamp_width(w: u8) -> f64 {
    (w as f64 + 2.0).clamp(7.0, 60.0)
}

/// Columnas de la hoja de tareas: primero las de la tabla del Gantt del archivo, luego datos
/// de estructura y por ultimo el resto de campos personalizados que tengan algun valor.
pub fn task_columns(ix: &Index) -> Vec<Col> {
    let p = ix.p;
    let mut cols: Vec<Col> = Vec::new();
    match find_table(p, false) {
        Some(t) => {
            for c in &t.columns {
                let idx = (c.field & 0xFFFF) as usize;
                if c.field & 0xFFFF_0000 != TASK.base || idx >= TASK.canon.len() {
                    continue;
                }
                let canon = TASK.canon[idx];
                let key = match key_for_task_field(canon) {
                    Some(k) => k,
                    None => continue,
                };
                let name = field_name(&TASK, canon);
                let custom = name.and_then(custom_rank).is_some();
                // Las columnas Start/Finish/Duration de Project usan internamente los campos "... Text".
                let plain = match key {
                    Key::Start => Some("Start"),
                    Key::Finish => Some("Finish"),
                    Key::Duration => Some("Duration"),
                    _ => name,
                };
                let title = c.title.clone().or_else(|| p.task_aliases.get(&canon).cloned()).or_else(|| plain.map(String::from)).unwrap_or_else(|| format!("Field {}", canon));
                push(&mut cols, &title, key, clamp_width(c.width), custom);
            }
        }
        None => {
            push(&mut cols, "ID", Key::Id, 7.0, false);
            push(&mut cols, "Task Name", Key::Name, 55.0, false);
            push(&mut cols, "Duration", Key::Duration, 11.0, false);
            push(&mut cols, "Start", Key::Start, 17.0, false);
            push(&mut cols, "Finish", Key::Finish, 17.0, false);
            push(&mut cols, "% Complete", Key::Field(tf::PERCENT_COMPLETE), 11.0, false);
            push(&mut cols, "Predecessors", Key::Preds, 18.0, false);
            push(&mut cols, "Resource Names", Key::ResNames, 40.0, false);
        }
    }
    if !cols.iter().any(|c| c.key == Key::Id) {
        cols.insert(0, Col { title: "ID".into(), key: Key::Id, width: 7.0, custom: false });
    }
    if !cols.iter().any(|c| c.key == Key::Name) {
        cols.insert(1, Col { title: "Task Name".into(), key: Key::Name, width: 55.0, custom: false });
    }

    push(&mut cols, "Outline Level", Key::OutlineLevel, 9.0, false);
    push(&mut cols, "Outline Number", Key::OutlineNumber, 12.0, false);
    push(&mut cols, "Summary", Key::Summary, 10.0, false);
    push(&mut cols, "Milestone", Key::Milestone, 10.0, false);
    push(&mut cols, "Critical", Key::Critical, 10.0, false);
    push(&mut cols, "Active", Key::Active, 9.0, false);
    push(&mut cols, "Total Slack", Key::TotalSlack, 11.0, false);
    push(&mut cols, "Work", Key::Field(tf::WORK), 11.0, false);
    push(&mut cols, "Constraint Type", Key::ConstraintType, 22.0, false);
    push(&mut cols, "Constraint Date", Key::Field(tf::CONSTRAINT_DATE), 17.0, false);
    if p.tasks.iter().any(|t| t.notes.is_some()) {
        push(&mut cols, "Notes", Key::Notes, 40.0, false);
    }

    // Resto de campos personalizados con datos.
    let mut extra: Vec<((usize, u32), u16, &'static str)> = TASK.names.iter().filter_map(|&(id, name)| custom_rank(name).map(|r| (r, id, name))).collect();
    extra.sort();
    for (rank, id, name) in extra {
        let key = if rank.0 == 7 { Key::Flag(rank.1 as usize - 1) } else { Key::Field(id) };
        if cols.iter().any(|c| c.key == key) {
            continue;
        }
        let used = p.tasks.iter().any(|t| match ix.task_cell(t, &key) {
            Cell::Empty | Cell::Bool(false) => false,
            _ => true,
        });
        if used {
            let title = p.task_aliases.get(&id).cloned().unwrap_or_else(|| name.to_string());
            let width = if rank.0 == 0 { 22.0 } else { 14.0 };
            push(&mut cols, &title, key, width, true);
        }
    }
    cols
}

pub fn resource_columns(ix: &Index) -> Vec<Col> {
    let p = ix.p;
    let mut cols: Vec<Col> = Vec::new();
    if let Some(t) = find_table(p, true) {
        for c in &t.columns {
            let idx = (c.field & 0xFFFF) as usize;
            if c.field & 0xFFFF_0000 != RESOURCE.base || idx >= RESOURCE.canon.len() {
                continue;
            }
            let canon = RESOURCE.canon[idx];
            let key = match key_for_resource_field(canon) {
                Some(k) => k,
                None => continue,
            };
            let name = field_name(&RESOURCE, canon);
            let title = c.title.clone().or_else(|| p.resource_aliases.get(&canon).cloned()).or_else(|| name.map(String::from)).unwrap_or_else(|| format!("Field {}", canon));
            push(&mut cols, &title, key, clamp_width(c.width), name.and_then(custom_rank).is_some());
        }
    }
    if !cols.iter().any(|c| c.key == Key::Id) {
        cols.insert(0, Col { title: "ID".into(), key: Key::Id, width: 7.0, custom: false });
    }
    if !cols.iter().any(|c| c.key == Key::Name) {
        cols.insert(1, Col { title: "Resource Name".into(), key: Key::Name, width: 34.0, custom: false });
    }
    push(&mut cols, "Type", Key::ResType, 10.0, false);
    push(&mut cols, "Group", Key::Field(rf::GROUP), 28.0, false);
    push(&mut cols, "Max. Units", Key::Field(rf::MAX_UNITS), 11.0, false);
    push(&mut cols, "Base Calendar", Key::BaseCalendar, 20.0, false);
    push(&mut cols, "Work", Key::Field(rf::WORK), 12.0, false);

    let mut extra: Vec<((usize, u32), u16, &'static str)> = RESOURCE.names.iter().filter_map(|&(id, name)| custom_rank(name).map(|r| (r, id, name))).collect();
    extra.sort();
    for (rank, id, name) in extra {
        let key = if rank.0 == 7 { Key::Flag(rank.1 as usize - 1) } else { Key::Field(id) };
        if cols.iter().any(|c| c.key == key) {
            continue;
        }
        let used = p.resources.iter().filter(|r| r.name.is_some()).any(|r| match ix.resource_cell(r, &key) {
            Cell::Empty | Cell::Bool(false) => false,
            _ => true,
        });
        if used {
            let title = p.resource_aliases.get(&id).cloned().unwrap_or_else(|| name.to_string());
            push(&mut cols, &title, key, 18.0, true);
        }
    }
    cols
}
