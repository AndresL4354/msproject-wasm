//! Lector de archivos Microsoft Project en formato MPP14 (Project 2010, 2013, 2016, 2019, 2021 y 365).
//!
//! La estructura del formato (que no esta documentada por Microsoft) se tomo del proyecto de
//! codigo abierto MPXJ, que la describe en su codigo fuente; esta es una implementacion propia.

pub mod blocks;
pub mod fields;
pub mod util;

use crate::cfb::Cfb;
use crate::model::*;
use blocks::*;
use fields::{assignment as af, resource as rf, task as tf, ASSIGNMENT, RESOURCE, TASK};
use std::collections::{BTreeMap, HashMap, HashSet};
use util::*;

const PROJECT_DIR: &str = "   114";
const VIEW_DIR: &str = "   214";

// Claves del bloque Props.
const K_TASK_FIELD_MAP: [u32; 2] = [131092, 50331668];
const K_RESOURCE_FIELD_MAP: [u32; 2] = [131093, 50331669];
const K_ASSIGNMENT_FIELD_MAP: [u32; 2] = [131095, 50331671];
const K_CUSTOM_FIELDS: u32 = 71303169;
const K_PASSWORD_FLAG: u32 = 893386752;
const K_PASSWORD_HASH: u32 = 893386756;
const K_ENCRYPTION_CODE: u32 = 893386759;
const K_COMPANY: u32 = 37748736;
const K_MANAGER: u32 = 37748737;
const K_START_DATE: u32 = 37748738;
const K_FINISH_DATE: u32 = 37748739;
const K_TITLE: u32 = 37748744;
const K_AUTHOR: u32 = 37748748;
const K_DEFAULT_CALENDAR_NAME: u32 = 37748750;
const K_CRITICAL_SLACK_LIMIT: u32 = 37748756;
const K_DURATION_UNITS: u32 = 37748757;
const K_MINUTES_PER_DAY: u32 = 37748765;
const K_MINUTES_PER_WEEK: u32 = 37748766;
const K_STATUS_DATE: u32 = 37748805;
const K_DAYS_PER_MONTH: u32 = 37753743;
const K_CURRENCY_SYMBOL: u32 = 37748752;

const NULL_TASK_BLOCK_SIZE: usize = 16;
const ASSIGNMENT_NULL_RESOURCE: i32 = -65535;

struct Ctx<'a> {
    cfb: &'a Cfb<'a>,
    encrypted: bool,
    mask: u8,
    props: Props,
    sc: Scale,
    app_version: i32,
    lookup: Lookup,
}

impl<'a> Ctx<'a> {
    fn stream_opt(&self, dir: &[&str], name: &str) -> Option<Vec<u8>> {
        let mut path: Vec<&str> = dir.to_vec();
        path.push(name);
        self.cfb.read_path(&path)
    }

    fn stream(&self, dir: &[&str], name: &str) -> Result<Vec<u8>, String> {
        self.stream_opt(dir, name).ok_or_else(|| format!("Falta el bloque {}/{} en el archivo.", dir.last().unwrap_or(&""), name))
    }

    /// Algunos bloques se guardan con un XOR simple cuando el archivo tiene contrasena de escritura.
    fn stream_enc(&self, dir: &[&str], name: &str) -> Result<Vec<u8>, String> {
        let mut d = self.stream(dir, name)?;
        if self.encrypted && self.mask != 0 {
            for b in d.iter_mut() {
                *b ^= self.mask;
            }
        }
        Ok(d)
    }

    fn var(&self, dir: &[&str]) -> Result<VarData, String> {
        VarData::parse(&self.stream(dir, "VarMeta")?, self.stream(dir, "Var2Data")?)
    }
}

fn bit(data: Option<&[u8]>, offset: usize, mask: u32) -> bool {
    data.map_or(false, |d| u32_at(d, offset) & mask != 0)
}

fn date_of(f: &Fields, id: u16) -> Option<i64> {
    match f.get(&id) {
        Some(Value::Date(d)) => Some(*d),
        _ => None,
    }
}

fn dur_of(f: &Fields, id: u16) -> Option<Dur> {
    match f.get(&id) {
        Some(Value::Dur(d)) => Some(*d),
        _ => None,
    }
}

fn int_of(f: &Fields, id: u16) -> Option<i32> {
    match f.get(&id) {
        Some(Value::Int(v)) => Some(*v),
        _ => None,
    }
}

fn num_of(f: &Fields, id: u16) -> Option<f64> {
    match f.get(&id) {
        Some(Value::Num(v)) => Some(*v),
        Some(Value::Int(v)) => Some(*v as f64),
        _ => None,
    }
}

fn text_of(f: &Fields, id: u16) -> Option<String> {
    match f.get(&id) {
        Some(Value::Text(s)) => Some(s.clone()),
        _ => None,
    }
}

/// Lee un archivo .mpp completo.
pub fn read(bytes: &[u8]) -> Result<Project, String> {
    let cfb = Cfb::open(bytes)?;

    if cfb.path(&[PROJECT_DIR]).is_none() {
        let old = if cfb.path(&["   112"]).is_some() {
            " Este archivo es de Project 2007."
        } else if cfb.path(&["   19"]).is_some() {
            " Este archivo es de Project 2000-2003."
        } else {
            ""
        };
        return Err(format!(
            "Formato de Project no soportado: se requiere un .mpp de Project 2010 o posterior.{} Abralo en Project y guardelo de nuevo.",
            old
        ));
    }

    let root_props = cfb.read_path(&["Props14"]).map(|d| Props::parse(&d)).unwrap_or_else(Props::empty);
    let password_flag = root_props.u8(K_PASSWORD_FLAG);
    if password_flag & 1 != 0 && root_props.bytes(K_PASSWORD_HASH).is_some() {
        return Err("El archivo esta protegido con contrasena de apertura; quitela en Project y vuelva a intentar.".into());
    }
    let code = root_props.u8(K_ENCRYPTION_CODE);
    let mut ctx = Ctx {
        cfb: &cfb,
        encrypted: password_flag != 0,
        mask: if code == 0 { 0 } else { 0xFF - code },
        props: Props::empty(),
        sc: Scale { minutes_per_day: 480.0, minutes_per_week: 2400.0, days_per_month: 20.0, default_units: Unit::Days },
        app_version: 16,
        lookup: Lookup::default(),
    };
    ctx.props = Props::parse(&ctx.stream_enc(&[PROJECT_DIR], "Props")?);

    // Version de la aplicacion que guardo el archivo (cambia la posicion de algunos indicadores).
    let mut app_name = String::new();
    if let Some(co) = cfb.read_path(&["\u{1}CompObj"]) {
        let len = i32_at(&co, 28).max(0) as usize;
        if len > 0 {
            app_name = latin1(co.get(32..32 + len).unwrap_or(&[]), 0);
        }
    }
    if let Some(pos) = app_name.find("Project") {
        let digits: String = app_name[pos + 7..].chars().skip(1).take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(v) = digits.parse::<i32>() {
            ctx.app_version = v;
        }
    }

    let p = &ctx.props;
    let default_units = duration_units(p.u16(K_DURATION_UNITS), None);
    ctx.sc = Scale {
        minutes_per_day: p.i32(K_MINUTES_PER_DAY) as f64,
        minutes_per_week: p.i32(K_MINUTES_PER_WEEK) as f64,
        days_per_month: p.u16(K_DAYS_PER_MONTH) as f64,
        default_units,
    };

    let (summary, doc_summary) = (
        cfb.read_path(&["\u{5}SummaryInformation"]).map(|d| property_set(&d)).unwrap_or_default(),
        cfb.read_path(&["\u{5}DocumentSummaryInformation"]).map(|d| property_set(&d)).unwrap_or_default(),
    );
    let pick = |m: &HashMap<u32, String>, id: u32, key: u32| m.get(&id).filter(|s| !s.is_empty()).cloned().or_else(|| p.text(key));
    let props = ProjectProps {
        title: pick(&summary, 2, K_TITLE),
        author: pick(&summary, 4, K_AUTHOR),
        last_author: summary.get(&8).cloned(),
        manager: pick(&doc_summary, 14, K_MANAGER),
        company: pick(&doc_summary, 15, K_COMPANY),
        start: p.timestamp(K_START_DATE),
        finish: p.timestamp(K_FINISH_DATE),
        status_date: p.timestamp(K_STATUS_DATE),
        minutes_per_day: p.i32(K_MINUTES_PER_DAY),
        minutes_per_week: p.i32(K_MINUTES_PER_WEEK),
        days_per_month: p.u16(K_DAYS_PER_MONTH) as i32,
        default_units,
        default_calendar: p.text(K_DEFAULT_CALENDAR_NAME),
        critical_slack_limit_days: p.i32(K_CRITICAL_SLACK_LIMIT),
        currency_symbol: p.text(K_CURRENCY_SYMBOL),
        app_name,
        app_version: ctx.app_version,
    };

    ctx.lookup = read_lookup(&ctx);
    let (mut calendars, resource_calendars) = read_calendars(&ctx);
    let mut resources = read_resources(&ctx)?;
    let mut tasks = read_tasks(&ctx, &props)?;
    let task_uids: HashSet<i32> = tasks.iter().map(|t| t.uid).collect();
    let relations = read_relations(&ctx, &task_uids);
    let resource_uids: HashSet<i32> = resources.iter().map(|r| r.uid).collect();
    let assignments = read_assignments(&ctx, &task_uids, &resource_uids)?;

    // Los calendarios derivados que no pertenecen a ningun recurso existente se descartan.
    let used: HashSet<i32> = resources.iter().filter_map(|r| resource_calendars.get(&r.uid).copied()).collect();
    calendars.retain(|c| c.base_uid.is_none() || used.contains(&c.uid));

    // Calendario y fechas de cada recurso.
    let cal_names: HashMap<i32, (Option<String>, Option<i32>)> = calendars.iter().map(|c| (c.uid, (c.name.clone(), c.base_uid))).collect();
    for r in resources.iter_mut() {
        if let Some(&cal_uid) = resource_calendars.get(&r.uid) {
            r.calendar_uid = Some(cal_uid);
            if let Some((name, base)) = cal_names.get(&cal_uid) {
                r.base_calendar = match base {
                    Some(b) => cal_names.get(b).and_then(|x| x.0.clone()),
                    None => name.clone(),
                };
            }
        }
        let own = assignments.iter().filter(|a| a.resource_uid == Some(r.uid));
        r.start = own.clone().filter_map(|a| a.start).min();
        r.finish = own.filter_map(|a| a.finish).max();
    }
    // Los calendarios de recurso llevan el nombre del recurso.
    for c in calendars.iter_mut() {
        if c.base_uid.is_some() {
            c.name = resources.iter().find(|r| r.calendar_uid == Some(c.uid)).and_then(|r| r.name.clone());
        }
    }
    calendars.retain(|c| c.name.is_some());

    tasks.sort_by_key(|t| t.id);
    resources.sort_by_key(|r| r.id);

    let task_dir = [PROJECT_DIR, "TBkndTask"];
    let rsc_dir = [PROJECT_DIR, "TBkndRsc"];
    Ok(Project {
        props,
        tasks,
        resources,
        assignments,
        relations,
        calendars,
        tables: read_tables(&ctx),
        task_aliases: read_aliases(&ctx, &task_dir, &TASK),
        resource_aliases: read_aliases(&ctx, &rsc_dir, &RESOURCE),
    })
}

// ---------------------------------------------------------------------------------------------
// Tareas
// ---------------------------------------------------------------------------------------------

struct TaskBits {
    flags: [(usize, u32); 20],
    milestone: (usize, u32),
    active: (usize, u32),
    manual: (usize, u32),
}

fn task_bits(app_version: i32) -> TaskBits {
    let mut flags = [(0usize, 0u32); 20];
    if app_version <= 14 {
        for (i, f) in flags.iter_mut().enumerate() {
            *f = (35, 0x40 << i);
        }
        TaskBits { flags, milestone: (8, 0x20), active: (8, 0x04), manual: (8, 0x08) }
    } else {
        for (i, f) in flags.iter_mut().enumerate() {
            *f = if i < 10 { (24, 0x2 << i) } else { (33, 0x2 << (i - 10)) };
        }
        TaskBits { flags, milestone: (10, 0x02), active: (8, 0x40), manual: (8, 0x80) }
    }
}

fn read_tasks(ctx: &Ctx, props: &ProjectProps) -> Result<Vec<Task>, String> {
    let map_data = K_TASK_FIELD_MAP.iter().find_map(|k| ctx.props.bytes(*k)).ok_or("El archivo no trae el mapa de campos de tareas.")?;
    let fm = FieldMap::parse(map_data, &TASK);
    let dir = [PROJECT_DIR, "TBkndTask"];
    let var = ctx.var(&dir)?;
    let fixed_meta = FixedMeta::parse(ctx.stream(&dir, "FixedMeta")?, 47)?;
    let fixed = FixedData::with_meta(&fixed_meta, ctx.stream(&dir, "FixedData")?, fm.max_fixed[0]);
    let fixed2_meta = FixedMeta::parse_guess(ctx.stream(&dir, "Fixed2Meta")?, fixed.len(), &[92, 93, 94, 95, 96])?;
    let fixed2 = FixedData::with_meta(&fixed2_meta, ctx.stream(&dir, "Fixed2Data")?, 0);

    let uid_offset = fm.fixed_offset(tf::UNIQUE_ID).ok_or("Mapa de campos de tareas incompleto (Unique ID).")?;
    let id_offset = fm.fixed_offset(tf::ID).ok_or("Mapa de campos de tareas incompleto (ID).")?;
    let max_size = fm.max_fixed[0];

    // Id unico -> posicion en FixedData. Se recorre hacia atras: ante duplicados, vale el ultimo.
    // Los tres primeros registros no son tareas. `None` marca una tarea eliminada.
    let mut task_map: BTreeMap<i32, Option<usize>> = BTreeMap::new();
    for i in (3..fixed_meta.adjusted_count).rev() {
        let (data, meta) = match (fixed.get(i), fixed2.get(i), fixed_meta.item(i)) {
            (Some(d), Some(_), Some(m)) => (d, m),
            _ => continue,
        };
        let flags = u32_at(meta, 0);
        if flags & 0x02 != 0 {
            task_map.entry(u16_at(data, 0) as i32).or_insert(None);
        } else if data.len() == NULL_TASK_BLOCK_SIZE {
            task_map.entry(i32_at(data, 0)).or_insert(Some(i));
        } else if max_size == 0 || (data.len() * 100) / max_size > 75 {
            let uid = i32_at(data, uid_offset);
            let known = task_map.contains_key(&uid);
            if (!known || var.has_uid(uid)) && (!known || flags & 0x04 == 0) {
                task_map.insert(uid, Some(i));
            }
        }
    }

    let bits = task_bits(ctx.app_version);
    let mut tasks: Vec<Task> = Vec::new();
    let mut removed: HashSet<i32> = HashSet::new();
    let mut by_file_id: HashMap<i32, usize> = HashMap::new();
    // Orden real de las filas (clave de orden -> id unico) y filas en blanco (id -> id unico).
    let mut task_order: BTreeMap<i64, i32> = BTreeMap::new();
    let mut null_order: BTreeMap<i32, i32> = BTreeMap::new();

    for (&uid, &pos) in task_map.iter() {
        let pos = match pos {
            Some(p) if p < fixed.len() => p,
            _ => continue,
        };
        let raw = match fixed.get(pos) {
            Some(d) => d,
            None => continue,
        };
        if raw.len() == NULL_TASK_BLOCK_SIZE {
            null_order.entry(i32_at(raw, 4)).or_insert(i32_at(raw, 0));
            continue;
        }
        let mut data = raw.to_vec();
        if data.len() < max_size {
            data.resize(max_size + 8, 0);
        }
        let file_id = i32_at(&data, id_offset);
        let meta = fixed_meta.item(pos);
        let meta2 = fixed2_meta.item(pos);
        let data2 = fixed2.get(pos);

        if let Some(&prev) = by_file_id.get(&file_id) {
            if !var.has_uid(uid) {
                continue; // tarea "fantasma" que comparte id con una valida
            }
            if tasks[prev].name.is_none() {
                removed.insert(tasks[prev].uid);
            }
        }

        let fixed_blocks = [Some(data.as_slice()), data2];
        let fields = fm.read_all(uid, &fixed_blocks, &var, &ctx.sc, &ctx.lookup);

        let manual = bit(meta2, bits.manual.0, bits.manual.1);
        let active = meta2.is_none() || bit(meta2, bits.active.0, bits.active.1);
        let milestone = bit(meta, bits.milestone.0, bits.milestone.1);
        let mut flags = [false; 20];
        for (i, f) in flags.iter_mut().enumerate() {
            *f = bit(meta, bits.flags[i].0, bits.flags[i].1);
        }

        // Las tareas programadas automaticamente usan los valores "scheduled".
        let mut start = date_of(&fields, tf::START);
        let mut finish = date_of(&fields, tf::FINISH);
        let sched_start = date_of(&fields, tf::SCHEDULED_START);
        let sched_finish = date_of(&fields, tf::SCHEDULED_FINISH);
        if start.is_none() || (sched_start.is_some() && !manual) {
            start = sched_start;
        }
        if finish.is_none() || (sched_finish.is_some() && !manual) {
            finish = sched_finish;
        }
        let duration = if manual { dur_of(&fields, tf::MANUAL_DURATION) } else { dur_of(&fields, tf::SCHEDULED_DURATION) };

        let late_start = date_of(&fields, tf::LATE_START);
        let late_finish = date_of(&fields, tf::LATE_FINISH);
        match int_of(&fields, tf::CONSTRAINT_TYPE).unwrap_or(0) {
            1 => {
                // Lo mas tarde posible.
                if let (Some(s), Some(ls)) = (start, late_start) {
                    if s < ls {
                        start = Some(ls);
                    }
                }
                if let (Some(f), Some(lf)) = (finish, late_finish) {
                    if f < lf {
                        finish = Some(lf);
                    }
                }
            }
            5 | 7 => {
                if let (Some(s), Some(f)) = (start, finish) {
                    if f < s {
                        finish = late_finish;
                    }
                }
            }
            _ => {}
        }

        let name = text_of(&fields, tf::NAME);
        let created = date_of(&fields, tf::CREATED);
        let is_blank = |d: Option<i64>| d.map_or(true, |v| v == 0);
        if name.is_none() && (is_blank(start) || is_blank(finish) || is_blank(created)) {
            null_order.entry(i32_at(&data, 4)).or_insert(i32_at(&data, 0));
            continue;
        }

        match data2 {
            Some(d2) if d2.len() >= 24 => {
                task_order.insert(i64_at(d2, 16), uid);
            }
            _ => {
                null_order.insert(file_id, uid);
            }
        }

        let notes = text_of(&fields, tf::NOTES);
        by_file_id.insert(file_id, tasks.len());
        tasks.push(Task {
            id: file_id,
            uid,
            name,
            outline_level: int_of(&fields, tf::OUTLINE_LEVEL).unwrap_or(0),
            parent_uid: None,
            start,
            finish,
            duration,
            milestone,
            active,
            manual,
            notes,
            flags,
            fields,
            ..Task::default()
        });
    }
    tasks.retain(|t| !removed.contains(&t.uid));

    // Renumera los ID segun el orden real guardado en el archivo, dejando hueco para las filas
    // en blanco (asi los ID coinciden con los que muestra Project).
    let has_root = tasks.iter().any(|t| t.uid == 0);
    let inc = ((null_order.len() as i64 / 1000) + 1) * 2000;
    let mut slots: BTreeMap<i64, i32> = BTreeMap::new();
    let mut next = if has_root { 0 } else { inc };
    for (_, &uid) in task_order.iter() {
        slots.insert(next, uid);
        next += inc;
    }
    let mut insertion = 0i64;
    let mut offsets: HashMap<i64, i64> = HashMap::new();
    for (&id_value, &uid) in null_order.iter() {
        let base = (id_value as i64 - insertion) * inc;
        let mut target = base;
        let mut offset = offsets.get(&base).map_or(0, |o| o + 1);
        insertion += 1;
        let mut ok = true;
        while slots.contains_key(&target) {
            offset += 1;
            if offset == inc {
                ok = false;
                break;
            }
            target = base - (inc - offset);
        }
        if ok {
            offsets.insert(base, offset);
            slots.insert(target, uid);
        }
    }
    let new_ids: HashMap<i32, i32> = slots.values().enumerate().map(|(i, &uid)| (uid, i as i32 + if has_root { 0 } else { 1 })).collect();
    for t in tasks.iter_mut() {
        if let Some(&id) = new_ids.get(&t.uid) {
            t.id = id;
        }
    }
    tasks.sort_by_key(|t| t.id);

    // Jerarquia: padre, tarea resumen y numero de esquema.
    let mut stack: Vec<(i32, usize)> = Vec::new(); // (nivel, indice)
    let mut child_count: Vec<u32> = vec![0; tasks.len()];
    let mut root_children = 0u32;
    for i in 0..tasks.len() {
        let level = tasks[i].outline_level;
        while stack.last().map_or(false, |&(l, _)| l >= level) {
            stack.pop();
        }
        let number = match stack.last() {
            Some(&(_, p)) => {
                tasks[p].summary = true;
                tasks[i].parent_uid = Some(tasks[p].uid);
                child_count[p] += 1;
                // Los hijos directos de la tarea resumen del proyecto (id 0) no llevan prefijo.
                if tasks[p].uid == 0 && tasks[p].outline_level == 0 {
                    child_count[p].to_string()
                } else {
                    format!("{}.{}", tasks[p].outline_number, child_count[p])
                }
            }
            None if tasks[i].uid == 0 && level == 0 => "0".to_string(),
            None => {
                root_children += 1;
                root_children.to_string()
            }
        };
        tasks[i].outline_number = text_of(&tasks[i].fields, tf::OUTLINE_NUMBER).filter(|s| !s.is_empty()).unwrap_or(number);
        // Sin codigo EDT propio, Project muestra el numero de esquema.
        tasks[i].wbs = text_of(&tasks[i].fields, tf::WBS).filter(|s| !s.is_empty()).unwrap_or_else(|| tasks[i].outline_number.clone());
        stack.push((level, i));
    }

    // Holgura total y ruta critica (se calculan, no se guardan en el archivo).
    for t in tasks.iter_mut() {
        let start_slack = dur_of(&t.fields, tf::START_SLACK);
        let finish_slack = dur_of(&t.fields, tf::FINISH_SLACK);
        t.total_slack = if date_of(&t.fields, tf::ACTUAL_START).is_some() {
            finish_slack
        } else {
            match (t.duration, start_slack, finish_slack) {
                (Some(_), Some(s), Some(f)) => {
                    let (sh, fh) = (to_hours(&ctx.sc, s).unwrap_or(s.value), to_hours(&ctx.sc, f).unwrap_or(f.value));
                    Some(if sh < fh { s } else { f })
                }
                _ => None,
            }
        };
        let pct = num_of(&t.fields, tf::PERCENT_COMPLETE).unwrap_or(0.0);
        // Project no marca como criticas las tareas programadas manualmente que tienen guardados
        // los campos de texto de inicio/fin/duracion, aunque esten vacios (verificado contra
        // XML exportado por Project).
        let no_text = |id: u16| !t.fields.contains_key(&id);
        let plain = !t.manual || (no_text(tf::START_TEXT) && no_text(tf::FINISH_TEXT) && no_text(tf::DURATION_TEXT));
        t.critical = match t.total_slack {
            Some(ts) if date_of(&t.fields, tf::ACTUAL_FINISH).is_none() => {
                let limit = props.critical_slack_limit_days as f64;
                let slack = if limit != 0.0 && ts.value != 0.0 && ts.unit != Unit::Days {
                    to_hours(&ctx.sc, ts).map_or(ts.value, |h| if ctx.sc.minutes_per_day > 0.0 { h * 60.0 / ctx.sc.minutes_per_day } else { h })
                } else {
                    ts.value
                };
                slack <= limit && pct != 100.0 && plain
            }
            _ => false,
        };
    }
    Ok(tasks)
}

// ---------------------------------------------------------------------------------------------
// Recursos
// ---------------------------------------------------------------------------------------------

fn read_resources(ctx: &Ctx) -> Result<Vec<Resource>, String> {
    let map_data = match K_RESOURCE_FIELD_MAP.iter().find_map(|k| ctx.props.bytes(*k)) {
        Some(d) => d,
        None => return Ok(Vec::new()),
    };
    let fm = FieldMap::parse(map_data, &RESOURCE);
    let dir = [PROJECT_DIR, "TBkndRsc"];
    if ctx.cfb.path(&dir).is_none() {
        return Ok(Vec::new());
    }
    let var = ctx.var(&dir)?;
    let fixed_meta = FixedMeta::parse(ctx.stream(&dir, "FixedMeta")?, 37)?;
    let fixed = FixedData::with_meta(&fixed_meta, ctx.stream_enc(&dir, "FixedData")?, 0);
    let fixed2_meta = FixedMeta::parse_guess(ctx.stream(&dir, "Fixed2Meta")?, fixed.len(), &[50, 51])?;
    let fixed2 = FixedData::with_meta(&fixed2_meta, ctx.stream_enc(&dir, "Fixed2Data")?, 0);

    let (uid_offset, id_offset) = match (fm.fixed_offset(rf::UNIQUE_ID), fm.fixed_offset(rf::ID)) {
        (Some(u), Some(i)) => (u, i),
        _ => return Ok(Vec::new()),
    };
    let mut positions: BTreeMap<i32, usize> = BTreeMap::new();
    for i in 0..fixed_meta.adjusted_count {
        if let Some(d) = fixed.get(i) {
            if d.len() >= fm.max_fixed[0] {
                positions.entry(u16_at(d, uid_offset) as i32).or_insert(i);
            }
        }
    }

    let new_layout = ctx.app_version > 14;
    let mut out = Vec::new();
    for &uid in var.uids() {
        let pos = match positions.get(&uid) {
            Some(&p) => p,
            None => continue,
        };
        let data = match fixed.get(pos) {
            Some(d) => d,
            None => continue,
        };
        let meta = fixed_meta.item(pos);
        let meta2 = fixed2_meta.item(pos);
        let fixed_blocks = [Some(data), fixed2.get(pos)];
        let fields = fm.read_all(uid, &fixed_blocks, &var, &ctx.sc, &ctx.lookup);
        let mut flags = [false; 20];
        for (i, f) in flags.iter_mut().enumerate() {
            // Flag1..Flag20; en el archivo el orden de bits es Flag10, Flag1..Flag9, Flag11..Flag20.
            *f = if new_layout {
                match i {
                    9 => bit(meta, 19, 0x40),
                    0..=8 => bit(meta, 19, 0x80 << i),
                    _ => bit(meta, 24, 0x80 << (i - 10)),
                }
            } else {
                match i {
                    9 => bit(meta, 28, 0x20),
                    0..=8 => bit(meta, 28, 0x40 << i),
                    _ => bit(meta, 28, 0x8000 << (i - 10)),
                }
            };
        }
        let is_work = if new_layout { meta.map_or(false, |m| u8_at(m, 12) & 0x10 != 0) } else { meta.map_or(false, |m| u8_at(m, 9) & 0x02 != 0) };
        let kind = if is_work {
            ResourceKind::Work
        } else if meta2.map_or(false, |m| u8_at(m, 8) & 0x10 != 0) {
            ResourceKind::Cost
        } else {
            ResourceKind::Material
        };
        out.push(Resource {
            id: i32_at(data, id_offset),
            uid,
            name: text_of(&fields, rf::NAME).filter(|s| !s.is_empty()),
            kind,
            calendar_uid: None,
            base_calendar: None,
            start: None,
            finish: None,
            notes: text_of(&fields, rf::NOTES),
            flags,
            fields,
        });
    }
    Ok(out)
}

// ---------------------------------------------------------------------------------------------
// Relaciones (predecesoras) y asignaciones
// ---------------------------------------------------------------------------------------------

fn read_relations(ctx: &Ctx, task_uids: &HashSet<i32>) -> Vec<Relation> {
    let dir = [PROJECT_DIR, "TBkndCons"];
    let mut out = Vec::new();
    let (meta, data) = match (ctx.stream_opt(&dir, "FixedMeta"), ctx.stream_enc(&dir, "FixedData")) {
        (Some(m), Ok(d)) => (m, d),
        _ => return out,
    };
    let fixed_meta = match FixedMeta::parse(meta, 10) {
        Ok(m) => m,
        Err(_) => return out,
    };
    let fixed = FixedData::with_meta_sized(&fixed_meta, data, 20);
    let (dur_off, units_off) = if ctx.app_version > 14 { (14, 18) } else { (16, 14) };
    let mut seen: HashSet<(i32, i32)> = HashSet::new();
    for i in 0..fixed_meta.item_count {
        let m = match fixed_meta.item(i) {
            Some(m) => m,
            None => continue,
        };
        if u16_at(m, 0) != 0 {
            continue; // eliminada
        }
        let d = match fixed.index_from_offset(i32_at(m, 4).max(0) as usize).and_then(|ix| fixed.get(ix)) {
            Some(d) if d.len() >= 14 => d,
            _ => continue,
        };
        let (pred, succ) = (i32_at(d, 4), i32_at(d, 8));
        if pred == 0 || succ == 0 || pred == succ || !task_uids.contains(&pred) || !task_uids.contains(&succ) {
            continue;
        }
        if !seen.insert((pred, succ)) {
            continue;
        }
        let kind = match u16_at(d, 12) {
            0 => RelType::FinishFinish,
            2 => RelType::StartFinish,
            3 => RelType::StartStart,
            _ => RelType::FinishStart,
        };
        let lag = adjusted_duration(&ctx.sc, i32_at(d, dur_off), duration_units(u16_at(d, units_off), None));
        out.push(Relation { uid: i32_at(d, 0), pred_uid: pred, succ_uid: succ, kind, lag });
    }
    out
}

fn read_assignments(ctx: &Ctx, task_uids: &HashSet<i32>, resource_uids: &HashSet<i32>) -> Result<Vec<Assignment>, String> {
    let mut out = Vec::new();
    let map_data = match K_ASSIGNMENT_FIELD_MAP.iter().find_map(|k| ctx.props.bytes(*k)) {
        Some(d) => d,
        None => return Ok(out),
    };
    let fm = FieldMap::parse(map_data, &ASSIGNMENT);
    let dir = [PROJECT_DIR, "TBkndAssn"];
    if ctx.cfb.path(&dir).is_none() {
        return Ok(out);
    }
    let var = ctx.var(&dir)?;
    let fixed_meta = FixedMeta::parse(ctx.stream(&dir, "FixedMeta")?, 34)?;
    let fixed = FixedData::sequential(ctx.stream_enc(&dir, "FixedData")?, 110);
    let fixed2 = match ctx.stream_enc(&dir, "Fixed2Data") {
        Ok(d) => FixedData::sequential(d, 48),
        Err(_) => FixedData::empty(),
    };
    let (uid_off, task_off, res_off) = match (fm.fixed_offset(af::UNIQUE_ID), fm.fixed_offset(af::TASK_UNIQUE_ID), fm.fixed_offset(af::RESOURCE_UNIQUE_ID)) {
        (Some(a), Some(b), Some(c)) => (a, b, c),
        _ => return Ok(out),
    };
    let mut seen: HashSet<(i32, Option<i32>)> = HashSet::new();
    for i in 0..fixed_meta.item_count {
        let m = match fixed_meta.item(i) {
            Some(m) if m[0] == 0 => m,
            _ => continue,
        };
        let idx = match fixed.index_from_offset(i32_at(m, 4).max(0) as usize) {
            Some(ix) => ix,
            None => continue,
        };
        let mut data = match fixed.get(idx) {
            Some(d) => d.to_vec(),
            None => continue,
        };
        if data.len() < fm.max_fixed[0] {
            data.resize(fm.max_fixed[0], 0);
        }
        let uid = i32_at(&data, uid_off);
        if !var.has_uid(uid) {
            continue;
        }
        let task_uid = i32_at(&data, task_off);
        if !task_uids.contains(&task_uid) {
            continue;
        }
        let raw_res = i32_at(&data, res_off);
        let resource_uid = if resource_uids.contains(&raw_res) { Some(raw_res) } else { None };
        if !seen.insert((task_uid, resource_uid)) {
            continue;
        }
        let fixed_blocks = [Some(data.as_slice()), fixed2.get(idx)];
        let fields = fm.read_all(uid, &fixed_blocks, &var, &ctx.sc, &ctx.lookup);
        out.push(Assignment {
            uid,
            task_uid,
            resource_uid: if raw_res == ASSIGNMENT_NULL_RESOURCE { None } else { resource_uid },
            units: num_of(&fields, af::ASSIGNMENT_UNITS).map(|u| u / 100.0),
            work: dur_of(&fields, af::WORK),
            start: date_of(&fields, af::START),
            finish: date_of(&fields, af::FINISH),
            fields,
        });
    }
    Ok(out)
}

// ---------------------------------------------------------------------------------------------
// Calendarios, listas de valores, alias y tablas (columnas de las vistas)
// ---------------------------------------------------------------------------------------------

/// Devuelve los calendarios y el mapa id unico de recurso -> id de calendario.
fn read_calendars(ctx: &Ctx) -> (Vec<Calendar>, HashMap<i32, i32>) {
    let dir = [PROJECT_DIR, "TBkndCal"];
    let mut cals: Vec<Calendar> = Vec::new();
    let mut by_resource: HashMap<i32, i32> = HashMap::new();
    let (var, meta, data) = match (ctx.var(&dir), ctx.stream_opt(&dir, "FixedMeta"), ctx.stream_enc(&dir, "FixedData")) {
        (Ok(v), Some(m), Ok(d)) => (v, m, d),
        _ => return (cals, by_resource),
    };
    let fixed_meta = match FixedMeta::parse(meta, 10) {
        Ok(m) => m,
        Err(_) => return (cals, by_resource),
    };
    let fixed = FixedData::with_meta(&fixed_meta, data, 12);
    let (cal_off, base_off, res_off) = if ctx.app_version > 14 { (8, 0, 4) } else { (0, 4, 8) };
    let mut seen: HashSet<i32> = HashSet::new();
    for i in 0..fixed.len() {
        let d = match fixed.get(i) {
            Some(d) if d.len() >= 8 => d,
            _ => continue,
        };
        let mut off = 0usize;
        while off + 12 <= d.len() {
            let cal_id = i32_at(d, off + cal_off);
            let base_id = i32_at(d, off + base_off);
            let res_id = i32_at(d, off + res_off);
            if cal_id > 0 && seen.insert(cal_id) {
                let is_base = base_id <= 0 || base_id == cal_id;
                let hours = var.get(cal_id, 8);
                let mut days: [DayHours; 7] = Default::default();
                if let Some(h) = hours {
                    for (day, slot) in days.iter_mut().enumerate() {
                        let o = 60 * day;
                        if u16_at(h, o) == 1 {
                            continue; // hereda
                        }
                        let periods = (u16_at(h, o + 2) as usize).min(5);
                        let mut ranges = Vec::new();
                        for p in 0..periods {
                            let start = (u16_at(h, o + 8 + p * 2) as u32 / 10) % 1440;
                            let len = u16_at(h, o + 20 + p * 4) as u32 / 10;
                            ranges.push((start, (start + len).min(1440)));
                        }
                        *slot = Some(ranges);
                    }
                }
                if is_base {
                    if res_id > 0 {
                        by_resource.entry(res_id).or_insert(cal_id);
                    }
                } else {
                    by_resource.insert(res_id, cal_id);
                }
                cals.push(Calendar {
                    uid: cal_id,
                    name: if is_base { var.text(cal_id, 1).filter(|s| !s.is_empty()) } else { None },
                    base_uid: if is_base { None } else { Some(base_id) },
                    resource_uid: if res_id > 0 { Some(res_id) } else { None },
                    days,
                });
            }
            off += 12;
        }
    }
    // Un calendario derivado cuyo base no existe no es valido.
    let named: HashSet<i32> = cals.iter().filter(|c| c.base_uid.is_none() && c.name.is_some()).map(|c| c.uid).collect();
    cals.retain(|c| c.base_uid.map_or(true, |b| named.contains(&b)));
    let valid: HashSet<i32> = cals.iter().map(|c| c.uid).collect();
    by_resource.retain(|_, c| valid.contains(c));
    (cals, by_resource)
}

fn read_lookup(ctx: &Ctx) -> Lookup {
    let dir = [PROJECT_DIR, "TBkndOutlCode"];
    let mut out = Lookup::default();
    let (var, meta, data) = match (ctx.var(&dir), ctx.stream_opt(&dir, "FixedMeta"), ctx.stream_opt(&dir, "FixedData")) {
        (Ok(v), Some(m), Some(d)) => (v, m, d),
        _ => return out,
    };
    let fixed_meta = match FixedMeta::parse(meta, 10) {
        Ok(m) => m,
        Err(_) => return out,
    };
    let fixed = FixedData::with_meta(&fixed_meta, data, 0);
    for i in 0..fixed.len() {
        if let Some(d) = fixed.get(i) {
            let id = u16_at(d, 4) as i32;
            if let Some(v) = var.get(id, 22) {
                out.entry(id).or_insert_with(|| v.to_vec());
            }
        }
    }
    out
}

fn read_aliases(ctx: &Ctx, dir: &[&str], table: &'static fields::FieldTable) -> HashMap<u16, String> {
    let mut out = HashMap::new();
    let props = match ctx.stream_enc(dir, "Props") {
        Ok(d) => Props::parse(&d),
        Err(_) => return out,
    };
    let d = match props.bytes(K_CUSTOM_FIELDS) {
        Some(d) => d,
        None => return out,
    };
    let block_size = u32_at(d, 0) as usize;
    let count = u32_at(d, 8) as usize;
    let mut off = 12usize;
    let mut i = 0usize;
    while i < count && off < block_size && off + 8 <= d.len() {
        let field = u32_at(d, off);
        let alias_off = u32_at(d, off + 4) as usize + 4;
        off += 8;
        i += 1;
        let idx = (field & 0xFFFF) as usize;
        if field & 0xFFFF_0000 != table.base || idx >= table.canon.len() || alias_off >= d.len() {
            continue;
        }
        let alias = utf16(d, alias_off);
        if !alias.is_empty() {
            out.insert(table.canon[idx], alias);
        }
    }
    out
}

fn read_tables(ctx: &Ctx) -> Vec<Table> {
    let dir = [VIEW_DIR, "CTable"];
    let mut out = Vec::new();
    let (var, data) = match (ctx.var(&dir), ctx.stream_opt(&dir, "FixedData")) {
        (Ok(v), Some(d)) => (v, d),
        _ => return out,
    };
    let fixed = FixedData::sequential(data, 230);
    for i in 0..fixed.len() {
        let d = match fixed.get(i) {
            Some(d) => d,
            None => continue,
        };
        let id = i32_at(d, 0);
        let name: String = utf16(d, 4).chars().filter(|&c| c != '&').collect();
        let cols = [8u16, 7, 6].iter().find_map(|k| var.get(id, *k));
        let mut columns = Vec::new();
        if let Some(c) = cols.filter(|c| c.len() > 6) {
            let n = u16_at(c, 4) as usize + 1;
            let mut index = 12usize;
            for _ in 0..n {
                if index + 13 > c.len() {
                    break;
                }
                let title = utf16(c.get(..(index + 115).min(c.len())).unwrap_or(c), index + 13);
                columns.push(Column { field: u32_at(c, index), title: if title.is_empty() { None } else { Some(title) }, width: u8_at(c, index + 4) });
                index += 115;
            }
        }
        out.push(Table { name, resource: u16_at(d, 108) == 1, columns });
    }
    out
}

// ---------------------------------------------------------------------------------------------
// Propiedades del documento OLE (titulo, autor...)
// ---------------------------------------------------------------------------------------------

/// Lee las propiedades de texto de un "property set" OLE: id de propiedad -> texto.
fn property_set(d: &[u8]) -> HashMap<u32, String> {
    let mut out = HashMap::new();
    if d.len() < 48 || u16_at(d, 0) != 0xFFFE {
        return out;
    }
    let sec = u32_at(d, 44) as usize;
    let count = (u32_at(d, sec + 4) as usize).min(256);
    let entries: Vec<(u32, usize)> = (0..count).map(|i| (u32_at(d, sec + 8 + i * 8), sec + u32_at(d, sec + 12 + i * 8) as usize)).collect();
    let codepage = entries.iter().find(|e| e.0 == 1).map_or(1252, |e| u16_at(d, e.1 + 4));
    for (id, off) in entries {
        let kind = u32_at(d, off) & 0xFFFF;
        let len = u32_at(d, off + 4) as usize;
        let text = match kind {
            30 => {
                let raw = d.get(off + 8..(off + 8).saturating_add(len)).unwrap_or(&[]);
                let raw = &raw[..raw.iter().position(|&b| b == 0).unwrap_or(raw.len())];
                if codepage == 65001 {
                    String::from_utf8_lossy(raw).into_owned()
                } else {
                    raw.iter().map(|&b| cp1252(b)).collect()
                }
            }
            31 => utf16(d.get(..(off + 8).saturating_add(len * 2).min(d.len())).unwrap_or(d), off + 8),
            _ => continue,
        };
        out.insert(id, text);
    }
    out
}
