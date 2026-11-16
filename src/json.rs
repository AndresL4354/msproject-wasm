//! Salida JSON (pensada para insertar en una base de datos, p. ej. Supabase).
//!
//! Serializador propio y minimo para no depender de serde.

use crate::model::*;
use crate::mpp::fields::{assignment as af, resource as rf, task as tf, TASK};
use crate::mpp::util::to_hours;
use crate::schema::{self, Cell, Index, Key};

pub struct Obj {
    buf: String,
    first: bool,
}

pub fn esc(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

pub fn num(v: f64) -> String {
    if !v.is_finite() {
        return "null".into();
    }
    let r = (v * 1e9).round() / 1e9;
    if r == r.trunc() && r.abs() < 1e15 {
        format!("{}", r as i64)
    } else {
        format!("{}", r)
    }
}

impl Obj {
    pub fn new() -> Obj {
        Obj { buf: String::from("{"), first: true }
    }

    fn key(&mut self, k: &str) {
        if !self.first {
            self.buf.push(',');
        }
        self.first = false;
        esc(&mut self.buf, k);
        self.buf.push(':');
    }

    pub fn raw(&mut self, k: &str, v: &str) -> &mut Obj {
        self.key(k);
        self.buf.push_str(v);
        self
    }

    pub fn str(&mut self, k: &str, v: &str) -> &mut Obj {
        self.key(k);
        esc(&mut self.buf, v);
        self
    }

    pub fn opt_str(&mut self, k: &str, v: Option<&str>) -> &mut Obj {
        match v {
            Some(s) => self.str(k, s),
            None => self.raw(k, "null"),
        }
    }

    pub fn int(&mut self, k: &str, v: i64) -> &mut Obj {
        self.raw(k, &v.to_string())
    }

    pub fn opt_int(&mut self, k: &str, v: Option<i64>) -> &mut Obj {
        match v {
            Some(n) => self.int(k, n),
            None => self.raw(k, "null"),
        }
    }

    pub fn num(&mut self, k: &str, v: f64) -> &mut Obj {
        self.raw(k, &num(v))
    }

    pub fn opt_num(&mut self, k: &str, v: Option<f64>) -> &mut Obj {
        match v {
            Some(n) => self.num(k, n),
            None => self.raw(k, "null"),
        }
    }

    pub fn bool(&mut self, k: &str, v: bool) -> &mut Obj {
        self.raw(k, if v { "true" } else { "false" })
    }

    pub fn date(&mut self, k: &str, v: Option<i64>) -> &mut Obj {
        match v {
            Some(d) => self.str(k, &iso(d)),
            None => self.raw(k, "null"),
        }
    }

    pub fn end(mut self) -> String {
        self.buf.push('}');
        self.buf
    }
}

fn array(items: Vec<String>) -> String {
    format!("[{}]", items.join(","))
}

fn cell_json(c: &Cell) -> String {
    match c {
        Cell::Empty => "null".into(),
        Cell::Text(s) => {
            let mut o = String::new();
            esc(&mut o, s);
            o
        }
        Cell::Num(v) | Cell::Pct(v) => num(*v),
        Cell::Int(v) => v.to_string(),
        Cell::Date(d) => format!("\"{}\"", iso(*d)),
        Cell::Dur(d) => format!("{{\"value\":{},\"units\":\"{}\"}}", num(d.value), d.unit.suffix()),
        Cell::Bool(b) => if *b { "true" } else { "false" }.into(),
    }
}

fn value_json(v: &Value) -> String {
    match v {
        Value::Text(s) => {
            let mut o = String::new();
            esc(&mut o, s);
            o
        }
        Value::Int(i) => i.to_string(),
        Value::Num(n) => num(*n),
        Value::Date(d) => format!("\"{}\"", iso(*d)),
        Value::Dur(d) => format!("{{\"value\":{},\"units\":\"{}\"}}", num(d.value), d.unit.suffix()),
        Value::Bool(b) => if *b { "true" } else { "false" }.into(),
        Value::Unit(u) => format!("\"{}\"", u.suffix()),
    }
}

fn dur_fields(o: &mut Obj, ix: &Index, prefix: &str, d: Option<Dur>) {
    o.opt_num(prefix, d.map(|x| x.value));
    o.opt_str(&format!("{}_units", prefix), d.map(|x| x.unit.suffix()));
    o.opt_num(&format!("{}_hours", prefix), d.and_then(|x| to_hours(&ix.sc, x)));
}

/// Convierte el proyecto a JSON. Con `full`, cada tarea incluye ademas todos los campos
/// estandar leidos del archivo (bajo "fields", por nombre de campo de Project).
pub fn project_json(p: &Project, full: bool) -> String {
    let ix = Index::new(p);
    let task_cols = schema::task_columns(&ix);
    let res_cols = schema::resource_columns(&ix);

    let mut proj = Obj::new();
    proj.opt_str("title", p.props.title.as_deref())
        .opt_str("author", p.props.author.as_deref())
        .opt_str("last_author", p.props.last_author.as_deref())
        .opt_str("company", p.props.company.as_deref())
        .opt_str("manager", p.props.manager.as_deref())
        .date("start", p.props.start)
        .date("finish", p.props.finish)
        .date("status_date", p.props.status_date)
        .int("minutes_per_day", p.props.minutes_per_day as i64)
        .int("minutes_per_week", p.props.minutes_per_week as i64)
        .int("days_per_month", p.props.days_per_month as i64)
        .str("default_duration_units", p.props.default_units.suffix())
        .opt_str("default_calendar", p.props.default_calendar.as_deref())
        .opt_str("currency_symbol", p.props.currency_symbol.as_deref())
        .str("application", &p.props.app_name)
        .int("application_version", p.props.app_version as i64)
        .int("task_count", p.tasks.len() as i64)
        .int("resource_count", p.resources.iter().filter(|r| r.name.is_some()).count() as i64)
        .int("assignment_count", p.assignments.len() as i64);

    let tasks: Vec<String> = p
        .tasks
        .iter()
        .map(|t| {
            let mut o = Obj::new();
            o.int("id", t.id as i64).int("uid", t.uid as i64).opt_str("name", t.name.as_deref());
            o.int("outline_level", t.outline_level as i64).str("outline_number", &t.outline_number);
            o.str("wbs", &t.wbs);
            o.opt_int("parent_uid", t.parent_uid.map(|v| v as i64));
            o.opt_int("parent_id", t.parent_uid.and_then(|u| ix.task(u)).map(|x| x.id as i64));
            o.date("start", t.start).date("finish", t.finish);
            dur_fields(&mut o, &ix, "duration", t.duration);
            let pct = |id: u16| match t.fields.get(&id) {
                Some(Value::Num(v)) => Some(*v),
                Some(Value::Int(v)) => Some(*v as f64),
                _ => None,
            };
            o.opt_num("percent_complete", pct(tf::PERCENT_COMPLETE));
            o.opt_num("percent_work_complete", pct(tf::PERCENT_WORK_COMPLETE));
            o.bool("summary", t.summary).bool("milestone", t.milestone).bool("critical", t.critical).bool("active", t.active).bool("manual", t.manual);
            let work = match t.fields.get(&tf::WORK) {
                Some(Value::Dur(d)) => Some(*d),
                _ => None,
            };
            o.opt_num("work_hours", work.and_then(|d| to_hours(&ix.sc, d)));
            o.opt_num("total_slack_hours", t.total_slack.and_then(|d| to_hours(&ix.sc, d)));
            o.opt_str("constraint_type", match t.fields.get(&tf::CONSTRAINT_TYPE) {
                Some(Value::Int(v)) => Some(schema::constraint_name(*v)),
                _ => None,
            });
            o.date("constraint_date", match t.fields.get(&tf::CONSTRAINT_DATE) {
                Some(Value::Date(d)) => Some(*d),
                _ => None,
            });
            let text = |k: Key| match ix.task_cell(t, &k) {
                Cell::Text(s) => s,
                _ => String::new(),
            };
            o.str("predecessors", &text(Key::Preds)).str("successors", &text(Key::Succs));
            let links: Vec<String> = ix
                .predecessors(t.uid)
                .iter()
                .map(|r| {
                    let mut l = Obj::new();
                    l.opt_int("id", ix.task(r.pred_uid).map(|x| x.id as i64)).int("uid", r.pred_uid as i64).str("type", r.kind.code());
                    l.num("lag", r.lag.map_or(0.0, |d| d.value)).opt_str("lag_units", r.lag.map(|d| d.unit.suffix()));
                    l.end()
                })
                .collect();
            o.raw("predecessor_links", &array(links));
            o.str("resource_names", &ix.resource_names(t.uid));
            let res: Vec<String> = ix
                .assignments(t.uid)
                .iter()
                .map(|(a, r)| {
                    let mut l = Obj::new();
                    l.int("uid", r.uid as i64).int("id", r.id as i64).opt_str("name", r.name.as_deref()).opt_num("units", a.units);
                    l.opt_num("work_hours", a.work.and_then(|d| to_hours(&ix.sc, d)));
                    l.end()
                })
                .collect();
            o.raw("resources", &array(res));
            o.opt_str("notes", t.notes.as_deref());

            // Campos personalizados (por alias si existe; si no, por nombre: "Text5").
            let mut custom = Obj::new();
            for c in task_cols.iter().filter(|c| c.custom) {
                let cell = ix.task_cell(t, &c.key);
                if !matches!(cell, Cell::Empty | Cell::Bool(false)) {
                    custom.raw(&c.title, &cell_json(&cell));
                }
            }
            o.raw("custom", &custom.end());

            if full {
                let mut f = Obj::new();
                let mut ids: Vec<&u16> = t.fields.keys().collect();
                ids.sort();
                for id in ids {
                    if let Some(name) = schema::field_name(&TASK, *id) {
                        f.raw(name, &value_json(&t.fields[id]));
                    }
                }
                o.raw("fields", &f.end());
            }
            o.end()
        })
        .collect();

    let resources: Vec<String> = p
        .resources
        .iter()
        .filter(|r| r.name.is_some())
        .map(|r| {
            let mut o = Obj::new();
            o.int("id", r.id as i64).int("uid", r.uid as i64).opt_str("name", r.name.as_deref());
            o.str("type", match r.kind {
                ResourceKind::Work => "work",
                ResourceKind::Material => "material",
                ResourceKind::Cost => "cost",
            });
            let text = |id: u16| match r.fields.get(&id) {
                Some(Value::Text(s)) if !s.is_empty() => Some(s.as_str()),
                _ => None,
            };
            let n = |id: u16| match r.fields.get(&id) {
                Some(Value::Num(v)) => Some(*v),
                _ => None,
            };
            o.opt_str("group", text(rf::GROUP)).opt_str("initials", text(rf::INITIALS));
            o.opt_num("max_units", n(rf::MAX_UNITS).map(|v| v / 100.0)).opt_num("peak_units", n(rf::PEAK).map(|v| v / 100.0));
            let work = match r.fields.get(&rf::WORK) {
                Some(Value::Dur(d)) => to_hours(&ix.sc, *d),
                _ => None,
            };
            o.opt_num("work_hours", work).opt_str("base_calendar", r.base_calendar.as_deref());
            o.date("start", r.start).date("finish", r.finish).opt_str("notes", r.notes.as_deref());
            let mut custom = Obj::new();
            for c in res_cols.iter().filter(|c| c.custom) {
                let cell = ix.resource_cell(r, &c.key);
                if !matches!(cell, Cell::Empty | Cell::Bool(false)) {
                    custom.raw(&c.title, &cell_json(&cell));
                }
            }
            o.raw("custom", &custom.end());
            o.end()
        })
        .collect();

    let assignments: Vec<String> = p
        .assignments
        .iter()
        .map(|a| {
            let mut o = Obj::new();
            let task = ix.task(a.task_uid);
            let res = a.resource_uid.and_then(|u| ix.resource(u));
            o.int("uid", a.uid as i64).int("task_uid", a.task_uid as i64).opt_int("task_id", task.map(|t| t.id as i64));
            o.opt_str("task_name", task.and_then(|t| t.name.as_deref()));
            o.opt_int("resource_uid", a.resource_uid.map(|v| v as i64)).opt_int("resource_id", res.map(|r| r.id as i64));
            o.opt_str("resource_name", res.and_then(|r| r.name.as_deref()));
            o.opt_num("units", a.units).opt_num("work_hours", a.work.and_then(|d| to_hours(&ix.sc, d)));
            let rem = match a.fields.get(&af::REMAINING_WORK) {
                Some(Value::Dur(d)) => to_hours(&ix.sc, *d),
                _ => None,
            };
            o.opt_num("remaining_work_hours", rem).date("start", a.start).date("finish", a.finish);
            o.end()
        })
        .collect();

    let calendars: Vec<String> = p
        .calendars
        .iter()
        .map(|c| {
            let mut o = Obj::new();
            o.int("uid", c.uid as i64).opt_str("name", c.name.as_deref());
            o.opt_str("base_calendar", c.base_uid.and_then(|b| p.calendars.iter().find(|x| x.uid == b)).and_then(|x| x.name.as_deref()));
            o.opt_int("resource_uid", c.resource_uid.map(|v| v as i64));
            o.end()
        })
        .collect();

    let columns: Vec<String> = task_cols
        .iter()
        .map(|c| {
            let mut s = String::new();
            esc(&mut s, &c.title);
            s
        })
        .collect();

    let mut root = Obj::new();
    root.raw("project", &proj.end())
        .raw("task_columns", &array(columns))
        .raw("tasks", &array(tasks))
        .raw("resources", &array(resources))
        .raw("assignments", &array(assignments))
        .raw("calendars", &array(calendars));
    root.end()
}
